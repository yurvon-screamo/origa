import { execFile } from "node:child_process";
import { existsSync, readFileSync, readdirSync, statSync } from "node:fs";
import { homedir } from "node:os";
import { isAbsolute, join, relative as relativePath } from "node:path";
import { promisify } from "node:util";

const run = promisify(execFile);

/** A process listening on a port. */
export interface ListenerInfo {
    port: number;
    pid: number | null;
    /** Working directory of the process — the zombie-worktree signal. */
    cwd: string | null;
}

/** Corpus object probed to detect a stale local corpus. */
export interface CorpusProbe {
    path: string;
    present: boolean;
}

/** Whether the local corpus carries the branded fonts. */
export interface FontState {
    /** `cdn/fonts/` holds the woff2 subset (build mode). */
    sourcePresent: boolean;
    /** The served bundle injects `@font-face` for Cormorant (dist mode). */
    bakedInBundle: boolean;
}

/**
 * Lists the processes listening on a TCP port.
 *
 * `ss` is Linux-only and does not report the working directory — that
 * comes from `/proc/<pid>/cwd`. On platforms without either tool the
 * probe degrades to "no information" instead of failing the capture.
 */
export async function listListeners(port: number): Promise<ListenerInfo[]> {
    const linux = await readLinuxListeners(port);
    if (linux !== null) return linux;
    return await readDarwinListeners(port);
}

async function readLinuxListeners(port: number): Promise<ListenerInfo[] | null> {
    let stdout: string;
    try {
        ({ stdout } = await run("ss", ["-tlnpH", `sport = :${port}`], { timeout: 5_000 }));
    } catch {
        return null;
    }
    const listeners: ListenerInfo[] = [];
    for (const line of stdout.split("\n")) {
        if (!line.includes(`:${port}`)) continue;
        const pidMatch = /pid=(\d+)/.exec(line);
        const pid = pidMatch ? Number(pidMatch[1]) : null;
        listeners.push({ port, pid, cwd: pid === null ? null : readProcCwd(pid) });
    }
    return listeners;
}

async function readDarwinListeners(port: number): Promise<ListenerInfo[]> {
    try {
        const { stdout } = await run("lsof", ["-nP", `-iTCP:${port}`, "-sTCP:LISTEN", "-Fpc"], {
            timeout: 5_000,
        });
        const listeners: ListenerInfo[] = [];
        let pid: number | null = null;
        for (const line of stdout.split("\n")) {
            if (line.startsWith("p")) {
                pid = Number(line.slice(1).trim());
            } else if (line.startsWith("c") && pid !== null) {
                listeners.push({ port, pid, cwd: null });
                pid = null;
            }
        }
        return listeners;
    } catch {
        return [];
    }
}

/** Working directory of a process (Linux only). */
export function readProcCwd(pid: number): string | null {
    try {
        return readFileSync(`/proc/${pid}/cwd`, "utf-8").trim() || null;
    } catch {
        return null;
    }
}

/**
 * Picks a small object of the CDN manifest and checks whether the local
 * corpus still carries it.
 *
 * The manifest is the repo's inventory of the corpus (1209 paths), so the
 * probe is reproducible. A missing object means the local corpus lags
 * behind the deployment — a warning, not a failure: the capture still
 * renders, it just shows older content.
 */
export function probeCorpus(manifestPath: string, cacheDir: string): CorpusProbe | null {
    if (!existsSync(manifestPath)) return null;
    const entries = readFileSync(manifestPath, "utf-8").split("\n").filter(Boolean);
    for (const entry of entries) {
        const file = join(cacheDir, entry);
        try {
            const stats = statSync(file);
            if (stats.isFile() && stats.size < 1_000_000) {
                return { path: entry, present: true };
            }
        } catch {
            return { path: entry, present: false };
        }
    }
    return null;
}

/** Font availability of the local corpus and of the built bundle. */
export function inspectFonts(cacheDir: string | null, distDir: string): FontState {
    const sourcePresent = cacheDir !== null && existsSync(join(cacheDir, "fonts"));
    return { sourcePresent, bakedInBundle: bundleHasBrandedFonts(distDir) };
}

/** Whether the built bundle injects `@font-face` for Cormorant and ships the woff2 files. */
function bundleHasBrandedFonts(distDir: string): boolean {
    try {
        const fontFiles = existsSync(join(distDir, "public", "fonts"))
            ? readdirSync(join(distDir, "public", "fonts")).filter((name) => name.endsWith(".woff2"))
            : [];
        if (fontFiles.length === 0) return false;
        return readdirSync(distDir)
            .filter((name) => name.endsWith(".css"))
            .some((name) => {
                const css = readFileSync(join(distDir, name), "utf-8");
                return css.includes("@font-face") && css.includes("Cormorant");
            });
    } catch {
        return false;
    }
}

/**
 * Whether the built bundle still carries the trunk live-reload bridge.
 *
 * `trunk serve` mutates `dist/index.html`; serving that file statically
 * breaks the WASM load mid-capture.
 */
export function isDistPoisoned(distDir: string): boolean {
    try {
        return readFileSync(join(distDir, "index.html"), "utf-8").includes(".well-known/trunk/ws");
    } catch {
        return false;
    }
}

/** Whether the AWS profile the upload needs exists in the credentials file. */
export function hasAwsProfile(profile: string): boolean {
    try {
        const credentials = readFileSync(join(homedir(), ".aws", "credentials"), "utf-8");
        return credentials.split("\n").some((line) => line.trim() === `[${profile}]`);
    } catch {
        return false;
    }
}

/** Whether a path lies inside another path (the same path counts as inside). */
export function isInside(inner: string, outer: string): boolean {
    // path.relative works with the separator of the platform — a string
    // comparison would miss the Windows backslash.
    const relative = relativePath(outer, inner);
    return !relative.startsWith("..") && !isAbsolute(relative);
}
