import assert from "node:assert/strict";
import { test } from "node:test";

import { buildStaticInput } from "./coverage-input.ts";
import { inventory } from "./coverage-inventory.ts";
import {
  checkStatic,
  compareManifest,
  formatCoverageReport,
  matchesScreen,
  type CoverageRow,
  type StaticCheckInput,
} from "./coverage.ts";
import type { ShotEntry } from "./manifest.ts";

async function staticInput(): Promise<StaticCheckInput> {
  return await buildStaticInput();
}

function row(overrides: Partial<CoverageRow> = {}): CoverageRow {
  return {
    id: "test:row",
    kind: "variation",
    route: "/home",
    source: "test-marker",
    scenario: "home",
    screens: ["home"],
    viewports: ["desktop", "mobile"],
    status: "covered",
    ...overrides,
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
  assert.deepEqual(checkStatic(await staticInput()), []);
});

test("every catalog scenario is named by an inventory row", async () => {
  const referenced = new Set(
    inventory.map((item) => item.scenario).filter((id): id is string => !!id),
  );
  for (const id of (await staticInput()).scenarioIds) {
    assert.ok(referenced.has(id), `scenario ${id} has no inventory row`);
  }
});

test("a covered row without a mobile frame is red", () => {
  const issues = compareManifest([row()], [frame("home", "home", "desktop")]);
  assert.deepEqual(issues, [
    { row: "test:row", problem: 'no mobile frame for screen "home"' },
  ]);
});

test("a frame in the other viewport does not satisfy the row", () => {
  const issues = compareManifest(
    [row()],
    [frame("home", "home", "mobile"), frame("home", "home", "mobile")],
  );
  assert.equal(issues.length, 1);
  assert.match(issues[0].problem, /desktop/);
});

test("dynamic screen names match a wildcard with a prefix", () => {
  assert.ok(matchesScreen("review-body-quiz-*", "review-body-quiz-3"));
  assert.ok(!matchesScreen("review-body-*", "writing-card"));
  assert.ok(!matchesScreen("*", "anything"));
});

test("mixed commits in one proof are red", () => {
  const issues = compareManifest(
    [row()],
    [
      frame("home", "home", "desktop", "aaa"),
      frame("home", "home", "mobile", "bbb"),
    ],
  );
  assert.ok(issues.some((issue) => issue.problem.includes("mixed commits")));
});

async function staticWithRows(rows: CoverageRow[]): Promise<StaticCheckInput> {
  return { ...(await staticInput()), rows };
}

test("a duplicate inventory id is red", async () => {
  const issues = await checkStatic(await staticWithRows([row(), row()]));
  assert.ok(issues.some((issue) => issue.problem.includes("duplicate")));
});

test("a route without a row is red", async () => {
  const issues = await checkStatic(
    await staticWithRows([row({ route: "/home" })]),
  );
  assert.ok(issues.some((issue) => issue.row === "/login"));
  assert.ok(
    issues.some((issue) => issue.problem === "route has no inventory row"),
  );
});

test("an alias target without a row is red", async () => {
  const issues = await checkStatic(
    await staticWithRows([row({ route: "/home", aliasOf: "/missing" })]),
  );
  assert.ok(issues.some((issue) => issue.problem.includes("alias target")));
});

test("a scenario without a row is red", async () => {
  const input = await staticInput();
  const rows = input.rows.filter((item) => item.scenario !== "home");
  const issues = checkStatic({ ...input, rows });
  assert.ok(
    issues.some(
      (issue) => issue.problem === "catalog scenario has no inventory row",
    ),
  );
});

test("a covered row without a scenario file is red", async () => {
  const issues = await checkStatic(
    await staticWithRows([row({ scenario: "missing-scenario" })]),
  );
  assert.ok(
    issues.some((issue) => issue.problem.includes("missing scenario file")),
  );
});

test("a covered row without its own examined screen is red", async () => {
  const issues = await checkStatic(
    await staticWithRows([
      row({ scenario: "home", screens: ["not-examined"] }),
    ]),
  );
  assert.ok(
    issues.some((issue) => issue.problem.includes("no examined screen")),
  );
});

test("an exclusion without a reason is red", async () => {
  const issues = await checkStatic(
    await staticWithRows([row({ status: "excluded" })]),
  );
  assert.ok(issues.some((issue) => issue.problem.includes("exclusion needs")));
});

test("tauri-only and feature-off exclusions need a source", async () => {
  for (const code of ["tauri-only", "feature-off"] as const) {
    const issues = await checkStatic(
      await staticWithRows([
        row({
          status: "excluded",
          exclusion: { code, reason: "test" },
        }),
      ]),
    );
    assert.ok(
      issues.some((issue) => issue.problem.includes("needs a flag or file")),
      `${code} without source passed`,
    );
  }
});

test("a row with zero viewports is red", async () => {
  const issues = await checkStatic(
    await staticWithRows([row({ viewports: [] })]),
  );
  assert.ok(
    issues.some((issue) => issue.problem === "row requires no viewport"),
  );
});

test("a single viewport without a reason is red", async () => {
  const issues = await checkStatic(
    await staticWithRows([row({ viewports: ["mobile"] })]),
  );
  assert.ok(issues.some((issue) => issue.problem.includes("viewportsReason")));
});

test("the report names every gap and every issue", () => {
  const gaps = inventory.filter((item) => item.status === "gap");
  const report = formatCoverageReport(inventory, [
    { row: "test:row", problem: "red proof" },
  ]);
  assert.ok(gaps.length > 0, "the inventory must keep unproven rows visible");
  for (const item of gaps) {
    assert.ok(report.includes(item.id), `gap ${item.id} missing from report`);
  }
  assert.match(report, /Проблемы \(1\)/);
  assert.match(report, /red proof/);
});

test("owner-question exclusions get their own report block", () => {
  const item = row({
    id: "test:owner",
    status: "excluded",
    exclusion: { code: "owner-question", reason: "нужен дискриминатор" },
  });
  const report = formatCoverageReport([item], []);
  assert.match(report, /Вопросы владельцу:/);
  assert.match(report, /нужен дискриминатор/);
});
