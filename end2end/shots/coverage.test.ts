import assert from "node:assert/strict";
import { test } from "node:test";
import { crc32 } from "node:zlib";

import { buildStaticInput } from "./coverage-input.ts";
import { inventory } from "./coverage-inventory.ts";
import {
  checkStatic,
  compareManifest,
  formatCoverageReport,
  matchesScreen,
  provedRows,
  type CoverageRow,
  type ProofContext,
  type StaticCheckInput,
} from "./coverage.ts";
import {
  sha256Hex,
  stampProof,
  type ShotEntry,
  type ShotProof,
} from "./manifest.ts";

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

/** A structurally valid PNG with no pixels: signature, IHDR, IEND. */
function minimalPng(): Buffer {
  const chunk = (type: string, data: Buffer): Buffer => {
    const result = Buffer.alloc(12 + data.length);
    result.writeUInt32BE(data.length, 0);
    result.write(type, 4, "latin1");
    data.copy(result, 8);
    result.writeUInt32BE(
      crc32(result.subarray(4, 8 + data.length)),
      8 + data.length,
    );
    return result;
  };
  return Buffer.concat([
    Buffer.from([137, 80, 78, 71, 13, 10, 26, 10]),
    chunk("IHDR", Buffer.alloc(13)),
    chunk("IEND", Buffer.alloc(0)),
  ]);
}

function stampFor(entry: ShotEntry, override?: Partial<ShotProof>): Buffer {
  return stampProof(minimalPng(), {
    schema: 1,
    commit: override?.commit ?? entry.commit,
    screen: override?.screen ?? entry.screen,
  });
}

function frame(
  scenario: string,
  screen: string,
  viewport: "desktop" | "mobile",
  commit = "abc123",
): ShotEntry {
  const file = `/tmp/frame-${scenario}-${screen}-${viewport}-${commit}.png`;
  const png = stampProof(minimalPng(), { schema: 1, commit, screen });
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
    bytes: png.byteLength,
    sha256: sha256Hex(png),
    file,
    capturedAt: "2026-11-26T00:00:00Z",
    commit,
    baseURL: "http://localhost:1420",
    cdnBaseUrl: "http://localhost:8081",
  };
}

function ctxFor(
  entries: readonly ShotEntry[],
  options: {
    expected?: string;
    ancestor?: boolean;
    stamp?: Partial<ShotProof>;
    raw?: Map<string, Buffer>;
  } = {},
): ProofContext {
  return {
    expectedCommit: options.expected ?? "abc123",
    isAncestor: () => options.ancestor ?? true,
    proofRoot: "/tmp",
    readPng: async (file: string) => {
      const raw = options.raw?.get(file);
      if (raw) return raw;
      const entry = entries.find((item) => item.file === file);
      if (!entry) throw new Error(`no frame: ${file}`);
      return stampFor(entry, options.stamp);
    },
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

test("a covered row without a mobile frame is red", async () => {
  const frames = [frame("home", "home", "desktop")];
  const issues = await compareManifest([row()], frames, ctxFor(frames));
  assert.deepEqual(issues, [
    { row: "test:row", problem: 'no mobile frame for screen "home"' },
  ]);
});

test("a frame in the other viewport does not satisfy the row", async () => {
  const frames = [
    frame("home", "home", "mobile"),
    frame("home", "home", "mobile"),
  ];
  const issues = await compareManifest([row()], frames, ctxFor(frames));
  assert.equal(issues.length, 1);
  assert.match(issues[0].problem, /desktop/);
});

test("dynamic screen names match a wildcard with a prefix", () => {
  assert.ok(matchesScreen("review-body-quiz-*", "review-body-quiz-3"));
  assert.ok(!matchesScreen("review-body-*", "writing-card"));
  assert.ok(!matchesScreen("*", "anything"));
});

test("a covered row without a scenario is red", async () => {
  const issues = await checkStatic(
    await staticWithRows([row({ scenario: null })]),
  );
  assert.ok(
    issues.some((issue) => issue.problem === "covered row has no scenario"),
  );
});

test("a wildcard without a prefix is red", async () => {
  const issues = await checkStatic(
    await staticWithRows([row({ screens: ["*"] })]),
  );
  assert.ok(
    issues.some((issue) => issue.problem === "screen wildcard needs a prefix"),
  );
});

test("a gap row cannot name a scenario outside the catalog", async () => {
  const issues = await checkStatic(
    await staticWithRows([row({ status: "gap", scenario: "not-in-catalog" })]),
  );
  assert.ok(
    issues.some((issue) => issue.problem.includes("not in the catalog")),
  );
});

test("mixed commits invalidate the whole proof", async () => {
  const frames = [
    frame("home", "home", "desktop", "aaa"),
    frame("home", "home", "mobile", "bbb"),
  ];
  const issues = await compareManifest(
    [row()],
    frames,
    ctxFor(frames, { expected: "bbb" }),
  );
  assert.deepEqual(provedRows([row()], issues), []);
});

test("mixed commits in one proof are red", async () => {
  const frames = [
    frame("home", "home", "desktop", "aaa"),
    frame("home", "home", "mobile", "bbb"),
  ];
  const issues = await compareManifest(
    [row()],
    frames,
    ctxFor(frames, { expected: "bbb" }),
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
  const input = await staticInput();
  const issues = checkStatic({
    ...input,
    rows: [row({ scenario: "home" })],
    scenarioFiles: input.scenarioFiles.filter((name) => name !== "home.ts"),
  });
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

test("a frame with no proof chunk is red even with a matching sha256", async () => {
  const entry = frame("home", "home", "desktop");
  const raw = minimalPng();
  const unstamped = { ...entry, sha256: sha256Hex(raw) };
  const issues = await compareManifest(
    [row({ viewports: ["desktop"] })],
    [unstamped],
    ctxFor([unstamped], { raw: new Map([[unstamped.file, raw]]) }),
  );
  assert.ok(issues.some((issue) => /without proof/.test(issue.problem)));
});

test("a stamp from a non-ancestor commit is red", async () => {
  const frames = [frame("home", "home", "desktop", "old")];
  const issues = await compareManifest(
    [row({ viewports: ["desktop"] })],
    frames,
    ctxFor(frames, { expected: "head", ancestor: false }),
  );
  assert.ok(issues.some((issue) => /not an ancestor/.test(issue.problem)));
  assert.deepEqual(provedRows([row()], issues), []);
});

test("an ancestor commit other than HEAD proves the row", async () => {
  const frames = [frame("home", "home", "desktop", "older-commit")];
  const issues = await compareManifest(
    [row({ viewports: ["desktop"] })],
    frames,
    ctxFor(frames, { expected: "head-commit" }),
  );
  assert.deepEqual(issues, []);
});

test("a stamp naming another screen is red", async () => {
  const entry = frame("home", "home", "desktop");
  const png = stampFor(entry, { screen: "other-screen" });
  const mismatched = { ...entry, sha256: sha256Hex(png) };
  const issues = await compareManifest(
    [row({ viewports: ["desktop"] })],
    [mismatched],
    ctxFor([mismatched], { raw: new Map([[mismatched.file, png]]) }),
  );
  assert.ok(
    issues.some((issue) => /screen stamp mismatch/.test(issue.problem)),
  );
});

test("a negative proof is a row issue, not a silent gap", async () => {
  const failed = { ...frame("home", "home", "desktop"), file: "", sha256: "" };
  const issues = await compareManifest(
    [row({ viewports: ["desktop"] })],
    [failed],
    ctxFor([]),
  );
  assert.ok(issues.some((issue) => /negative proof/.test(issue.problem)));
});

test("a frame path outside --run is red", async () => {
  const entry = { ...frame("home", "home", "desktop"), file: "/etc/passwd" };
  const issues = await compareManifest([row()], [entry], ctxFor([entry]));
  assert.ok(issues.some((issue) => /outside --run/.test(issue.problem)));
});

test("editing only the manifest commit field does not change the verdict", async () => {
  const entry = { ...frame("home", "home", "desktop"), commit: "forged" };
  const issues = await compareManifest(
    [row({ viewports: ["desktop"] })],
    [entry],
    ctxFor([entry], { stamp: { commit: "abc123" } }),
  );
  assert.deepEqual(issues, []);
});
