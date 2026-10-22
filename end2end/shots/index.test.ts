import assert from "node:assert/strict";
import { test } from "node:test";

import { renderIndex } from "./index.ts";
import type { RegistryShot } from "./registry.ts";

function shot(overrides: Partial<RegistryShot> = {}): RegistryShot {
    return {
        schema: 1,
        scenario: "home",
        screen: "home",
        route: "/home",
        viewport: "desktop",
        locale: "ru-RU",
        width: 1280,
        height: 800,
        fullPage: false,
        bytes: 1000,
        sha256: "a".repeat(64),
        capturedAt: "2026-10-21T10:00:00.000Z",
        commit: "abcdef1234567890",
        baseURL: "http://localhost:1420",
        cdnBaseUrl: "http://localhost:8080",
        url: "https://s3.origa.uwuwu.net/pr-screenshots/x/home.png",
        uploadedSha256: "a".repeat(64),
        uploadedAt: "2026-10-21T10:00:05.000Z",
        ...overrides,
    };
}

test("the index marks itself as generated", () => {
    const index = renderIndex([shot()]);

    assert.match(index, /Сгенерировано/);
    assert.match(index, /Не редактировать вручную/);
});

test("the index links a published capture", () => {
    const index = renderIndex([shot()]);

    assert.match(index, /\[png\]\(https:\/\/s3\.origa\.uwuwu\.net\/pr-screenshots\/x\/home\.png\)/);
});

test("the index says a capture that was not published", () => {
    const index = renderIndex([shot({ url: null, uploadedSha256: null, uploadedAt: null })]);

    assert.match(index, /не загружен/);
});

test("the newest capture comes first", () => {
    const index = renderIndex([
        shot({ screen: "old", capturedAt: "2026-10-01T10:00:00.000Z" }),
        shot({ screen: "new", capturedAt: "2026-10-20T10:00:00.000Z" }),
    ]);

    assert.ok(index.indexOf("| new |") < index.indexOf("| old |"));
});

test("an empty registry still renders a table", () => {
    const index = renderIndex([]);

    assert.match(index, /\| Экран \|/);
});
