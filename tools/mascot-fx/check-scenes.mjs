// Scene sanity check: determinism (same t -> byte-identical frame) and
// per-frame draw cost (< 2 ms, the pixelfx bar). Runs against studio scenes.
//   node check-scenes.mjs [scene ...]
import { createFrame, clearFrame, composite, painterFor, resolveOptions } from "pixelfx";

const SCENES = {
  idle: "./fx/idle-blink.js",
  welcome: "./fx/welcome.js",
  complete: "./fx/lesson-complete.js",
  streak: "./fx/streak-flame.js",
  japaneseSpring: "./fx/japanese-spring.js",
  levelUp: "./fx/level-up.js",
  holyAnswer: "./fx/holy-answer.js",
  sleepy: "./fx/sleepy.js",
  charmed: "./fx/charmed.js",
  achievement: "./fx/achievement.js",
};

let failed = 0;
for (const [name, file] of Object.entries(SCENES)) {
  const mod = await import(new URL(file, import.meta.url).href);
  const meta = mod.meta;
  const [w, h] = meta.size;
  const opts = resolveOptions(meta, { seed: 7 });
  const inst = mod.create ? mod.create(opts) : null;
  const sceneDraw = inst ? (p, t) => inst.draw(p, t) : (p, t) => mod.draw(p, t, opts);

  const frame = createFrame(w, h, -meta.origin[0], -meta.origin[1]);
  const painter = painterFor(frame);
  const a = new Uint8ClampedArray(w * h * 4);
  const b = new Uint8ClampedArray(w * h * 4);

  // advance a simulated scene halfway, then compare two renders of t
  let simTime = 0;
  const half = meta.duration / 2;
  const drawAt = (t, out) => {
    if (inst) {
      while (simTime < t - 1e-9) {
        inst.update(1 / 120);
        simTime += 1 / 120;
      }
      simTime = t; // pure scenes don't update; sims snap forward deterministically
    }
    clearFrame(frame);
    painter.reset();
    painter.place(0, 0);
    sceneDraw(painter, t);
    composite(frame, out, null);
  };

  drawAt(half, a);
  drawAt(half, b);
  let identical = true;
  for (let i = 0; i < a.length; i++) {
    if (a[i] !== b[i]) { identical = false; break; }
  }

  // perf: average over 30 frames spread across the duration
  const t0 = performance.now();
  const probes = 30;
  for (let i = 0; i < probes; i++) drawAt((meta.duration * i) / probes, a);
  const ms = (performance.now() - t0) / probes;

  const ok = identical && ms < 2;
  if (!ok) failed++;
  console.log(`${name}: deterministic=${identical ? "yes" : "NO"} draw=${ms.toFixed(2)}ms ${ok ? "ok" : "FAIL"}`);
}
process.exit(failed ? 1 : 0);
