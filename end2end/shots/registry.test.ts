import assert from "node:assert/strict";
import { mkdtemp, rm } from "node:fs/promises";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { after, before, test } from "node:test";

import type { ShotEntry } from "./manifest.ts";
import { mergeEntry, readRegistry } from "./registry.ts";

let directory: string;

before(async () => {
    directory = await mkdtemp(join(tmpdir(), "origa-registry-"));
});

after(async () => {
    await rm(directory, { recursive: true, force: true });
});

function entry(overrides: Partial<ShotEntry> = {}): ShotEntry {
    return {
        scenario: "home",
        title: "Главный экран",
        screen: "home",
        route: "/home",
        viewport: "desktop",
        locale: "ru-RU",
        width: 1280,
        height: 800,
        fullPage: false,
        bytes: 1000,
        sha256: "a".repeat(64),
        file: "/tmp/opencode/shots/home--home--desktop.png",
        capturedAt: "2026-10-21T10:00:00.000Z",
        commit: "abcdef1234567890",
        baseURL: "http://localhost:1420",
        cdnBaseUrl: "http://localhost:8080",
        ...overrides,
    };
}

test("a capture is written into the registry and read back", async () => {
    await mergeEntry(directory, entry());

    const entries = await readRegistry(directory);

    assert.equal(entries.length, 1);
    assert.equal(entries[0].scenario, "home");
    assert.equal(entries[0].sha256, "a".repeat(64));
});

test("a second capture of the same screen replaces the first one", async () => {
    await mergeEntry(directory, entry({ sha256: "b".repeat(64), capturedAt: "2026-10-22T10:00:00.000Z" }));

    const entries = await readRegistry(directory);

    assert.equal(entries.length, 1);
    assert.equal(entries[0].sha256, "b".repeat(64));
});

test("a capture of another viewport is a separate entry", async () => {
    await mergeEntry(directory, entry({ viewport: "mobile" }));

    const entries = await readRegistry(directory);

    assert.equal(entries.length, 2);
});

test("a published capture keeps its link", async () => {
    await mergeEntry(directory, entry(), {
        url: "https://s3.origa.uwuwu.net/pr-screenshots/x/home.png",
        sha256: "a".repeat(64),
        uploadedAt: "2026-10-21T10:00:05.000Z",
    });

    const entries = await readRegistry(directory);
    const published = entries.find((candidate) => candidate.viewport === "desktop");

    assert.equal(published?.url, "https://s3.origa.uwuwu.net/pr-screenshots/x/home.png");
    assert.equal(published?.uploadedSha256, "a".repeat(64));
});

test("a capture that was not published keeps the link of the previous upload", async () => {
    await mergeEntry(directory, entry({ sha256: "c".repeat(64), capturedAt: "2026-10-23T10:00:00.000Z" }));

    const entries = await readRegistry(directory);
    const published = entries.find((candidate) => candidate.viewport === "desktop");

    assert.equal(published?.sha256, "c".repeat(64));
    assert.equal(published?.url, "https://s3.origa.uwuwu.net/pr-screenshots/x/home.png");
});
