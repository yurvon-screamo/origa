import assert from "node:assert/strict";
import { mkdirSync, mkdtempSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { test } from "node:test";

import { isInside, readProcCwd, resolveBuildFontsDir } from "./probe.ts";

test("a directory is inside itself", () => {
  assert.equal(isInside("/repo", "/repo"), true);
});

test("a subdirectory is inside", () => {
  assert.equal(isInside("/repo/end2end/shots", "/repo"), true);
});

test("a parent directory is not inside", () => {
  assert.equal(isInside("/repo", "/repo/end2end"), false);
});

test("a sibling directory is not inside", () => {
  assert.equal(isInside("/other/worktree", "/repo"), false);
});

// Regression pin (task 20261109020526-1hzlugy): the cwd probe used
// readFileSync on /proc/<pid>/cwd — a symlink to a directory — which
// always failed with EISDIR and silently returned null, so no port
// verdict ever fired on Linux. The readlink is the behavior under test.
test(
  "the cwd probe reads the live /proc symlink",
  { skip: process.platform !== "linux" },
  () => {
    assert.equal(readProcCwd(process.pid), process.cwd());
  },
);

test("a cwd probe of a non-existent pid degrades to unknown ownership", () => {
  assert.equal(readProcCwd(-1), null);
});

// Resolution matrix of resolveBuildFontsDir — the mirror of build.rs
// cdn_dir (task 20261110041750-4bs23o3): the check and the build must look
// at the same place, otherwise preflight passes while the build bakes a
// sans-serif bundle. The build follows the corpus canon only — the shot
// serving chain (ORIGA_CDN_CACHE_DIR first) is deliberately not consulted
// (round-5 review seam).
test("the ORIGA_CDN_DIR corpus with fonts wins", () => {
  const root = mkdtempSync(join(tmpdir(), "origa-probe-fonts-"));
  try {
    const corpus = join(root, "corpus");
    mkdirSync(join(corpus, "fonts"), { recursive: true });
    writeFileSync(join(corpus, "fonts", "a-12345678.woff2"), "");
    assert.equal(
      resolveBuildFontsDir(corpus, join(root, "home")),
      join(corpus, "fonts"),
    );
  } finally {
    rmSync(root, { recursive: true, force: true });
  }
});

test("an empty override falls back to ~/origa-cdn in the home dir", () => {
  const root = mkdtempSync(join(tmpdir(), "origa-probe-fonts-"));
  try {
    const home = join(root, "home");
    mkdirSync(join(home, "origa-cdn", "fonts"), { recursive: true });
    assert.equal(
      resolveBuildFontsDir("", home),
      join(home, "origa-cdn", "fonts"),
    );
    // the fallback is not required to exist — absence resolves to null
    assert.equal(
      resolveBuildFontsDir(undefined, join(root, "bare-home")),
      null,
    );
  } finally {
    rmSync(root, { recursive: true, force: true });
  }
});

test("a corpus without fonts resolves to null", () => {
  const root = mkdtempSync(join(tmpdir(), "origa-probe-fonts-"));
  try {
    const corpus = join(root, "corpus");
    mkdirSync(corpus, { recursive: true }); // no fonts/ inside
    assert.equal(resolveBuildFontsDir(corpus, join(root, "home")), null);
  } finally {
    rmSync(root, { recursive: true, force: true });
  }
});

// build.rs resolves with is_dir(): a regular FILE named fonts degrades the
// build to sans-serif, and the mirror must say the same thing (review of
// task 20261110041750-4bs23o3 — a bare existsSync would pass preflight).
test("a regular file named fonts is not a fonts source", () => {
  const root = mkdtempSync(join(tmpdir(), "origa-probe-fonts-"));
  try {
    const corpus = join(root, "corpus");
    mkdirSync(corpus, { recursive: true });
    writeFileSync(join(corpus, "fonts"), "");
    assert.equal(resolveBuildFontsDir(corpus, join(root, "home")), null);
  } finally {
    rmSync(root, { recursive: true, force: true });
  }
});
