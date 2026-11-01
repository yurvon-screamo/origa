// Render a mascot pose to a transparent PNG preview (nearest-neighbor zoom).
// Usage: node render-sprite.mjs [pose] [zoom]   e.g. node render-sprite.mjs idle 8
import { writeFileSync } from "node:fs";
import { createFrame, clearFrame, composite, painterFor } from "pixelfx";
import { encodePNG } from "./node_modules/pixelfx/tools/lib/png.mjs";
import { W, H, POSES, draw } from "./mascot/origa.js";

const poseName = process.argv[2] ?? "idle";
const zoom = Number(process.argv[3] ?? 8);
const pose = POSES[poseName];
if (!pose) throw new Error(`unknown pose: ${poseName} (have: ${Object.keys(POSES).join(", ")})`);

// Matrix sanity: every row must be exactly W wide over the A-Z of the palette.
const allowed = new Set("lLhsdbHmNnNkwp.".split(""));
pose.forEach((row, i) => {
  if (row.length !== W) throw new Error(`row ${i}: length ${row.length} != ${W}\n  ${row}`);
  for (const ch of row) {
    if (!allowed.has(ch)) throw new Error(`row ${i}: unknown char "${ch}"\n  ${row}`);
  }
});

// Canvas window in local space: feet land at (cw/2, ch-6) on screen.
// ox/oy is NEGATIVE (pixelfx convention, see fxlib renderFrames: -m).
const cw = W + 24, ch = H + 12;
const frame = createFrame(cw, ch, -cw / 2, -(ch - 6));
const p = painterFor(frame);
clearFrame(frame);
p.place(0, 0);
draw(p, pose);

const img = new Uint8Array(cw * ch * 4);
composite(frame, img, null);
const scaled = new Uint8Array(cw * zoom * ch * zoom * 4);
for (let y = 0; y < ch * zoom; y++) {
  for (let x = 0; x < cw * zoom; x++) {
    const si = (Math.floor(y / zoom) * cw + Math.floor(x / zoom)) * 4;
    const di = (y * cw * zoom + x) * 4;
    scaled[di] = img[si]; scaled[di + 1] = img[si + 1];
    scaled[di + 2] = img[si + 2]; scaled[di + 3] = img[si + 3];
  }
}
const file = `records/sprite-${poseName}.png`;
writeFileSync(file, encodePNG(cw * zoom, ch * zoom, scaled));
console.log(`wrote ${file} (${cw * zoom}x${ch * zoom})`);
