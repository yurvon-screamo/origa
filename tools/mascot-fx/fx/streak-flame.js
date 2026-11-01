// Streak loop: a compact campfire built from the Sprite Fusion fire recipe's
// own building blocks (tongues, puffs, smoke, embers, sparks, floor light) at
// the mascot's scale, with a modest warm light pool that doesn't swallow the
// frame. Adapted from Sprite Fusion's open-source fire
// (https://www.spritefusion.com/pixel-art-vfx/fire), MIT License,
// Copyright (c) 2026 Hugo Duprez.
import { Rng, fit, resolveOptions } from "pixelfx";
import {
  noiseFor, hotFirst, flameTongue, lick, smokePuff, ember, spark,
  drift, sparkAt, floorLight, looped,
} from "pixelfx/fx/fire/_lib";
import { draw as drawMascot } from "../mascot/origa.js";

export const meta = {
  name: "Streak flame",
  desc: "A compact campfire beside the mascot: tongues, sparks, embers, smoke, warm light.",
  tags: ["ui", "fire", "loop"],
  size: [96, 104],
  origin: [48, 96],
  duration: 3,
  loop: true,
  fps: 8,
  bg: "none",
  options: {
    // terracotta-tinted fire ramp, dark to bright (Origa brand accent)
    palette: ["#4a2018", "#8a341f", "#c94a26", "#f2701d", "#ff8a2a", "#ffc23d", "#ffe26b", "#fff3a6", "#ffffff"],
    wood: ["#0e080a", "#24130f", "#3b2216", "#5a3520", "#8a5a34", "#b9884f"],
    seed: 3,
  },
};

const D = meta.duration, NL = noiseFor(D);
const FIRE = { x: -27, y: -3 }; // fire floor shares y 0 with the mascot's feet

// tongues: [dx, base, height share, half width, back row?]
const TONGUES = [
  [-6, -3, 0.35, 1.5, 1],
  [4.5, -3, 0.4, 1.6, 1],
  [-2.5, -4, 0.7, 1.9, 0],
  [2, -4, 0.74, 1.9, 0],
  [0, -5, 1, 2.6, 0],
];

export function create(o) {
  const F = fit(o.palette, 9), H = hotFirst(F), W = fit(o.wood, 6), S = o.seed ?? 3;
  const SPARKS = [H[1], H[2], H[4], H[5]], EMB = [H[1], H[2], H[4], H[6]];
  let t = 0;

  return {
    update(dt) { t += dt; },
    draw(p) {
      const T = t;
      const nl = (rate, s = 0, off = 0) => NL(T, rate, s, off);
      const fl = 0.74 + 0.16 * nl(7.3, S + 1) + 0.1 * nl(19, S + 2);
      const lean = nl(0.67, S + 90) * 0.12;
      const R = new Rng(S);

      const T_ = TONGUES.map(([dx, y, h, w, back], i) => ({
        dx: dx + R.range(-0.3, 0.3), y, h: 3 + 20 * h * R.range(0.92, 1.08),
        w: w * R.range(0.9, 1.1), back, n: (S * 7 + i) * 13,
      }));
      const height = (i) => {
        const tg = T_[i], leap = Math.max(0, nl(2.6, tg.n + 70) - 0.55) * 1.4;
        return (tg.h + 0.25 * tg.h * nl(3.3, tg.n) + tg.h * leap) * (0.9 + 0.1 * fl);
      };
      const sweep = (i, h) => nl(2.2, T_[i].n + 40) * (1 + h * 0.08) + lean * h * 0.9;
      const tip = (i) => {
        const h = height(i);
        return [T_[i].dx + sweep(i, h) * 0.85, T_[i].y - h * 0.88 + 3];
      };
      const tongueAt = (i) => {
        const tg = T_[i], h = height(i);
        flameTongue(p, tg.dx, tg.y, h, tg.w, sweep(i, h), nl, tg.n, H,
          tg.back
            ? [[1, 1, 7], [0.76, 0.86, 5], [0.54, 0.68, 3], [0.3, 0.42, 2]]
            : [[1, 1, 6], [0.76, 0.86, 4], [0.54, 0.68, 2], [0.3, 0.42, 0]],
          0.5, 1.1 + h * 0.04);
      };

      // smoke curling off the top, behind everything
      looped(T, D, 20, S + 300, (r) => ({ x: r.range(-3, 3), y: -14 - r.range(0, 5), vx: r.range(-2, 2), vy: -r.range(10, 16), life: r.range(1.6, 2.4), s: r.range(1.8, 2.8) }), (q, age) => {
        const [x, y] = drift(q.x, q.y, q.vx, q.vy, age, 3, -16, 1.4, 1);
        smokePuff(p, x + nl(0.8, S + 60) * (1.5 + 2 * age), y, 1 - age / q.life, q.s, 0);
      });

      for (const tg of T_) if (tg.back) tongueAt(T_.indexOf(tg));

      // licks torn off the tips, rising and shrinking
      looped(T, D, T_.length * 8, S + 220, (r, j) => ({ i: j % T_.length, vx: r.range(-3, 3), vy: -r.range(12, 22), s: r.range(0.9, 1.5) * (0.8 + 0.5 * T_[j % T_.length].h / 17), life: 0.26 }), (q, age) => {
        const [tx, ty] = tip(q.i);
        lick(p, tx + q.vx * age, ty - 2 + q.vy * age, 1 - age / q.life, q.s, H);
      });

      for (const tg of T_) if (!tg.back) tongueAt(T_.indexOf(tg));

      // popping sparks thrown up out of the wood
      looped(T, D, 3, S + 400, (r) => ({ x: FIRE.x + r.range(-6, 6), y: FIRE.y - r.range(4, 8), n: r.int(3, 5), seed: r.int(0, 1e6), life: 0.7 }), (q, age) => {
        if (age < 0.08) {
          p.px(q.x, q.y, "#ffffff");
          p.glow(q.x, q.y, 9, H[1], 0.6 * (1 - age / 0.08));
        }
        const r2 = new Rng(q.seed);
        for (let j = 0; j < q.n; j++) {
          const a = -Math.PI / 2 + r2.range(-0.8, 0.8), sp = r2.range(90, 180), life = r2.range(0.25, 0.6);
          if (age >= life) continue;
          const [x, y] = sparkAt(q.x, q.y, Math.cos(a) * sp, Math.sin(a) * sp, age, 0);
          spark(p, x, y, Math.cos(a) * sp, Math.sin(a) * sp, 1 - age / life, SPARKS);
        }
      });

      // embers carried up by the heat, wandering and cooling
      looped(T, D, 16, S + 500, (r) => ({ x: FIRE.x + r.range(-5, 5), y: FIRE.y - r.range(8, 14), vx: r.range(-14, 14), vy: -r.range(24, 55), life: r.range(1.1, 2.2), s: r.range(1, 1.5), a: r.range(0, 6.3), b: r.range(0, 6.3) }), (q, age) => {
        const [x, y] = drift(q.x, q.y, q.vx, q.vy, age, 0, -26, 1.5, 1.5);
        ember(p, x + 5 * (Math.sin(age * 2.3 + q.a) - Math.sin(q.a)), y + 1.5 * (Math.sin(age * 3 + q.b) - Math.sin(q.b)), 1 - age / q.life, q.s, EMB, age * 40 + q.a);
      });

      // charred logs bed (simple crossed pair: dark rim + lit top)
      p.line(FIRE.x - 11, FIRE.y - 1, FIRE.x + 9, FIRE.y - 5, W[1]);
      p.line(FIRE.x - 9, FIRE.y, FIRE.x + 11, FIRE.y - 4, W[2]);
      p.line(FIRE.x - 10, FIRE.y - 4, FIRE.x + 10, FIRE.y - 1, W[0]);

      // the mascot, drawn last: the fire's light buffer already tints it warm
      p.place(0, 0);
      drawMascot(p);

      // light: a tight hot heart at the flames + a modest floor pool
      p.glow(FIRE.x, FIRE.y - 12, 22 * fl, H[2], 0.3 * fl);
      p.glow(FIRE.x, FIRE.y - 5, 10, H[0], 0.15 * fl);
      floorLight(p, FIRE.x, 0.5, 22, 5, H[3], 0.22 * fl);
    },
  };
}
