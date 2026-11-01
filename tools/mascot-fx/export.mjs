// Batch-export mascot scenes to product assets:
//   animated WebP (lossless, 12 fps) + still PNG + records/manifest.json
//   Usage: node export.mjs [scene ...]   (default: all)
// Destinations (see README.md):
//   app assets      -> origa_ui/public/mascot/
//   landing copies  -> origa_landing/public/images/mascot/
//   marketing-only  -> tools/mascot-fx/assets/   (streak: NOT shipped in-app)
import { spawnSync } from "node:child_process";
import { copyFileSync, mkdirSync, rmSync, statSync, writeFileSync } from "node:fs";
import { join } from "node:path";
import { createFrame, clearFrame, composite, painterFor, resolveOptions } from "pixelfx";
import { encodePNG } from "./node_modules/pixelfx/tools/lib/png.mjs";
import { mascotActor } from "./mascot/origa.js";

const STUDIO = import.meta.dirname;
const REPO_ROOT = join(STUDIO, "..", "..");
const SIM_HZ = 120; // simulated effects step at 120 Hz or finer

const SCENES = {
  idle: { file: "./fx/idle-blink.js", dests: ["app", "landing"], budget: 150, zoom: 2 },
  welcome: { file: "./fx/welcome.js", dests: ["app"], budget: 100, zoom: 2 },
  complete: { file: "./fx/lesson-complete.js", dests: ["app"], budget: 150, zoom: 2 },
  streak: { file: "./fx/streak-flame.js", dests: ["marketing"], budget: 300, zoom: 2 },
  // demo-only gallery scenes: dests [] -> render to records/, no product copies
  japaneseSpring: { file: "./fx/japanese-spring.js", dests: [], budget: 300, zoom: 2 },
  levelUp: { file: "./fx/level-up.js", dests: [], budget: 300, zoom: 2 },
  holyAnswer: { file: "./fx/holy-answer.js", dests: [], budget: 300, zoom: 2 },
  sleepy: { file: "./fx/sleepy.js", dests: [], budget: 300, zoom: 2 },
  charmed: { file: "./fx/charmed.js", dests: [], budget: 300, zoom: 2 },
  achievement: { file: "./fx/achievement.js", dests: [], budget: 300, zoom: 1 },
};

const DEST_DIRS = {
  app: join(REPO_ROOT, "origa_ui", "public", "mascot"),
  landing: join(REPO_ROOT, "origa_landing", "public", "images", "mascot"),
  marketing: join(STUDIO, "assets"),
};

async function exportScene(name, cfg) {
  const mod = await import(new URL(cfg.file, import.meta.url).href);
  const meta = mod.meta;
  const [w, h] = meta.size;
  const [ox, oy] = meta.origin;
  const zoom = cfg.zoom ?? 2;
  const fps = meta.fps;
  const n = Math.max(1, Math.round(meta.duration * fps) + (meta.loop ? 0 : 1));
  const opts = resolveOptions(meta, { seed: 7, ...(meta.actor ? { actor: mascotActor() } : {}) });

  // unified scene interface: draw(painter, t); simulated scenes also update(dt)
  const inst = mod.create ? mod.create(opts) : null;
  const sceneDraw = inst ? (p, t) => inst.draw(p, t) : (p, t) => mod.draw(p, t, opts);
  const sceneUpdate = inst ? (dt) => inst.update(dt) : null;
  const drawBack = inst?.drawBack ?? mod.drawBack ?? null;

  const frame = createFrame(w, h, -ox, -oy);
  const painter = painterFor(frame);
  const rgba = new Uint8ClampedArray(w * h * 4);

  const dir = join(STUDIO, "records", `export-${name}`);
  // stale frames from a previous run (different fps/zoom) poison the WebP:
  // wipe the whole dir before rendering
  rmSync(dir, { recursive: true, force: true });
  mkdirSync(dir, { recursive: true });

  let simTime = 0;
  const frames = [];
  for (let i = 0; i < n; i++) {
    const t = i / fps;
    if (sceneUpdate) {
      while (simTime < t - 1e-9) {
        sceneUpdate(1 / SIM_HZ);
        simTime += 1 / SIM_HZ;
      }
    }
    clearFrame(frame);
    painter.reset();
    painter.place(0, 0);
    if (drawBack) drawBack(painter, t, opts);
    sceneDraw(painter, t);
    composite(frame, rgba, null);

    const f = join(dir, `f${String(i).padStart(3, "0")}.png`);
    const big = new Uint8ClampedArray(w * zoom * h * zoom * 4);
    for (let by = 0; by < h * zoom; by++) {
      for (let bx = 0; bx < w * zoom; bx++) {
        const si = (Math.floor(by / zoom) * w + Math.floor(bx / zoom)) * 4;
        const di = (by * w * zoom + bx) * 4;
        big[di] = rgba[si]; big[di + 1] = rgba[si + 1]; big[di + 2] = rgba[si + 2]; big[di + 3] = rgba[si + 3];
      }
    }
    writeFileSync(f, encodePNG(w * zoom, h * zoom, big));
    frames.push(f);
  }

  // still: first frame for loops (neutral pose), last for one-shots (settled)
  const stillSrc = meta.loop ? frames[0] : frames[frames.length - 1];

  const webp = join(dir, `${name}.webp`);
  const lossless = !cfg.dests.includes("marketing") ? "1" : "0";
  const py = spawnSync("python", [
    "scripts/webp_from_frames.py", dir, webp, String(fps),
    meta.loop ? "0" : "1", lossless,
  ], { stdio: "pipe" });
  if (py.status !== 0) throw new Error(`webp assembly (${name}): ${py.stderr}`);

  const kb = Math.round((statSync(webp).size / 1024) * 10) / 10;
  if (kb > cfg.budget) {
    console.warn(`WARNING: ${name} ${kb}KB exceeds budget ${cfg.budget}KB (fps -> 8 next, see README)`);
  }

  const stillName = `${name}-still.png`;
  copyFileSync(stillSrc, join(dir, stillName));

  for (const dest of cfg.dests) {
    mkdirSync(DEST_DIRS[dest], { recursive: true });
    copyFileSync(webp, join(DEST_DIRS[dest], `${name}.webp`));
    copyFileSync(stillSrc, join(DEST_DIRS[dest], stillName));
  }

  return { fps, loop: meta.loop, frames: n, frameSize: [w * zoom, h * zoom], kb, budget: cfg.budget, dests: cfg.dests };
}

async function main() {
  const wanted = process.argv.slice(2);
  const names = wanted.length ? wanted : Object.keys(SCENES);
  const manifest = { scenes: {}, totalKB: 0 };

  for (const name of names) {
    const cfg = SCENES[name];
    if (!cfg) throw new Error(`unknown scene: ${name} (have: ${Object.keys(SCENES).join(", ")})`);
    const info = await exportScene(name, cfg);
    manifest.scenes[name] = info;
    // the 600KB budget covers SHIPPED assets only: marketing (streak) lives
    // outside the product's public dirs and does not count
    if (info.dests.some((d) => d !== "marketing")) {
      manifest.totalKB = Math.round((manifest.totalKB + info.kb) * 10) / 10;
    }
    console.log(`${name}: ${info.kb}KB ${info.loop ? "loop" : "one-shot"} (${info.frames}f @${info.fps}fps) -> ${info.dests.join(", ")}`);
  }

  writeFileSync(join(STUDIO, "records", "manifest.json"), JSON.stringify(manifest, null, 2));
  // the app reads the manifest from its own public dir (consistency test:
  // src/ui_components/mascot_animation.rs includes it via include_str!)
  mkdirSync(DEST_DIRS.app, { recursive: true });
  writeFileSync(join(DEST_DIRS.app, "manifest.json"), JSON.stringify(manifest, null, 2));
  console.log(`product total: ${manifest.totalKB}KB / 600KB budget`);
}

main().catch((e) => {
  console.error(e);
  process.exit(1);
});
