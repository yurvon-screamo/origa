import assert from "node:assert/strict";
import { test } from "node:test";

import { descriptors, listDescriptors } from "./catalog.ts";
import { PRESET_IDS } from "./presets/registry.ts";
import { VIEWPORTS } from "./viewports.ts";

test("every catalog entry is addressed by its own id", () => {
    for (const [key, descriptor] of Object.entries(descriptors)) {
        assert.equal(key, descriptor.id);
    }
});

test("every catalog entry names a known data state", () => {
    for (const descriptor of listDescriptors()) {
        assert.ok(
            (PRESET_IDS as readonly string[]).includes(descriptor.preset),
            `${descriptor.id} names an unknown preset "${descriptor.preset}"`,
        );
    }
});

test("every catalog entry names known viewports", () => {
    for (const descriptor of listDescriptors()) {
        for (const viewport of descriptor.viewports ?? []) {
            assert.ok(viewport in VIEWPORTS, `${descriptor.id} names an unknown viewport`);
        }
    }
});

test("the catalog is listed in a stable order", () => {
    const ids = listDescriptors().map((descriptor) => descriptor.id);

    assert.deepEqual(ids, [...ids].sort());
});

test("the catalog is not empty", () => {
    assert.ok(listDescriptors().length > 0);
});
