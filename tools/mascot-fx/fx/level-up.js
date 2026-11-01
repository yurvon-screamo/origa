// JLPT level up: the library's level-up pillar of light and rings climbing
// the mascot, LEVEL UP! lettering dropping in above. onBody: the effect
// receives our sprite through o.actor and paints its light on it.
import { resolveOptions } from "pixelfx";
import { draw as drawMascot, mascotActor } from "../mascot/origa.js";
import * as levelFx from "pixelfx/fx/ui/level-up";

export const meta = {
  name: "JLPT level up",
  desc: "Gold pillar of light climbs the mascot, LEVEL UP! above.",
  tags: ["ui", "reward"],
  size: [112, 136],
  origin: [56, 120],
  duration: 3.2, // matches the library one-shot
  loop: false,
  fps: 8,
  bg: "none",
  options: {},
};

export function drawBack(p, t, o) {
  levelFx.drawBack?.(p, t, o);
}

export function create(o) {
  const opts = resolveOptions(levelFx.meta, { ...o, actor: mascotActor() });
  const inst = levelFx.create(opts);

  return {
    update(dt) {
      inst.update(dt);
    },
    draw(p) {
      drawMascot(p); // the body first: the pillar's tint paints over it
      inst.draw(p);
    },
  };
}
