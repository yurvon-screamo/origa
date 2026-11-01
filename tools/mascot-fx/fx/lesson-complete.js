// Lesson complete one-shot: the library's confetti (two party poppers,
// paper bits, streamers) bursting over the mascot, who stands in the middle.
// Composed in the shared 64x96 world; export upscales by an integer factor.
import { resolveOptions } from "pixelfx";
import { draw as drawMascot } from "../mascot/origa.js";
import * as confettiFx from "pixelfx/fx/ui/confetti";

export const meta = {
  name: "Lesson complete",
  desc: "Party poppers bang confetti over the mascot.",
  tags: ["ui", "reward"],
  size: [148, 136],
  origin: [74, 130],
  duration: 3.4, // matches confetti's own duration
  loop: false,
  fps: 8,
  bg: "none",
  options: {
    // brand-toned paper ramp: blonde, cream, blush, olive highlights
    palette: ["#f2d286", "#d9a858", "#e09a86", "#fdfcf7", "#8b9763", "#ffd98a"],
    seed: 11,
  },
};

export function create(o) {
  const opts = resolveOptions(confettiFx.meta, o);
  const confetti = confettiFx.create(opts);

  return {
    update(dt) {
      confetti.update(dt);
    },
    draw(p, t) {
      // back flutter behind the figure
      confetti.draw(p, "back");
      drawMascot(p);
      // front confetti, poppers and settled bits over everything
      confetti.draw(p, "front");
      void t;
    },
  };
}
