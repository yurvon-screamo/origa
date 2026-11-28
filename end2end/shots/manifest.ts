import { createHash } from "node:crypto";
import { crc32 } from "node:zlib";

import type { ViewportTag } from "./viewports.ts";

/**
 * One captured screen.
 *
 * The entry is the contract between the capture process and the registry:
 * it carries the image facts (size, hash) AND the provenance (which
 * scenario, which build, which content source) — a screenshot without
 * provenance cannot be reproduced.
 */
export interface ShotEntry {
  scenario: string;
  title: string;
  screen: string;
  route: string;
  viewport: ViewportTag;
  locale: string;
  width: number;
  height: number;
  fullPage: boolean;
  bytes: number;
  sha256: string;
  file: string;
  capturedAt: string;
  commit: string;
  baseURL: string;
  cdnBaseUrl: string;
}

export interface ShotManifest {
  version: 1;
  label: string;
  entries: ShotEntry[];
}

/**
 * Harness-attested facts bound to the pixels, not to the JSON manifest.
 * Both fields have an independent counterpart at verification time.
 */
export interface ShotProof {
  schema: 1;
  commit: string;
  screen: string;
}

const PROOF_KEYWORD = "origa-proof";

/** Writes the proof chunk immediately before IEND, before hashing the PNG. */
export function stampProof(png: Buffer, proof: ShotProof): Buffer {
  const payload = Buffer.concat([
    Buffer.from(PROOF_KEYWORD, "latin1"),
    Buffer.from([0]),
    Buffer.from(JSON.stringify(proof), "utf8"),
  ]);
  const chunk = Buffer.alloc(12 + payload.length);
  chunk.writeUInt32BE(payload.length, 0);
  chunk.write("tEXt", 4, "latin1");
  payload.copy(chunk, 8);
  chunk.writeUInt32BE(
    crc32(chunk.subarray(4, 8 + payload.length)),
    8 + payload.length,
  );

  let offset = 8;
  while (offset + 12 <= png.length) {
    const length = png.readUInt32BE(offset);
    const type = png.toString("latin1", offset + 4, offset + 8);
    if (type === "IEND") {
      return Buffer.concat([
        png.subarray(0, offset),
        chunk,
        png.subarray(offset),
      ]);
    }
    offset += 12 + length;
  }
  throw new Error("invalid PNG: IEND not found");
}

/**
 * Reads the proof chunk; a damaged chunk is an error, never an absence.
 * Legacy frames without the chunk return null.
 */
export function readProof(png: Buffer): ShotProof | null {
  let offset = 8;
  while (offset + 12 <= png.length) {
    const length = png.readUInt32BE(offset);
    const type = png.toString("latin1", offset + 4, offset + 8);
    const data = png.subarray(offset + 8, offset + 8 + length);
    const checksum = png.readUInt32BE(offset + 8 + length);
    if (crc32(png.subarray(offset + 4, offset + 8 + length)) !== checksum) {
      throw new Error(`corrupt PNG chunk: ${type}`);
    }
    if (type === "tEXt") {
      const separator = data.indexOf(0);
      if (
        separator >= 0 &&
        data.toString("latin1", 0, separator) === PROOF_KEYWORD
      ) {
        return JSON.parse(data.toString("utf8", separator + 1)) as ShotProof;
      }
    }
    if (type === "IEND") break;
    offset += 12 + length;
  }
  return null;
}

/** SHA-256 of a buffer, hex encoded. */
export function sha256Hex(data: Buffer): string {
  return createHash("sha256").update(data).digest("hex");
}

/**
 * Pixel dimensions of a PNG, read from the IHDR chunk.
 *
 * The capture does not trust the requested viewport: the file on disk is
 * the fact that ends up in the registry.
 */
export function readPngSize(data: Buffer): { width: number; height: number } {
  return { width: data.readUInt32BE(16), height: data.readUInt32BE(20) };
}

/**
 * File name of a capture — deterministic, one screen per file.
 *
 * The locale is part of the name: two locales of one screen in the same
 * output directory would otherwise overwrite each other while the
 * registry keeps both entries.
 */
export function shotFileName(
  scenario: string,
  screen: string,
  viewport: ViewportTag,
  locale: string,
): string {
  return `${scenario}--${screen}--${viewport}--${locale}.png`;
}

/** Stable file name of a capture kept in the repository for PR review. */
export function repoImageName(entry: {
  scenario: string;
  screen: string;
  viewport: ViewportTag;
  locale: string;
}): string {
  return shotFileName(
    entry.scenario,
    entry.screen,
    entry.viewport,
    entry.locale,
  );
}

/** Key of a capture inside the registry. */
export function shotKey(entry: {
  scenario: string;
  screen: string;
  viewport: string;
  locale: string;
}): string {
  return `${entry.scenario}--${entry.screen}--${entry.viewport}--${entry.locale}`;
}
