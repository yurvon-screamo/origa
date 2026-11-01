// Origa mascot palette — 15 colors, quantized from origa_ui/public/logo.png
// (popularity count over opaque pixels, AA edges dropped, hand-tuned to brand).
// Single source of truth for every mascot scene. Do NOT trim colors to save
// bytes: brand consistency outranks file size (see tools/mascot-fx/README.md).
export const PAL = {
  l: "#f2d286", // blond light
  L: "#d9a858", // blond mid
  h: "#a87834", // blond shade
  s: "#f4d3a6", // skin light
  d: "#c9906c", // skin shade (nose, mouth)
  b: "#e09a86", // blush
  H: "#8b9763", // hoodie light
  m: "#78885a", // hoodie mid
  n: "#5b6b48", // hoodie shade
  N: "#3a4a30", // hoodie deep / legs
  k: "#100e0a", // ink: glasses, outlines, soles (no pure black)
  w: "#fdfcf7", // white: eye whites, sneakers
  p: "#7a4a20", // pupils
};
