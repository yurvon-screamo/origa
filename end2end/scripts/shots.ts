#!/usr/bin/env node
/**
 * The UI screenshot tool.
 *
 * One command captures the screens of a scenario, keeps the provenance and
 * (optionally) publishes the files to the CDN. Agents used to write a
 * bespoke Playwright script per pull request; this is the shared
 * replacement.
 *
 *   npm run shots -- list
 *   npm run shots -- capture --scenario home --label issue-808 [--upload]
 *
 * The CLI itself is a thin shell: it checks the environment, then hands
 * the work to the Playwright driver (`screenshots-spec/capture.spec.ts`),
 * which owns the browser session.
 */
import { execFile, execFileSync, spawn } from "node:child_process";
import { promisify } from "node:util";
import { mkdir, readdir, readFile, writeFile } from "node:fs/promises";
import { join } from "node:path";
import { fileURLToPath } from "node:url";
import { config as loadDotenv } from "dotenv";

import { listDescriptors } from "../shots/catalog.ts";
import { SERVE_MODE, resolveCdnSource } from "../shots/config.ts";
import { renderIndex } from "../shots/index.ts";
import type { ShotEntry, ShotManifest } from "../shots/manifest.ts";
import { runPreflight } from "../shots/preflight.ts";
import { mergeEntry, readRegistry, registryDir, type RegistryShot, type UploadInfo } from "../shots/registry.ts";
import { DEFAULT_VIEWPORTS, VIEWPORTS, type ViewportTag } from "../shots/viewports.ts";

const run = promisify(execFile);

const end2endRoot = fileURLToPath(new URL("..", import.meta.url));
const repoRoot = fileURLToPath(new URL("../..", import.meta.url));

loadDotenv({ path: join(end2endRoot, ".env"), override: true, quiet: true });

const CDN_HOST = "https://s3.origa.uwuwu.net";
const CDN_BUCKET = "origa";
const CDN_ENDPOINT = "https://t3.storage.dev";
const CDN_PROFILE = "origa-t3";
const DEFAULT_APP_BASE_URL = "http://localhost:1420";

interface Options {
    scenario: string[];
    viewports: ViewportTag[];
    locale?: string;
    out: string;
    label: string;
    upload: boolean;
    baseUrl?: string;
    /** Skip the registry update (a throwaway capture). */
    registry: boolean;
}

/** Parses `--key value`, `--key=value` and boolean flags. */
function parseOptions(argv: string[]): Options {
    // A flag may be repeated: `--scenario a --scenario b` means both.
    const values = new Map<string, string[]>();
    const flags = new Set<string>();
    for (let index = 0; index < argv.length; index++) {
        const argument = argv[index];
        if (!argument.startsWith("--")) continue;
        const [key, inline] = argument.slice(2).split("=");
        const value = inline ?? (argv[index + 1]?.startsWith("--") ? undefined : argv[++index]);
        if (value === undefined) {
            flags.add(key);
            continue;
        }
        values.set(key, [...(values.get(key) ?? []), value]);
    }

    const last = (key: string): string | undefined => values.get(key)?.at(-1);
    const all = (key: string): string[] => values.get(key) ?? [];

    const scenario = all("scenario")
        .flatMap((value) => value.split(","))
        .map((value) => value.trim())
        .filter(Boolean);
    const requestedViewports = all("viewports")
        .flatMap((value) => value.split(","))
        .map((value) => value.trim())
        .filter(Boolean) as ViewportTag[];
    for (const viewport of requestedViewports) {
        if (!(viewport in VIEWPORTS)) {
            throw new Error(`Unknown viewport "${viewport}". Known: ${Object.keys(VIEWPORTS).join(", ")}`);
        }
    }
    const label = last("label") ?? `shots-${new Date().toISOString().replace(/[:.]/g, "-")}`;
    if (!/^[A-Za-z0-9._-]+$/.test(label)) {
        // The label becomes a directory name and a part of the S3 key.
        throw new Error(
            `Invalid label "${label}" — only letters, digits, dot, underscore and dash are allowed`,
        );
    }

    return {
        scenario,
        viewports: requestedViewports.length > 0 ? requestedViewports : DEFAULT_VIEWPORTS,
        locale: last("locale"),
        out: last("out") ?? join("/tmp/opencode/shots", label),
        label,
        upload: flags.has("upload"),
        baseUrl: last("base-url"),
        registry: !flags.has("no-registry"),
    };
}

function headCommit(): string {
    try {
        return execFileSync("git", ["rev-parse", "HEAD"], { cwd: repoRoot, encoding: "utf-8" }).trim();
    } catch {
        return "unknown";
    }
}

function listCommand(): number {
    console.log("Сценарии:");
    for (const descriptor of listDescriptors()) {
        const viewports = (descriptor.viewports ?? DEFAULT_VIEWPORTS).join(", ");
        console.log(`  ${descriptor.id.padEnd(20)} ${descriptor.title} — ${descriptor.route} [${viewports}]`);
    }
    return 0;
}

async function captureCommand(options: Options): Promise<number> {
    const known = new Set(listDescriptors().map((descriptor) => descriptor.id));
    const unknown = options.scenario.filter((id) => !known.has(id));
    if (options.scenario.length === 0) {
        console.error("Укажите сценарий: --scenario <id>[,<id>]");
        return 1;
    }
    if (unknown.length > 0) {
        console.error(`Неизвестные сценарии: ${unknown.join(", ")}. Известные: ${[...known].join(", ")}`);
        return 1;
    }

    const cdnSource = resolveCdnSource();
    const appBaseUrl = options.baseUrl ?? process.env.E2E_APP_BASE_URL ?? DEFAULT_APP_BASE_URL;
    const report = await runPreflight({
        serveMode: SERVE_MODE,
        end2endRoot,
        repoRoot,
        distDir: join(repoRoot, "origa_ui", "dist"),
        manifestPath: join(end2endRoot, "cdn-manifest.txt"),
        appBaseUrl,
        cdnSource,
        outDir: options.out,
        upload: options.upload,
    });

    for (const line of report.info) console.log(`  ${line}`);
    for (const line of report.warnings) console.warn(`  ! ${line}`);
    if (report.errors.length > 0) {
        for (const line of report.errors) console.error(`  ✗ ${line}`);
        return 1;
    }

    await mkdir(options.out, { recursive: true });
    const exitCode = await runDriver(options, appBaseUrl, cdnSource.baseUrl);
    if (exitCode !== 0) return exitCode;

    const entries = await readManifestEntries(options.out);
    if (entries.length === 0) {
        console.error("Снимки не получены — манифест пуст");
        return 1;
    }

    const uploads = new Map<string, UploadInfo>();
    if (options.upload) {
        for (const entry of entries) {
            const info = await uploadEntry(entry, options.label);
            uploads.set(entry.file, info);
        }
    }

    if (options.registry) {
        const directory = registryDir(repoRoot);
        for (const entry of entries) {
            await mergeEntry(directory, entry, uploads.get(entry.file));
        }
        await writeIndex(directory, await readRegistry(directory));
        console.log(`\nРеестр обновлён: ${join(repoRoot, "docs", "ui-screenshots")}`);
    }

    console.log(`\nСнимки (${entries.length}):`);
    for (const entry of entries) {
        console.log(`  ${entry.scenario}/${entry.screen} [${entry.viewport}] ${entry.width}×${entry.height} ${entry.sha256.slice(0, 8)}`);
    }
    if (uploads.size === entries.length && entries.length > 0) {
        console.log(`\n${renderMarkdown(entries, uploads)}`);
    } else {
        // Without an upload there is no link to paste into a pull request —
        // a markdown image with a local path would break there.
        console.log("\nЛокальные файлы (без --upload вставить в PR нельзя):");
        for (const entry of entries) {
            console.log(`  ${entry.file}`);
        }
    }
    return 0;
}

/** Regenerates the human-readable index from the registry. */
async function indexCommand(): Promise<number> {
    const directory = registryDir(repoRoot);
    const entries = await readRegistry(directory);
    await writeIndex(directory, entries);
    console.log(`Индекс обновлён: ${entries.length} записей`);
    return 0;
}

/** Regenerates the human-readable index from the registry. */
async function writeIndex(directory: string, entries: RegistryShot[]): Promise<void> {
    const target = join(repoRoot, "docs", "ui-screenshots");
    await mkdir(target, { recursive: true });
    await writeFile(join(target, "README.md"), renderIndex(entries), "utf-8");
}

/** Runs the Playwright driver that owns the browser session. */
async function runDriver(options: Options, appBaseUrl: string, cdnBaseUrl: string): Promise<number> {
    const binary = join(
        end2endRoot,
        "node_modules",
        ".bin",
        process.platform === "win32" ? "playwright.cmd" : "playwright",
    );
    const child = spawn(
        binary,
        ["test", "--config=playwright.screenshots.config.ts", "capture.spec.ts"],
        {
            cwd: end2endRoot,
            stdio: "inherit",
            env: {
                ...process.env,
                SHOT_SCENARIOS: options.scenario.join(","),
                SHOT_OUT_DIR: options.out,
                SHOT_LABEL: options.label,
                SHOT_VIEWPORTS: options.viewports.join(","),
                SHOT_LOCALE: options.locale ?? "",
                SHOT_CDN_BASE_URL: cdnBaseUrl,
                SHOT_COMMIT: headCommit(),
                E2E_APP_BASE_URL: appBaseUrl,
            },
        },
    );
    return await new Promise<number>((resolve) => {
        child.on("close", (code) => resolve(code ?? 1));
    });
}

/** Merges the per-scenario manifests the driver wrote. */
async function readManifestEntries(outDir: string): Promise<ShotEntry[]> {
    const files = (await readdir(outDir)).filter((name) => name.startsWith("manifest.") && name.endsWith(".json"));
    const entries: ShotEntry[] = [];
    for (const file of files) {
        const manifest = JSON.parse(await readFile(join(outDir, file), "utf-8")) as ShotManifest;
        entries.push(...manifest.entries);
    }
    return entries;
}

/**
 * Publishes a capture under a content-addressed key.
 *
 * The key carries the hash of the file: the same content keeps the same
 * URL, a changed capture gets a new one — an edge cache can never serve
 * a picture of an earlier build.
 */
async function uploadEntry(entry: ShotEntry, label: string): Promise<UploadInfo> {
    const key = `pr-screenshots/${label}/${entry.scenario}--${entry.screen}--${entry.viewport}-${entry.sha256.slice(0, 8)}.png`;
    await run(
        "aws",
        [
            "s3",
            "cp",
            entry.file,
            `s3://${CDN_BUCKET}/${key}`,
            "--profile",
            CDN_PROFILE,
            "--endpoint-url",
            CDN_ENDPOINT,
            "--cache-control",
            "public,max-age=31536000,immutable",
            "--content-type",
            "image/png",
        ],
        { timeout: 120_000 },
    );
    return {
        url: `${CDN_HOST}/${key}`,
        sha256: entry.sha256,
        uploadedAt: new Date().toISOString(),
    };
}

/** The markdown block for a pull request body. */
function renderMarkdown(entries: ShotEntry[], uploads: Map<string, UploadInfo>): string {
    return entries
        .map((entry) => {
            const label = `${entry.scenario}/${entry.screen} (${entry.viewport})`;
            const target = uploads.get(entry.file)?.url ?? entry.file;
            return `![${label}](${target})`;
        })
        .join("\n");
}

function usage(): number {
    console.log(`Инструмент UI-скриншотов.

  npm run shots -- list
  npm run shots -- capture --scenario <id>[,<id>] [options]
  npm run shots -- index

Опции capture:
  --scenario <id>      сценарий из каталога (обязательно)
  --viewports <list>   desktop,mobile (по умолчанию оба)
  --locale <locale>   ru-RU или en-US (по умолчанию локаль сценария)
  --out <dir>          каталог файлов (по умолчанию /tmp/opencode/shots/<label>)
  --label <label>      метка запуска, попадает в имя объекта CDN
  --base-url <url>     другой бандл (сборка before/after на другом порту)
  --upload             залить в CDN и напечатать ссылки
  --no-registry        не обновлять реестр (разовый прогон)

Переменные окружения сценариев: SHOT_WORD — слово для «word-examples» (あさ по умолчанию),
SHOT_BASE_URL — другая сборка (before/after на другом порту).`);
    return 1;
}

async function main(): Promise<number> {
    const [command, ...rest] = process.argv.slice(2);
    try {
        switch (command) {
            case "list":
                return listCommand();
            case "capture":
                return await captureCommand(parseOptions(rest));
            case "index":
                return await indexCommand();
            default:
                return usage();
        }
    } catch (error) {
        console.error(error instanceof Error ? error.message : String(error));
        return 1;
    }
}

process.exitCode = await main();
