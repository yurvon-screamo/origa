import assert from "node:assert/strict";
import { test } from "node:test";
import { crc32 } from "node:zlib";

import {
  readProof,
  readPngSize,
  repoImageName,
  sha256Hex,
  shotFileName,
  shotKey,
  stampProof,
} from "./manifest.ts";

/** Minimal PNG header: signature + IHDR with the given dimensions. */
function pngHeader(width: number, height: number): Buffer {
  const buffer = Buffer.alloc(24);
  buffer.writeUInt32BE(width, 16);
  buffer.writeUInt32BE(height, 20);
  return buffer;
}

test("sha256 of the same bytes is the same hash", () => {
  const first = sha256Hex(Buffer.from("origa"));
  const second = sha256Hex(Buffer.from("origa"));

  assert.equal(first, second);
  assert.equal(first.length, 64);
});

test("sha256 of different bytes differs", () => {
  assert.notEqual(
    sha256Hex(Buffer.from("origa")),
    sha256Hex(Buffer.from("origb")),
  );
});

test("png dimensions are read from the header", () => {
  const size = readPngSize(pngHeader(1280, 800));

  assert.equal(size.width, 1280);
  assert.equal(size.height, 800);
});

test("capture file name carries the scenario, the screen, the viewport and the locale", () => {
  assert.equal(
    shotFileName("home", "home", "desktop", "ru-RU"),
    "home--home--desktop--ru-RU.png",
  );
});

test("the repository image name is stable across captures", () => {
  assert.equal(
    repoImageName({
      scenario: "home",
      screen: "home",
      viewport: "mobile",
      locale: "ru-RU",
    }),
    "home--home--mobile--ru-RU.png",
  );
});

test("the locales of one screen do not share a file", () => {
  assert.notEqual(
    shotFileName("home", "home", "desktop", "ru-RU"),
    shotFileName("home", "home", "desktop", "en-US"),
  );
});

test("registry key separates the locales of one screen", () => {
  const ru = shotKey({
    scenario: "home",
    screen: "home",
    viewport: "desktop",
    locale: "ru-RU",
  });
  const en = shotKey({
    scenario: "home",
    screen: "home",
    viewport: "desktop",
    locale: "en-US",
  });

  assert.notEqual(ru, en);
});

/** A structurally valid PNG with no pixels: signature, IHDR, IEND. */
function minimalPng(): Buffer {
  const chunk = (type: string, data: Buffer): Buffer => {
    const result = Buffer.alloc(12 + data.length);
    result.writeUInt32BE(data.length, 0);
    result.write(type, 4, "latin1");
    data.copy(result, 8);
    result.writeUInt32BE(
      crc32(result.subarray(4, 8 + data.length)),
      8 + data.length,
    );
    return result;
  };
  return Buffer.concat([
    Buffer.from([137, 80, 78, 71, 13, 10, 26, 10]),
    chunk("IHDR", Buffer.alloc(13)),
    chunk("IEND", Buffer.alloc(0)),
  ]);
}

test("a proof survives a stamp and read round trip", () => {
  const proof = { schema: 1 as const, commit: "abc123", screen: "home" };
  const stamped = stampProof(minimalPng(), proof);

  assert.deepEqual(readProof(stamped), proof);
  assert.equal(readProof(minimalPng()), null);
});

test("a damaged proof chunk fails loudly", () => {
  const stamped = stampProof(minimalPng(), {
    schema: 1,
    commit: "abc123",
    screen: "home",
  });
  const damaged = Buffer.from(stamped);
  damaged[damaged.length - 20] ^= 0xff;

  assert.throws(() => readProof(damaged), /corrupt PNG chunk/);
});

test("a proof with an unknown schema is rejected", () => {
  const png = stampProof(minimalPng(), {
    schema: 99 as 1,
    commit: "abc123",
    screen: "home",
  });
  assert.throws(() => readProof(png), /invalid proof payload/);
});

test("the same pixels with a different commit have a different hash", () => {
  const png = minimalPng();
  const first = stampProof(png, { schema: 1, commit: "aaa", screen: "home" });
  const second = stampProof(png, { schema: 1, commit: "bbb", screen: "home" });

  assert.notEqual(sha256Hex(first), sha256Hex(second));
});
