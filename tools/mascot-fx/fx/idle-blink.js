// Idle loop: the mascot blinks twice per cycle and breathes (a slow 1px bob).
// Pure loop — the frame is a function of t alone, so the loop tiles seamlessly.
// World: 64x96 (the sprite is drawn 1:1 with library effects; the export
// pipeline upscales the whole frame by an integer factor).
import { draw as drawMascot, POSES } from "../mascot/origa.js";

export const meta = {
  name: "Idle blink",
  desc: "Mascot standing easy: blinks twice, breathes with a slow 1px bob.",
  tags: ["ui", "idle"],
  size: [72, 100],
  origin: [36, 96],
  duration: 3.2,
  loop: true,
  fps: 12,
  bg: "none",
  options: {},
};

const BLINKS = [
  [1.0, 1.18],
  [2.6, 2.78],
];

export function draw(p, t) {
  // breathe: the figure dips 1px through the middle of the loop, back by the end
  const bob = t > 1.5 && t < 2.9 ? 1 : 0;
  p.place(0, -bob);
  const blinking = BLINKS.some(([a, b]) => t >= a && t < b);
  drawMascot(p, blinking ? POSES.blink : POSES.idle);
}
