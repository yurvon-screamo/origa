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
// resolve_fonts_dir (task 20261110041750-4bs23o3): the check and the build
// must look at the same place, otherwise preflight passes while the build
// bakes a sans-serif bundle.
test("the env corpus with fonts wins", () => {
  const root = mkdtempSync(join(tmpdir(), "origa-probe-fonts-"));
  try {
    const corpus = join(root, "corpus");
    const repo = join(root, "repo");
    mkdirSync(join(corpus, "fonts"), { recursive: true });
    writeFileSync(join(corpus, "fonts", "a-12345678.woff2"), "");
    const resolved = resolveBuildFontsDir(corpus, repo);
    assert.equal(resolved, join(corpus, "fonts"));
  } finally {
    rmSync(root, { recursive: true, force: true });
  }
});

test("an env corpus without fonts falls back to the repo-local cdn/fonts", () => {
  const root = mkdtempSync(join(tmpdir(), "origa-probe-fonts-"));
  try {
    const corpus = join(root, "corpus");
    const repo = join(root, "repo");
    mkdirSync(corpus, { recursive: true }); // no fonts/ inside
    mkdirSync(join(repo, "cdn", "fonts"), { recursive: true });
    const resolved = resolveBuildFontsDir(corpus, repo);
    assert.equal(resolved, join(repo, "cdn", "fonts"));
  } finally {
    rmSync(root, { recursive: true, force: true });
  }
});

test("no env corpus uses the repo-local cdn/fonts", () => {
  const root = mkdtempSync(join(tmpdir(), "origa-probe-fonts-"));
  try {
    const repo = join(root, "repo");
    mkdirSync(join(repo, "cdn", "fonts"), { recursive: true });
    assert.equal(resolveBuildFontsDir(null, repo), join(repo, "cdn", "fonts"));
  } finally {
    rmSync(root, { recursive: true, force: true });
  }
});

test("no fonts anywhere resolves to null", () => {
  const root = mkdtempSync(join(tmpdir(), "origa-probe-fonts-"));
  try {
    // env corpus present but empty, repo-local absent
    const corpus = join(root, "corpus");
    const repo = join(root, "repo");
    mkdirSync(corpus, { recursive: true });
    assert.equal(resolveBuildFontsDir(corpus, repo), null);
    // no env corpus at all
    assert.equal(resolveBuildFontsDir(null, repo), null);
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
    const repo = join(root, "repo");
    mkdirSync(corpus, { recursive: true });
    writeFileSync(join(corpus, "fonts"), "");
    assert.equal(resolveBuildFontsDir(corpus, repo), null);
    // the repo-local dir is a real source — the fallback answers
    mkdirSync(join(repo, "cdn", "fonts"), { recursive: true });
    assert.equal(
      resolveBuildFontsDir(corpus, repo),
      join(repo, "cdn", "fonts"),
    );
  } finally {
    rmSync(root, { recursive: true, force: true });
  }
});
