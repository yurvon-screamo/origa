import assert from "node:assert/strict";
import { test } from "node:test";

import { winTrailKillCommand } from "./trail-kill.ts";

test("the kill is scoped to the harness port, not to the image name", () => {
  const command = winTrailKillCommand(4123);

  assert.ok(command.includes("--address 127\\.0\\.0\\.1:4123(\\s|$)"));
  assert.doesNotMatch(command, /taskkill \/f \/im/);
});

test("the port match is anchored — a longer port is not a prefix hit", () => {
  // 4000 must not match a stranger's TrailBase spawned on 40000: the
  // address argument is followed by the anchor, not by another digit.
  const command = winTrailKillCommand(4000);

  assert.ok(!command.includes(":40000"));
  assert.ok(command.includes(":4000(\\s|$)"));
});

test("the process filter names trail.exe and nothing else", () => {
  const command = winTrailKillCommand(4000);

  assert.match(command, /Name=''trail\.exe''/);
});
