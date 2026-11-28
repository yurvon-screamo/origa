import assert from "node:assert/strict";
import { mkdtemp, readFile } from "node:fs/promises";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { test } from "node:test";
import { crc32 } from "node:zlib";
import type { Page } from "@playwright/test";

import { createSink } from "./capture.ts";
import { readProof } from "./manifest.ts";

// The sink measures computed styles; the stubbed element carries them.
Object.assign(globalThis, {
  getComputedStyle: (element: { style: unknown }): unknown => element.style,
  window: { innerHeight: 800 },
});

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

interface MarkerState {
  readonly count?: number;
  readonly opacity?: number;
  readonly top?: number;
  readonly bottom?: number;
  readonly splashAttached?: boolean;
}

function fakePage(state: MarkerState = {}): Page {
  const marker = {
    first: () => marker,
    count: async () => state.count ?? 1,
    evaluate: async (
      callback: (element: unknown, allowOffscreen: boolean) => boolean,
      allowOffscreen: boolean,
    ) =>
      callback(
        {
          style: {
            opacity: String(state.opacity ?? 1),
            visibility: "visible",
            display: "block",
          },
          getBoundingClientRect: () => ({
            top: state.top ?? 10,
            bottom: state.bottom ?? 200,
            height: (state.bottom ?? 200) - (state.top ?? 10),
          }),
        },
        allowOffscreen,
      ),
  };
  const splash = {
    first: () => splash,
    waitFor: async () => {
      if (state.splashAttached) throw new Error("splash still attached");
    },
  };
  const page = {
    locator: (selector: string) =>
      selector === "#origa-boot-splash" ? splash : marker,
    getByTestId: (id: string) => (id === "splash" ? splash : marker),
    screenshot: async () => minimalPng(),
  };
  return page as unknown as Page;
}

async function sinkShot(
  state: MarkerState,
  options: { fullPage?: boolean; marker?: string } = {},
): Promise<{ file: string; sha256: string }> {
  const outDir = await mkdtemp(join(tmpdir(), "shots-capture-test-"));
  const sink = createSink(fakePage(state), {
    outDir,
    scenario: "home",
    title: "home",
    route: "/home",
    viewport: "desktop",
    locale: "ru-RU",
    baseURL: "http://localhost:1420",
    cdnBaseUrl: "http://localhost:8080",
    commit: "abc123",
  });
  const entry = await sink.shot("home", {
    marker: options.marker ?? "home-page",
    fullPage: options.fullPage,
  });
  const png = entry.file === "" ? null : await readFile(entry.file);
  return {
    file: entry.file,
    sha256: png === null ? "" : (readProof(png)?.commit ?? ""),
  };
}

test("a settled marker produces a stamped frame", async () => {
  const result = await sinkShot({});
  assert.notEqual(result.file, "");
  assert.equal(result.sha256, "abc123");
});

test("a marker below the fold produces a negative entry", async () => {
  const result = await sinkShot({ top: 900, bottom: 1180 });
  assert.equal(result.file, "");
});

test("fullPage accepts a marker below the viewport", async () => {
  const result = await sinkShot({ top: 900, bottom: 1180 }, { fullPage: true });
  assert.notEqual(result.file, "");
});

test("a fading splash produces a negative entry", async () => {
  const result = await sinkShot({ splashAttached: true });
  assert.equal(result.file, "");
});

test("a missing marker produces a negative entry", async () => {
  const result = await sinkShot({ count: 0 });
  assert.equal(result.file, "");
});

test("a css= selector is resolved as a locator", async () => {
  const result = await sinkShot(
    {},
    {
      marker: "css=canvas.kanji-drawing-canvas",
    },
  );
  assert.equal(result.sha256, "abc123");
});
