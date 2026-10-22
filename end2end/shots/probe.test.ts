import assert from "node:assert/strict";
import { test } from "node:test";

import { isInside } from "./probe.ts";

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
