// Holy answer: the library's heal — petals bloom at the feet, a hoop of
// light climbs the mascot, motes pop into tiny hearts. "You got it right."
import { resolveOptions } from "pixelfx";
import { draw as drawMascot, mascotActor } from "../mascot/origa.js";
import * as healFx from "pixelfx/fx/holy/heal";

export const meta = {
  name: "Holy answer",
  desc: "Petals at the feet, a light hoop climbing the mascot, tiny hearts.",
  tags: ["holy", "feedback"],
  size: [88, 128],
  origin: [44, 120],
  duration: 1.9, // matches the library one-shot
  loop: false,
  fps: 8,
  bg: "none",
  options: {},
};

export function drawBack(p, t, o) {
  healFx.drawBack?.(p, t, o);
}

export function create(o) {
  const opts = resolveOptions(healFx.meta, { ...o, actor: mascotActor() });
  const inst = healFx.create(opts);

  return {
    update(dt) {
      inst.update(dt);
    },
    draw(p) {
      drawMascot(p); // body first: petals, hoop and hearts paint over it
      inst.draw(p);
    },
  };
}
