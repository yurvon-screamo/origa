// Sleepy: the mascot dozes off — eyes closed (our blink pose), the library's
// status/sleep floats soft Zs above the head with dream motes and a breathing
// glow. Meant for empty and idle-quiet screens.
import { resolveOptions } from "pixelfx";
import { draw as drawMascot, POSES, mascotActor } from "../mascot/origa.js";
import * as sleepFx from "pixelfx/fx/status/sleep";

export const meta = {
  name: "Sleepy",
  desc: "Mascot dozing: closed eyes, Zs floating up, dream motes.",
  tags: ["status", "idle"],
  size: [72, 108],
  origin: [36, 96],
  duration: 2.4, // matches the library loop
  loop: true,
  fps: 12,
  bg: "none",
  options: {},
};

export function draw(p, t, o) {
  drawMascot(p, POSES.blink); // eyes closed the whole time
  const fxOpts = resolveOptions(sleepFx.meta, { ...o, actor: mascotActor(POSES.blink) });
  sleepFx.draw(p, t, fxOpts);
}
