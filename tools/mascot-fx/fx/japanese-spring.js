// Japanese spring: the library's cherry-blossom rain over a plum dusk sky,
// the mascot standing under it. Full-scene weather effect — the frame keeps
// the effect's own dark sky (an intentional contrast card in the demo).
import { resolveOptions } from "pixelfx";
import { draw as drawMascot } from "../mascot/origa.js";
import * as sakuraFx from "pixelfx/fx/weather/cherry-blossoms";

export const meta = {
  name: "Japanese spring",
  desc: "Cherry blossoms rain over the mascot under a plum dusk sky.",
  tags: ["weather", "landing"],
  size: [160, 96],
  origin: [80, 96],
  duration: 10, // matches the library loop length
  loop: true,
  fps: 8,
  bg: "none",
  options: {
    // the library's petal ramp, verbatim
    palette: ["#4a1832", "#8e3460", "#d26a94", "#f2a2c0", "#ffd2e2", "#fff4f8"],
    seed: 12,
  },
};

export function drawBack(p, t, o) {
  sakuraFx.drawBack(p, t, o);
}

export function draw(p, t, o) {
  sakuraFx.draw(p, t, o);
  p.place(0, 0);
  drawMascot(p);
}
