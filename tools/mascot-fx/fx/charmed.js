// Charmed: the FSRS "perfect answer" reaction — the library's status/charmed:
// a beating heart over the head, little hearts floating up, a blush of light.
import { resolveOptions } from "pixelfx";
import { draw as drawMascot, mascotActor } from "../mascot/origa.js";
import * as charmedFx from "pixelfx/fx/status/charmed";

export const meta = {
  name: "Charmed",
  desc: "A beating heart over the head, little hearts floating up.",
  tags: ["status", "feedback"],
  size: [72, 108],
  origin: [36, 96],
  duration: 2.4, // matches the library loop
  loop: true,
  fps: 12,
  bg: "none",
  options: {},
};

export function drawBack(p, t, o) {
  const fxOpts = resolveOptions(charmedFx.meta, { ...o, actor: mascotActor() });
  charmedFx.drawBack?.(p, t, fxOpts);
}

export function draw(p, t, o) {
  drawMascot(p);
  const fxOpts = resolveOptions(charmedFx.meta, { ...o, actor: mascotActor() });
  charmedFx.draw(p, t, fxOpts);
}
