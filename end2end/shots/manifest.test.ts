import assert from "node:assert/strict";
import { test } from "node:test";

import { readPngSize, sha256Hex, shotFileName, shotKey } from "./manifest.ts";

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
    assert.notEqual(sha256Hex(Buffer.from("origa")), sha256Hex(Buffer.from("origb")));
});

test("png dimensions are read from the header", () => {
    const size = readPngSize(pngHeader(1280, 800));

    assert.equal(size.width, 1280);
    assert.equal(size.height, 800);
});

test("capture file name carries the scenario, the screen and the viewport", () => {
    assert.equal(shotFileName("home", "home", "desktop"), "home--home--desktop.png");
});

test("registry key separates the locales of one screen", () => {
    const ru = shotKey({ scenario: "home", screen: "home", viewport: "desktop", locale: "ru-RU" });
    const en = shotKey({ scenario: "home", screen: "home", viewport: "desktop", locale: "en-US" });

    assert.notEqual(ru, en);
});
