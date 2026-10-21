import { existsSync } from "node:fs";
import { join } from "node:path";

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
    outDirInsideRepo: boolean;
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
    warnings.push(...checkPorts(facts));
    errors.push(...checkUpload(facts));
    errors.push(...checkOutput(facts));

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
    if (facts.nodeModulesPresent) return [];
    return ["end2end/node_modules отсутствует — выполните `npm ci` в end2end/"];
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
                    "Укажите каталог корпуса через ORIGA_CDN_CACHE_DIR (симлинк или копия dictionaries/ + fonts/)",
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

/** The servers of the run: a listener from another checkout is a zombie. */
function checkPorts(facts: PreflightFacts): string[] {
    const warnings: string[] = [];
    for (const listener of facts.listeners) {
        if (listener.pid === null || listener.cwd === null) continue;
        if (!isInside(listener.cwd, facts.repoRoot)) {
            warnings.push(
                `порт ${listener.port} занят процессом из ${listener.cwd} — возможен зомби-ворктри, снимок уйдёт не из того бандла`,
            );
        }
    }
    return warnings;
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
        outDirInsideRepo: isInside(options.outDir, options.repoRoot),
    };

    return evaluatePreflight(facts);
}
