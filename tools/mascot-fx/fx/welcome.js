// Welcome one-shot: the mascot pops out of a cartoon poof, then nods twice
// while golden glints pop around the head. The poof is the library's own
// spawn effect; the entrance fade uses sprite alpha.
import { resolveOptions, sparkle } from "pixelfx";
import { draw as drawMascot, POSES } from "../mascot/origa.js";
import { PAL } from "../mascot/palette.js";
import * as poofFx from "pixelfx/fx/spawn/poof";

export const meta = {
  name: "Welcome",
  desc: "Mascot pops out of a poof, nods twice, glints sparkle.",
  tags: ["ui", "greeting"],
  size: [112, 108],
  origin: [56, 96],
  duration: 2.0,
  loop: false,
  fps: 12,
  bg: "none",
  options: {},
};

const NODS = [
  [0.9, 1.35],
  [1.6, 1.9],
];
const GLINTS = [
  [-26, -84, 1.0],
  [22, -76, 1.2],
  [-32, -50, 1.4],
  [28, -46, 1.55],
];

export function create(o) {
  const opts = resolveOptions(poofFx.meta, {
    ...o,
    seed: (o.seed ?? 0) + 5,
    // brand-toned smoke: warm creams instead of the stock purple haze
    palette: ["#8a7a58", "#b9a878", "#d9c898", "#f2e4bc", "#fbf6ea", "#ffffff"],
    stars: ["#ffd98a", "#fff3d0", "#ffffff"],
  });
  const poof = poofFx.create(opts);
  let t = 0;

  return {
    update(dt) {
      t += dt;
      poof.update(dt);
    },
    draw(p) {
      // poof plays over the torso; the mascot materializes through it
      p.place(0, -48);
      poof.draw(p);
      const fade = Math.min(1, Math.max(0, (t - 0.25) / 0.45));
      p.place(0, 0);
      drawMascot(p, POSES.idle, { alpha: fade });

      // nods: the figure dips 2px while a nod is active
      for (const [a, b] of NODS) {
        if (t >= a && t < b) {
          p.place(0, -2);
          drawMascot(p, POSES.idle, { alpha: fade });
          break;
        }
      }

      // golden glints around the head after the poof settles
      for (const [gx, gy, at] of GLINTS) {
        const lt = t - at;
        if (lt < 0 || lt > 0.5) continue;
        const arm = 6 * Math.sin((lt / 0.5) * Math.PI);
        sparkle(p, gx, gy, arm, PAL.l, PAL.w);
      }
    },
  };
}
