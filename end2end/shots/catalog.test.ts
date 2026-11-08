import assert from "node:assert/strict";
import { test } from "node:test";

import { descriptors, listDescriptors } from "./catalog.ts";
import { PRESET_IDS } from "./presets/registry.ts";

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

test("no catalog entry restricts viewports — every scenario captures the pair", () => {
    for (const descriptor of listDescriptors()) {
        assert.ok(
            !("viewports" in descriptor),
            `${descriptor.id} restricts viewports — the desktop+mobile pair is the policy, ` +
                `a per-scenario restriction lives only in the run (--viewports), not the catalog`,
        );
    }
});

test("the catalog is listed in a stable order", () => {
    const ids = listDescriptors().map((descriptor) => descriptor.id);

    assert.deepEqual(ids, [...ids].sort());
});

test("the catalog is not empty", () => {
    assert.ok(listDescriptors().length > 0);
});
