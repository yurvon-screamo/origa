import assert from "node:assert/strict";
import { test } from "node:test";

import { winTrailKillCommand } from "./trail-kill.ts";

test("the kill is scoped to the harness port, not to the image name", () => {
  const command = winTrailKillCommand(4123);

  assert.match(command, /--address 127\.0\.0\.1:4123/);
  assert.doesNotMatch(command, /taskkill \/f \/im/);
});

test("the process filter names trail.exe and nothing else", () => {
  const command = winTrailKillCommand(4000);

  assert.match(command, /Name=''trail\.exe''/);
});
