import assert from "node:assert/strict";
import { readdir, readFile } from "node:fs/promises";
import { join } from "node:path";
import { test } from "node:test";

import { inventory } from "./coverage-inventory.ts";
import {
  checkStatic,
  compareManifest,
  formatCoverageReport,
  matchesScreen,
  parseRoutePatterns,
  type CoverageRow,
} from "./coverage.ts";
import { listDescriptors } from "./catalog.ts";
import { repoRoot } from "./config.ts";
import type { ShotEntry, ShotManifest } from "./manifest.ts";

async function staticInput() {
  const routes = await readFile(
    join(repoRoot, "origa_ui/src/routes.rs"),
    "utf-8",
  );
  const model = JSON.parse(
    await readFile(join(import.meta.dirname, "examiner/model.json"), "utf-8"),
  ) as { screens: Record<string, unknown> };
  const files = (await readdir(join(import.meta.dirname, "scenarios"))).filter(
    (name) => name.endsWith(".ts") && name !== "index.ts",
  );
  return {
    rows: inventory,
    routePatterns: parseRoutePatterns(routes),
    scenarioIds: listDescriptors().map((descriptor) => descriptor.id),
    scenarioFiles: files,
    examKeys: Object.keys(model.screens),
  };
}

function frame(
  scenario: string,
  screen: string,
  viewport: "desktop" | "mobile",
  commit = "abc123",
): ShotEntry {
  return {
    scenario,
    title: scenario,
    screen,
    route: "/home",
    viewport,
    locale: "ru-RU",
    width: 1280,
    height: 800,
    fullPage: false,
    bytes: 100,
    sha256: "hash",
    file: "/tmp/frame.png",
    capturedAt: "2026-11-26T00:00:00Z",
    commit,
    baseURL: "http://localhost:1420",
    cdnBaseUrl: "http://localhost:8081",
  };
}

test("the router table has every application route", async () => {
  const input = await staticInput();
  assert.equal(input.routePatterns.length, 17);
  assert.ok(input.routePatterns.includes("/login"));
  assert.ok(input.routePatterns.includes("/grammar/:id"));
});

test("the committed inventory satisfies the structural invariants", async () => {
  const issues = checkStatic(await staticInput());
  assert.deepEqual(issues, []);
});

test("every catalog scenario is named by an inventory row", async () => {
  const input = await staticInput();
  const referenced = new Set(
    inventory.map((row) => row.scenario).filter((id): id is string => !!id),
  );
  for (const id of input.scenarioIds) {
    assert.ok(referenced.has(id), `scenario ${id} has no inventory row`);
  }
});

test("a covered row without a mobile frame is red", () => {
  const row = inventory.find((entry) => entry.id === "page:home");
  assert.ok(row);
  const issues = compareManifest([row], [frame("home", "home", "desktop")]);
  assert.deepEqual(issues, [
    { row: "page:home", problem: 'no mobile frame for screen "home"' },
  ]);
});

test("a frame in the other viewport does not satisfy the row", () => {
  const row = inventory.find((entry) => entry.id === "page:home");
  assert.ok(row);
  const issues = compareManifest(
    [row],
    [frame("home", "home", "mobile"), frame("home", "home", "mobile")],
  );
  assert.equal(issues.length, 1);
  assert.match(issues[0].problem, /desktop/);
});

test("dynamic screen names match a wildcard row", () => {
  assert.ok(matchesScreen("review-body-*", "review-body-quiz-3"));
  assert.ok(!matchesScreen("review-body-*", "writing-card"));
});

test("a weak comparator would pass a one-viewport proof — the real one does not", async () => {
  const input = await staticInput();
  const row = input.rows.find((entry) => entry.id === "page:home");
  assert.ok(row);
  const manifest: ShotManifest = {
    version: 1,
    label: "one-viewport",
    entries: [frame("home", "home", "desktop")],
  };
  assert.equal(compareManifest([row], manifest.entries).length, 1);
});

test("mixed commits in one proof are red", () => {
  const row = inventory.find((entry) => entry.id === "page:home");
  assert.ok(row);
  const issues = compareManifest(
    [row],
    [
      frame("home", "home", "desktop", "aaa"),
      frame("home", "home", "mobile", "bbb"),
    ],
  );
  assert.ok(issues.some((issue) => issue.problem.includes("mixed commits")));
});

test("a single viewport without a reason is red", async () => {
  const input = await staticInput();
  const narrow: CoverageRow = {
    id: "test:narrow",
    kind: "variation",
    route: "/home",
    source: "test",
    scenario: "home",
    screens: ["home"],
    viewports: ["mobile"],
    status: "covered",
  };
  const issues = checkStatic({ ...input, rows: [narrow] });
  assert.ok(issues.some((issue) => issue.problem.includes("viewportsReason")));
});

test("the report names every gap instead of hiding it", () => {
  const report = formatCoverageReport(inventory, []);
  const gaps = inventory.filter((row) => row.status === "gap");
  for (const row of gaps) {
    assert.ok(report.includes(row.id), `gap ${row.id} missing from report`);
  }
});
