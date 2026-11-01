// Achievement toast: the library's ui/achievement toast slides in over the
// mascot's head (gold medal pops, shine sweeps), the figure standing below.
import { resolveOptions } from "pixelfx";
import { draw as drawMascot } from "../mascot/origa.js";
import * as achFx from "pixelfx/fx/ui/achievement";

export const meta = {
  name: "Achievement",
  desc: "Achievement toast pops in over the mascot's head.",
  tags: ["ui", "reward"],
  size: [224, 170],
  origin: [112, 136],
  duration: 2.4, // matches the library one-shot
  loop: false,
  fps: 8,
  bg: "none",
  options: {},
};

export function create(o) {
  const opts = resolveOptions(achFx.meta, o);
  const inst = achFx.create(opts);

  return {
    update(dt) {
      inst.update(dt);
    },
    draw(p) {
      // toast floats above the head, the mascot stands below it
      p.place(0, -128);
      inst.draw(p);
      p.place(0, 0);
      drawMascot(p);
    },
  };
}
