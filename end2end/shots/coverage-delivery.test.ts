import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { readFile } from "node:fs/promises";
import { join } from "node:path";
import { test } from "node:test";

import { matchesScreen } from "./coverage-checks.ts";
import { buildStaticInput } from "./coverage-input.ts";
import { inventory } from "./coverage-inventory.ts";
import { repoRoot } from "./config.ts";
import { repoImageName, readPngSize } from "./manifest.ts";
import { readRegistry, type RegistryShot } from "./registry.ts";
import { VIEWPORTS, type ViewportTag } from "./viewports.ts";

const imagesDir = join(repoRoot, "docs/ui-screenshots/images");
const registryDir = join(repoRoot, "docs/ui-screenshots/shots");

/** The store's own naming rule, with the registry's string viewport narrowed. */
function imageName(entry: RegistryShot): string {
  return repoImageName({ ...entry, viewport: entry.viewport as ViewportTag });
}

/** Frames the committed inventory claims as proof of its covered rows. */
function proofFrames(registry: readonly RegistryShot[]): RegistryShot[] {
  const frames: RegistryShot[] = [];
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
        frames.push(found);
      }
    }
  }
  return frames;
}

/**
 * The inventory is a promise: a row marked `covered` must be backed by a
 * delivered frame the reader can open. The registry and the PNG store are
 * the evidence the PR carries, so this test reads them as a user would.
 */
test("every covered row has a delivered ru-RU frame pair", async () => {
  const registry = await readRegistry(registryDir);

  for (const found of proofFrames(registry)) {
    let png: Buffer;
    try {
      png = await readFile(join(imagesDir, imageName(found)));
    } catch (error) {
      if ((error as NodeJS.ErrnoException).code === "ENOENT") {
        assert.fail(`registered PNG is missing: ${imageName(found)}`);
      }
      throw error;
    }
    assert.equal(
      png.length,
      found.bytes,
      `PNG size differs from the registry: ${imageName(found)}`,
    );
    const size = readPngSize(png);
    assert.deepEqual(
      size,
      { width: found.width, height: found.height },
      `PNG dimensions differ from the registry: ${imageName(found)}`,
    );
    if (!found.fullPage) {
      const viewport = VIEWPORTS[found.viewport as keyof typeof VIEWPORTS];
      assert.ok(viewport, `unknown viewport: ${found.viewport}`);
      assert.equal(
        size.width,
        viewport.width,
        `frame does not match its viewport: ${imageName(found)}`,
      );
    }
  }
});

test("every delivered frame matches the sha256 in its registry entry", async () => {
  const registry = await readRegistry(registryDir);
  const required = new Set(proofFrames(registry).map(imageName));
  let checked = 0;

  for (const entry of registry) {
    let png: Buffer;
    try {
      png = await readFile(join(imagesDir, imageName(entry)));
    } catch (error) {
      // Older entries can name frames that were never put into the store.
      // A read failure that is not "absent" must never pass silently.
      if ((error as NodeJS.ErrnoException).code === "ENOENT") continue;
      throw error;
    }
    assert.equal(
      createHash("sha256").update(png).digest("hex"),
      entry.sha256,
      `sha256 differs for ${imageName(entry)}`,
    );
    checked++;
  }
  assert.ok(
    checked >= required.size,
    `checked ${checked} frames, but covered rows require ${required.size}`,
  );
});

test("every published link points at the frame it describes", async () => {
  const registry = await readRegistry(registryDir);

  for (const entry of registry.filter((item) => item.url !== null)) {
    assert.equal(
      entry.uploadedSha256,
      entry.sha256,
      `published hash differs from the frame: ${imageName(entry)}`,
    );
    assert.equal(
      entry.url?.split("/").pop(),
      `${entry.scenario}--${entry.screen}--${entry.viewport}-${entry.sha256.slice(0, 8)}.png`,
      `CDN key does not carry the current sha8: ${imageName(entry)}`,
    );
  }
});

test("scenario-less variations stay exclusions, never coverage", async () => {
  const input = await buildStaticInput();
  const registry = await readRegistry(registryDir);

  // The five rows listed in uwuwu#1691 carry code `no-scenario`. When
  // #1736 adds a scenario for one of them, an exam key or a captured
  // screen appears and this test fails: the exclusion must be revisited.
  const scenarioLess = inventory.filter(
    (row) => row.exclusion?.code === "no-scenario",
  );
  assert.ok(
    scenarioLess.length > 0,
    "no no-scenario rows left in the inventory",
  );

  for (const row of scenarioLess) {
    assert.equal(row.status, "excluded", `${row.id} must not claim coverage`);
    assert.equal(row.scenario, null, `${row.id} has a scenario now`);
    assert.ok(row.exclusion?.reason, `${row.id} has no exclusion reason`);

    const examined = input.examKeys.filter((key) => {
      const screen = key.slice(key.indexOf(":") + 1);
      return row.screens.some((pattern) => matchesScreen(pattern, screen));
    });
    assert.deepEqual(examined, [], `${row.id} now has an examined screen`);
    const captured = registry
      .filter((entry) =>
        row.screens.some((pattern) => matchesScreen(pattern, entry.screen)),
      )
      .map(imageName);
    assert.deepEqual(captured, [], `${row.id} now has a captured screen`);
  }
});
