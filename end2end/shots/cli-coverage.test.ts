import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import { mkdtemp, rm, writeFile } from "node:fs/promises";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { after, test } from "node:test";
import { fileURLToPath } from "node:url";

import type { ShotEntry, ShotManifest } from "./manifest.ts";
import type { ViewportTag } from "./viewports.ts";

// Acceptance layer: drive the real CLI the way a tester does, against
// manifest fixtures. No browser, CDN or TrailBase is needed to prove the
// command's wiring: the static input and the inventory are the committed
// ones, only the proof directory is a fixture.
const end2endRoot = fileURLToPath(new URL("..", import.meta.url));
const cli = join(end2endRoot, "scripts/shots.ts");
const fixtures: string[] = [];

after(async () => {
  for (const dir of fixtures) {
    await rm(dir, { recursive: true, force: true });
  }
});

function entry(
  viewport: ViewportTag,
  commit: string,
  dir: string,
  scenario = "login",
  screen = "login",
): ShotEntry {
  return {
    scenario,
    title: "Публичный экран входа",
    screen,
    route: "/login",
    viewport,
    locale: "ru-RU",
    width: viewport === "desktop" ? 1280 : 390,
    height: viewport === "desktop" ? 800 : 844,
    fullPage: false,
    bytes: 100,
    sha256: "fixture-hash",
    file: join(dir, `${scenario}--${screen}--${viewport}--ru-RU.png`),
    capturedAt: "2026-11-27T00:00:00Z",
    commit,
    baseURL: "http://localhost:1420",
    cdnBaseUrl: "http://localhost:8080",
  };
}

async function proofDir(
  makeEntries: (dir: string) => ShotEntry[],
): Promise<string> {
  const dir = await mkdtemp(join(tmpdir(), "shots-coverage-cli-"));
  fixtures.push(dir);
  const manifest: ShotManifest = {
    version: 1,
    label: "cli-proof",
    entries: makeEntries(dir),
  };
  await writeFile(
    join(dir, "manifest.cli-proof.json"),
    JSON.stringify(manifest),
  );
  return dir;
}

function coverage(args: string[] = []): { code: number; output: string } {
  try {
    const output = execFileSync(process.execPath, [cli, "coverage", ...args], {
      cwd: end2endRoot,
      encoding: "utf-8",
      stdio: ["ignore", "pipe", "pipe"],
    });
    return { code: 0, output };
  } catch (error) {
    const failure = error as {
      status?: number | null;
      stdout?: string;
      stderr?: string;
    };
    assert.equal(
      typeof failure.status,
      "number",
      `CLI did not run: ${failure.stderr ?? String(error)}`,
    );
    return {
      code: failure.status as number,
      output: `${failure.stdout ?? ""}${failure.stderr ?? ""}`,
    };
  }
}

test("CLI proves the login row from a real two-viewport manifest", async () => {
  const dir = await proofDir((dir) => [
    entry("desktop", "commit-a", dir),
    entry("mobile", "commit-a", dir),
  ]);
  const result = coverage(["--run", dir]);
  // The command stays red while the two other declared-covered rows
  // (root/home) have no frames — exactly what the live single-scenario
  // run showed. Only the login row is promoted to a proof.
  assert.equal(result.code, 1);
  assert.match(result.output, /доказано манифестом 1/);
  assert.match(result.output, /covered\s+page:login.*кадры есть/);
  const unproved = [
    ...result.output.matchAll(/^\s{2}(.+?): no (?:desktop|mobile) frame/gm),
  ].map((match) => match[1]);
  assert.ok(unproved.includes("page:root"));
  assert.ok(unproved.includes("page:home"));
  assert.ok(!unproved.includes("page:login"));
});

test("CLI red-proofs a row whose mobile frame is missing", async () => {
  const dir = await proofDir((dir) => [entry("desktop", "commit-a", dir)]);
  const result = coverage(["--run", dir]);
  assert.equal(result.code, 1);
  assert.match(result.output, /page:login: no mobile frame for screen "login"/);
  assert.match(result.output, /доказано манифестом 0/);
});

test("CLI rejects a proof with mixed commits", async () => {
  const dir = await proofDir((dir) => [
    entry("desktop", "commit-a", dir),
    entry("mobile", "commit-b", dir),
  ]);
  const result = coverage(["--run", dir]);
  assert.equal(result.code, 1);
  assert.match(result.output, /mixed commits in one proof/);
  assert.match(result.output, /доказано манифестом 0/);
});

test("CLI structural report does not claim a proof without --run", () => {
  const result = coverage();
  assert.equal(result.code, 0, result.output);
  assert.match(result.output, /доказательство не проверено/);
  assert.doesNotMatch(result.output, /Проблемы \(\d+\)/);
});
