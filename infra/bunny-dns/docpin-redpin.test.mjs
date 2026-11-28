/**
 * Red-pin gate for the README ↔ handleQuery.js doc-contract pins.
 *
 * Run: node infra/bunny-dns/docpin-redpin.test.mjs
 * Plain node:test + node:assert — no dependencies.
 *
 * The doc-contract pins in smoke.test.mjs are the deliverable of uwuwu #973.
 * A pin that silently stops testing is indistinguishable from a green suite:
 * deleting or weakening a check leaves the suite green. This gate mutates a
 * throwaway copy of the contract and requires the smoke suite to go red for
 * every declared drift. The task worktree is never modified.
 *
 * Lives beside the smoke suite, outside CI on purpose (infra/** is not in
 * ci.yml path filters). Run it together with smoke.test.mjs per the README
 * "Change procedure".
 */

import { strict as assert } from "node:assert";
import { spawnSync } from "node:child_process";
import { mkdtempSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import { after, beforeEach, describe, it } from "node:test";

const sourceDir = dirname(fileURLToPath(import.meta.url));
const FILES = ["README.md", "handleQuery.js", "smoke.test.mjs"];
const baseline = Object.fromEntries(
  FILES.map((file) => [file, readFileSync(join(sourceDir, file), "utf8")]),
);
const fixture = mkdtempSync(join(tmpdir(), "bunny-docpin-"));

after(() => rmSync(fixture, { recursive: true, force: true }));

function writeFixture() {
  for (const file of FILES) {
    writeFileSync(join(fixture, file), baseline[file]);
  }
}

/** Replace a unique anchor; a missing/duplicated anchor is itself a drift signal. */
function mutate(file, from, to) {
  const text = baseline[file];
  assert.equal(
    text.split(from).length - 1,
    1,
    `mutation anchor must appear exactly once in ${file}: ${from}`,
  );
  writeFileSync(join(fixture, file), text.replace(from, to));
}

function runSmoke() {
  // Under `node --test` the runner injects NODE_TEST_CONTEXT into children;
  // the smoke suite would emit binary V8 protocol instead of TAP. Run it as
  // the standalone command the README prescribes, whatever the caller used.
  const env = { ...process.env };
  delete env.NODE_TEST_CONTEXT;
  delete env.NODE_TEST_REPORTER;
  const result = spawnSync(process.execPath, ["smoke.test.mjs"], {
    cwd: fixture,
    encoding: "utf8",
    env,
    timeout: 30_000,
  });
  assert.equal(
    result.error,
    undefined,
    "smoke suite must run, not fail to spawn",
  );
  return { status: result.status, output: `${result.stdout}${result.stderr}` };
}

function assertRed(reasonFragment) {
  const { status, output } = runSmoke();
  assert.notEqual(status, 0, "declared drift must make the smoke suite red");
  assert.match(output, /not ok/, "red must come from a failing test");
  if (reasonFragment) {
    assert.ok(
      output.includes(reasonFragment),
      `suite must fail for the pinned reason ${reasonFragment}; got:\n${output}`,
    );
  }
}

describe("doc-contract pins stay effective (red-pin gate)", () => {
  beforeEach(writeFixture);

  it("green baseline actually runs the doc-contract block", () => {
    const { status, output } = runSmoke();
    assert.equal(status, 0, `baseline suite must be green:\n${output}`);
    assert.match(
      output,
      /README ↔ handleQuery\.js doc-contract pins/,
      "the doc-contract block must still be present in the suite",
    );
  });

  it("README world-target drift fails the target pin", () => {
    mutate(
      "README.md",
      "d3gbi3wo8j4c2w.cloudfront.net",
      "drifted.cloudfront.net",
    );
    assertRed("README s3 section must carry the world CNAME");
  });

  it("README RF-target drift fails the target pin", () => {
    mutate("README.md", "A 193.233.217.243", "A 203.0.113.9");
    assertRed("README s3 section must carry the RF VM IP");
  });

  it("README content-target drift fails the target pin", () => {
    mutate("README.md", "origa.t3.tigrisbucket.io", "drifted.tigrisbucket.io");
    assertRed("README content section must carry the Tigris CNAME");
  });

  it("rewritten guard wording in README fails the guard pin", () => {
    mutate(
      "README.md",
      "answers only the two attached names",
      "answers every name in the zone",
    );
    assertRed();
  });

  it("a third attached section in README fails the exactly-two pin", () => {
    // Insert after content.origa, not before it: the target pin destructures
    // [s3, content] from the documented order and would otherwise fail here
    // too, masking a silent loss of the exactly-two assertion.
    mutate(
      "README.md",
      "\nEverything else in the zone",
      "\n- `new.origa`     → CNAME drifted.example for everyone\n\nEverything else in the zone",
    );
    assertRed();
  });

  it("a script guard narrowed past content.origa fails the attached-names pin", () => {
    mutate(
      "handleQuery.js",
      'hostname !== "content.origa.uwuwu.net"',
      'hostname !== "gone.origa.uwuwu.net"',
    );
    assertRed();
  });

  it("a script guard widened onto a documented plain name fails the guard pin", () => {
    mutate(
      "handleQuery.js",
      '    hostname !== "content.origa.uwuwu.net"\n',
      '    hostname !== "content.origa.uwuwu.net" &&\n    hostname !== "umami.origa.uwuwu.net"\n',
    );
    assertRed();
  });
});
