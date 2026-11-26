import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import { mkdtemp, rm, writeFile } from "node:fs/promises";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { after, test } from "node:test";
import { fileURLToPath } from "node:url";

import { inventory } from "./coverage-inventory.ts";
import type { CoverageRow } from "./coverage-types.ts";
import type { ShotEntry, ShotManifest } from "./manifest.ts";
import type { ViewportTag } from "./viewports.ts";

// Acceptance layer: drive the real CLI the way a tester does. The static
// input and the inventory are the committed ones; only the proof directory
// is a fixture. Frames are generated FROM the covered rows, so the suite
// keeps working when #1691 turns more rows covered.
const end2endRoot = fileURLToPath(new URL("..", import.meta.url));
const cli = join(end2endRoot, "scripts/shots.ts");
const commit = "fixture-commit";
const fixtures: string[] = [];

after(async () => {
  for (const dir of fixtures) {
    await rm(dir, { recursive: true, force: true });
  }
});

function coveredRows(): readonly CoverageRow[] {
  return inventory.filter((row) => row.status === "covered");
}

/** A row whose frames are not shared by another covered row. */
function isolatedRow(): CoverageRow {
  const rows = coveredRows();
  const isolated = rows.find(
    (row) =>
      rows.filter(
        (other) =>
          other !== row &&
          other.scenario === row.scenario &&
          other.screens.some((screen) => row.screens.includes(screen)),
      ).length === 0,
  );
  return isolated ?? rows[0];
}

function escapeRegExp(value: string): string {
  return value.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
}

/** The marker on one report line; the id is delimited by the em dash. */
function rowMark(id: string, marker: string): RegExp {
  return new RegExp(`covered\\s+${escapeRegExp(id)}\\s+—[^\\n]*${marker}`);
}

function frame(
  row: CoverageRow,
  screen: string,
  viewport: ViewportTag,
  dir: string,
  frameCommit = commit,
): ShotEntry {
  return {
    scenario: row.scenario as string,
    title: row.source,
    screen,
    route: row.route ?? "/",
    viewport,
    locale: "ru-RU",
    width: viewport === "desktop" ? 1280 : 390,
    height: viewport === "desktop" ? 800 : 844,
    fullPage: false,
    bytes: 100,
    sha256: "fixture-hash",
    file: join(dir, `${row.scenario}--${screen}--${viewport}--ru-RU.png`),
    capturedAt: "2026-11-27T00:00:00Z",
    commit: frameCommit,
    baseURL: "http://localhost:1420",
    cdnBaseUrl: "http://localhost:8080",
  };
}

interface ProofOptions {
  readonly skip?: (row: CoverageRow, viewport: ViewportTag) => boolean;
  readonly frameCommit?: (row: CoverageRow, viewport: ViewportTag) => string;
}

/** Writes the shape `capture` leaves behind: one manifest per scenario. */
async function proofDir(options: ProofOptions = {}): Promise<string> {
  const dir = await mkdtemp(join(tmpdir(), "shots-coverage-cli-"));
  fixtures.push(dir);
  const byScenario = new Map<string, ShotEntry[]>();
  for (const row of coveredRows()) {
    assert.ok(row.scenario, `covered row ${row.id} has no scenario`);
    for (const screen of row.screens) {
      // A wildcard row is satisfied by one concrete frame with its prefix.
      const concrete = screen.replace(/\*$/, "sample");
      for (const viewport of row.viewports) {
        if (options.skip?.(row, viewport)) continue;
        const entry = frame(
          row,
          concrete,
          viewport,
          dir,
          options.frameCommit?.(row, viewport) ?? commit,
        );
        byScenario.set(row.scenario, [
          ...(byScenario.get(row.scenario) ?? []),
          entry,
        ]);
      }
    }
  }
  for (const [scenario, entries] of byScenario) {
    const manifest: ShotManifest = {
      version: 1,
      label: "cli-proof",
      entries,
    };
    await writeFile(
      join(dir, `manifest.${scenario}.json`),
      JSON.stringify(manifest),
    );
  }
  return dir;
}

interface CommandResult {
  readonly code: number;
  readonly stdout: string;
  readonly stderr: string;
}

function runCli(args: string[]): CommandResult {
  try {
    const stdout = execFileSync(process.execPath, [cli, ...args], {
      cwd: end2endRoot,
      encoding: "utf-8",
      timeout: 30_000,
      stdio: ["ignore", "pipe", "pipe"],
    });
    return { code: 0, stdout, stderr: "" };
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
      stdout: failure.stdout ?? "",
      stderr: failure.stderr ?? "",
    };
  }
}

function coverage(args: string[] = []): CommandResult {
  return runCli(["coverage", ...args]);
}

test("CLI exits green when every covered row has frames in both viewports", async () => {
  const rows = coveredRows();
  assert.ok(rows.length > 0, "the committed inventory has no covered rows");
  const dir = await proofDir();
  const result = coverage(["--run", dir]);
  assert.equal(result.code, 0, `${result.stdout}${result.stderr}`);
  assert.match(result.stdout, new RegExp(`доказано манифестом ${rows.length}`));
  for (const row of rows) {
    assert.match(result.stdout, rowMark(row.id, "кадры есть"));
  }
  assert.match(result.stdout, /Структурных проблем нет/);
  assert.doesNotMatch(result.stdout, /Проблемы \(\d+\)/);

  const inline = coverage([`--run=${dir}`]);
  assert.equal(inline.code, 0, `${inline.stdout}${inline.stderr}`);
  assert.equal(inline.stdout, result.stdout);
});

test("CLI marks the unproved row as having no frames in each viewport", async () => {
  const rows = coveredRows();
  const target = isolatedRow();
  for (const missing of target.viewports) {
    const dir = await proofDir({
      skip: (row, viewport) => row.id === target.id && viewport === missing,
    });
    const result = coverage(["--run", dir]);
    assert.equal(result.code, 1, result.stdout);
    assert.match(
      result.stdout,
      new RegExp(`${escapeRegExp(target.id)}: no ${missing} frame`),
    );
    assert.match(
      result.stdout,
      new RegExp(`доказано манифестом ${rows.length - 1}`),
    );
    assert.match(result.stdout, rowMark(target.id, "кадров нет"));
    for (const other of rows) {
      if (other.id !== target.id) {
        assert.match(result.stdout, rowMark(other.id, "кадры есть"));
      }
    }
  }
});

test("CLI rejects a proof whose manifests carry different commits", async () => {
  const target = isolatedRow();
  const dir = await proofDir({
    frameCommit: (row, viewport) =>
      row.id === target.id && viewport === target.viewports[0]
        ? "other-commit"
        : commit,
  });
  const result = coverage(["--run", dir]);
  assert.equal(result.code, 1, result.stdout);
  assert.match(result.stdout, /mixed commits in one proof/);
  assert.match(result.stdout, /доказано манифестом 0/);
});

test("CLI structural report claims no proof without --run", () => {
  const result = coverage();
  assert.equal(result.code, 0, `${result.stdout}${result.stderr}`);
  assert.match(result.stdout, /доказано манифестом 0/);
  assert.match(result.stdout, /доказательство не проверено/);
  assert.doesNotMatch(result.stdout, /Проблемы \(\d+\)/);
});

test("CLI rejects invalid arguments before a browser starts", async () => {
  const missingDir = join(await proofDir(), "absent");
  const cases: readonly [string[], RegExp][] = [
    [
      [
        "capture",
        "--scenario",
        "unknown-screen",
        "--label",
        "t-neg",
        "--no-registry",
      ],
      /Неизвестные сценарии/,
    ],
    [
      [
        "capture",
        "--scenario",
        "login",
        "--label",
        "bad/label",
        "--no-registry",
      ],
      /Invalid label/,
    ],
    [
      [
        "capture",
        "--scenario",
        "login",
        "--viewports",
        "tablet",
        "--label",
        "t-neg",
      ],
      /Unknown viewport/,
    ],
    [["coverage", "--run", missingDir], /ENOENT/],
  ];
  for (const [args, expected] of cases) {
    const result = runCli(args);
    assert.equal(result.code, 1, `${args.join(" ")}: ${result.stderr}`);
    assert.match(`${result.stdout}${result.stderr}`, expected);
  }
});
