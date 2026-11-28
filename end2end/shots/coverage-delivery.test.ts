import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { readFile } from "node:fs/promises";
import { join } from "node:path";
import { test } from "node:test";

import { matchesScreen } from "./coverage-checks.ts";
import { inventory } from "./coverage-inventory.ts";
import { repoRoot } from "./config.ts";
import { readRegistry, type RegistryShot } from "./registry.ts";

const imagesDir = join(repoRoot, "docs/ui-screenshots/images");
const registryDir = join(repoRoot, "docs/ui-screenshots/shots");

function imageName(entry: RegistryShot): string {
  return `${entry.scenario}--${entry.screen}--${entry.viewport}--${entry.locale}.png`;
}

/**
 * The inventory is a promise: a row marked `covered` must be backed by a
 * delivered frame the reader can open. The registry and the PNG store are
 * the evidence the PR carries, so this test reads them as a user would.
 */
test("every covered row has a delivered ru-RU frame pair", async () => {
  const registry = await readRegistry(registryDir);

  for (const row of inventory.filter((item) => item.status === "covered")) {
    assert.ok(row.scenario, `${row.id} has no scenario`);
    for (const viewport of row.viewports) {
      for (const screen of row.screens) {
        const found = registry.find(
          (entry) =>
            entry.scenario === row.scenario &&
            entry.viewport === viewport &&
            entry.locale === "ru-RU" &&
            matchesScreen(screen, entry.screen),
        );
        assert.ok(
          found,
          `${row.id}: no ${viewport} ru-RU frame in the registry for "${screen}"`,
        );

        let png: Buffer;
        try {
          png = await readFile(join(imagesDir, imageName(found)));
        } catch {
          assert.fail(
            `${row.id}: registered PNG is missing: ${imageName(found)}`,
          );
        }
        assert.equal(
          png.length,
          found.bytes,
          `${row.id}: PNG size differs from the registry`,
        );
      }
    }
  }
});

test("every delivered frame matches the sha256 in its registry entry", async () => {
  const registry = await readRegistry(registryDir);

  for (const entry of registry) {
    let png: Buffer;
    try {
      png = await readFile(join(imagesDir, imageName(entry)));
    } catch {
      // Older entries can name frames that were never put into the store.
      // Only a frame that is present can be checked for integrity.
      continue;
    }
    assert.equal(
      createHash("sha256").update(png).digest("hex"),
      entry.sha256,
      `sha256 differs for ${imageName(entry)}`,
    );
  }
});

test("scenario-less variations stay exclusions, never coverage", () => {
  const scenarioLess = [
    "variation:lesson-card-reversed",
    "variation:lesson-card-grammar-mutated",
    "variation:lesson-grammar-quiz",
    "variation:words-filters-inbox",
    "variation:sets-import-empty",
  ];
  for (const id of scenarioLess) {
    const row = inventory.find((item) => item.id === id);
    assert.ok(row, `${id} is missing from the inventory`);
    assert.equal(row.status, "excluded", `${id} must not claim coverage`);
    assert.equal(row.exclusion?.code, "no-scenario");
    assert.ok(row.exclusion?.reason, `${id} has no exclusion reason`);
  }
});
