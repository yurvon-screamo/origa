import assert from "node:assert/strict";
import { test } from "node:test";

import { isInside, readProcCwd } from "./probe.ts";

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
test("the cwd probe reads the live /proc symlink", { skip: process.platform !== "linux" }, () => {
    assert.equal(readProcCwd(process.pid), process.cwd());
});

test("a cwd probe of a non-existent pid degrades to unknown ownership", () => {
    assert.equal(readProcCwd(-1), null);
});
