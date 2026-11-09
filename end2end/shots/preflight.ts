import { existsSync } from "node:fs";
import { join } from "node:path";

import { listOllamaModels } from "./examiner/d3.ts";
import {
    APP_PORT,
    LOCAL_CDN_PORT,
    TRAILBASE_PORT,
    type CdnSource,
    type ServeMode,
} from "./config.ts";
import {
    hasAwsProfile,
    inspectFonts,
    isDistPoisoned,
    isInside,
    listListeners,
    probeCorpus,
    type CorpusProbe,
    type ListenerInfo,
} from "./probe.ts";

/** Facts the capture depends on. */
export interface PreflightFacts {
    serveMode: ServeMode;
    nodeModulesPresent: boolean;
    /** Branded fonts available: corpus sources (build) or baked bundle (dist). */
    fontsReady: boolean;
    /** Tokenizer dictionary present in the local corpus. */
    dictPresent: boolean;
    /** Built bundle still carries the trunk live-reload bridge. */
    distPoisoned: boolean;
    corpusProbe: CorpusProbe | null;
    listeners: ListenerInfo[];
    appBaseUrl: string;
    cdnBaseUrl: string;
    cdnCacheDir: string | null;
    /** Root of the checkout the capture belongs to. */
    repoRoot: string;
    /** `null` when the run does not upload. */
    awsProfilePresent: boolean | null;
    /** The e2e stack has its TrailBase URL. */
    trailBaseUrlConfigured: boolean;
    /** The e2e stack has the admin password of the fixture. */
    adminPasswordConfigured: boolean;
    outDirInsideRepo: boolean;
    /** The run examines captures with d3; null facts mean Ollama is unreachable. */
    examinerEnabled: boolean;
    examModelTag: string;
    ollamaUrl: string;
    /** `null` — Ollama unreachable (an error when the run examines). */
    ollamaModels: string[] | null;
}

export interface PreflightReport {
    errors: string[];
    warnings: string[];
    info: string[];
}

export interface PreflightOptions {
    serveMode: ServeMode;
    end2endRoot: string;
    repoRoot: string;
    distDir: string;
    manifestPath: string;
    appBaseUrl: string;
    cdnSource: CdnSource;
    outDir: string;
    upload: boolean;
    examine: boolean;
    examModelTag: string;
    ollamaUrl: string;
}

/**
 * Turns the collected facts into the report.
 *
 * Pure on purpose: the verdicts are the part worth unit-testing, the
 * probing around them is environment noise.
 */
export function evaluatePreflight(facts: PreflightFacts): PreflightReport {
    const errors: string[] = [];
    const warnings: string[] = [];

    errors.push(...checkToolchain(facts));
    errors.push(...checkBundle(facts));
    warnings.push(...checkCorpus(facts));
    errors.push(...checkPorts(facts));
    errors.push(...checkUpload(facts));
    errors.push(...checkOutput(facts));
    errors.push(...checkExaminer(facts));

    return { errors, warnings, info: describe(facts) };
}

/** What the run is about to photograph. */
function describe(facts: PreflightFacts): string[] {
    return [
        `Режим сборки: ${facts.serveMode === "dist" ? "готовый dist" : "сборка из исходников"}`,
        `Приложение: ${facts.appBaseUrl}`,
        `CDN: ${facts.cdnBaseUrl}`,
        `Корпус: ${facts.cdnCacheDir ?? "локального корпуса нет — контент с прод-CDN"}`,
    ];
}

/** The dependencies of the run itself. */
function checkToolchain(facts: PreflightFacts): string[] {
    const errors: string[] = [];
    if (!facts.nodeModulesPresent) {
        errors.push("end2end/node_modules отсутствует — выполните `npm ci` в end2end/");
    }
    if (!facts.trailBaseUrlConfigured) {
        errors.push(
            "TRAILBASE_URL не задан — нужен файл end2end/.env (он в .gitignore: скопируй из основного чекаута) или переменная окружения",
        );
    }
    if (!facts.adminPasswordConfigured) {
        errors.push(
            "ORIGA_ADMIN_PASSWORD не задан — нужен файл end2end/.env (он в .gitignore: скопируй из основного чекаута) или переменная окружения",
        );
    }
    return errors;
}

/**
 * The bundle that is about to be photographed.
 *
 * Without the branded fonts the screenshots would be sans-serif, and a
 * poisoned bundle breaks the WASM load mid-capture.
 */
function checkBundle(facts: PreflightFacts): string[] {
    const errors: string[] = [];
    if (facts.serveMode === "build") {
        if (!facts.fontsReady) {
            errors.push(
                "в локальном корпусе нет cdn/fonts/ — бандл соберётся без @font-face (sans-serif). " +
                    "ORIGA_CDN_CACHE_DIR параметризует раздачу контента, но реестр шрифтов билда " +
                    "build.rs читает из repo-локального cdn/fonts: для брендовых скринов симлинк cdn → корпус",
            );
        }
        return errors;
    }
    if (facts.distPoisoned) {
        errors.push(
            "origa_ui/dist/index.html содержит trunk live-reload мост — статическая раздача рвёт загрузку WASM. " +
                "Пересоберите: `cd origa_ui && trunk build --release`",
        );
    }
    if (!facts.fontsReady) {
        errors.push("в собранном бандле нет @font-face для Cormorant или woff2-файлов — скрины будут в sans-serif");
    }
    return errors;
}

/** The corpus the app reads its content from. */
function checkCorpus(facts: PreflightFacts): string[] {
    const warnings: string[] = [];
    if (facts.serveMode === "build" && !facts.dictPresent) {
        warnings.push(
            "нет cdn/dictionaries/sudachidict-20260723/ — токенизатор не будет работать (влияет на разбор слов)",
        );
    }
    if (facts.corpusProbe && !facts.corpusProbe.present) {
        warnings.push(
            `локальный корпус отстаёт: нет ${facts.corpusProbe.path} — на экране будет контент старше деплоя`,
        );
    }
    return warnings;
}

/**
 * The servers of the run.
 *
 * A listener from another checkout is a stranger the run would reuse as
 * its own server (Playwright's `reuseExistingServer`; the same trap via
 * global-setup for TrailBase) — the capture would photograph the wrong
 * bundle or read the corpus from the stranger. That must fail the run
 * before it starts, not warn mid-run.
 *
 * The CDN port belongs to the run only with a local corpus: without one
 * the content comes from the production CDN and the port is never bound,
 * so a stranger there is harmless.
 */
function checkPorts(facts: PreflightFacts): string[] {
    const errors: string[] = [];
    for (const listener of facts.listeners) {
        if (listener.pid === null || listener.cwd === null) continue;
        if (isInside(listener.cwd, facts.repoRoot)) continue;
        if (listener.port === LOCAL_CDN_PORT && facts.cdnCacheDir === null) continue;

        let reason: string;
        if (listener.port === LOCAL_CDN_PORT) {
            reason =
                "локальный корпус будет раздаваться чужим сервисом — приложение потянет контент у него. " +
                "Задайте E2E_CDN_PORT=<свободный> или остановите процесс";
        } else if (listener.port === APP_PORT) {
            reason =
                "playwright «переиспользует» чужой сервер — снимок уйдёт не из того бандла. Остановите процесс";
        } else {
            reason = "TrailBase не сможет привязать свой порт. Остановите процесс";
        }
        errors.push(`порт ${listener.port} занят процессом из ${listener.cwd} — ${reason}`);
    }
    return errors;
}

/** The upload needs a profile — a missing one must fail before the run. */
function checkUpload(facts: PreflightFacts): string[] {
    if (facts.awsProfilePresent !== false) return [];
    return ["профиль AWS origa-t3 не найден в ~/.aws/credentials — заливка в CDN невозможна"];
}

/** The images must not land in the repository. */
function checkOutput(facts: PreflightFacts): string[] {
    if (!facts.outDirInsideRepo) return [];
    return [
        "каталог вывода внутри репозитория — PNG попадут в историю git (решение #797: скрины в git не коммитим)",
    ];
}

/**
 * The exam needs a local Ollama with the decision model pulled.
 *
 * A missing model or a dead Ollama must fail the run before a pixel is
 * written: a silent skip would look like «no defects found».
 */
function checkExaminer(facts: PreflightFacts): string[] {
    if (!facts.examinerEnabled) return [];
    if (facts.ollamaModels === null) {
        return [
            `Ollama недоступен по ${facts.ollamaUrl} — прогон --examine невозможен (ollama serve)`,
        ];
    }
    if (!facts.ollamaModels.includes(facts.examModelTag)) {
        return [`модель ${facts.examModelTag} не скачана — выполните ollama pull ${facts.examModelTag}`];
    }
    return [];
}

/** Collects the facts and evaluates them. */
export async function runPreflight(options: PreflightOptions): Promise<PreflightReport> {
    const { cdnSource } = options;
    const fonts = inspectFonts(cdnSource.cacheDir, options.distDir);
    const corpusProbe =
        cdnSource.cacheDir === null
            ? null
            : probeCorpus(options.manifestPath, cdnSource.cacheDir);

    const listeners = (
        await Promise.all([APP_PORT, LOCAL_CDN_PORT, TRAILBASE_PORT].map((port) => listListeners(port)))
    ).flat();

    // A dead Ollama is only an error when the run examines; the probe
    // still reports null so the verdicts stay pure and testable.
    const ollamaModels = options.examine
        ? await listOllamaModels(options.ollamaUrl).catch(() => null)
        : null;

    const facts: PreflightFacts = {
        serveMode: options.serveMode,
        nodeModulesPresent: existsSync(join(options.end2endRoot, "node_modules")),
        fontsReady:
            options.serveMode === "build" ? fonts.sourcePresent : fonts.bakedInBundle,
        dictPresent:
            cdnSource.cacheDir !== null &&
            existsSync(join(cdnSource.cacheDir, "dictionaries", "sudachidict-20260723")),
        distPoisoned: options.serveMode === "dist" && isDistPoisoned(options.distDir),
        corpusProbe,
        listeners,
        appBaseUrl: options.appBaseUrl,
        cdnBaseUrl: cdnSource.baseUrl,
        cdnCacheDir: cdnSource.cacheDir,
        repoRoot: options.repoRoot,
        awsProfilePresent: options.upload ? hasAwsProfile("origa-t3") : null,
        trailBaseUrlConfigured: Boolean(process.env.TRAILBASE_URL),
        adminPasswordConfigured: Boolean(process.env.ORIGA_ADMIN_PASSWORD),
        outDirInsideRepo: isInside(options.outDir, options.repoRoot),
        examinerEnabled: options.examine,
        examModelTag: options.examModelTag,
        ollamaUrl: options.ollamaUrl,
        ollamaModels,
    };

    return evaluatePreflight(facts);
}
