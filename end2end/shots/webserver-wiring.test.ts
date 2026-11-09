import assert from "node:assert/strict";
import { mkdtempSync, rmSync, existsSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { test } from "node:test";

import { APP_SERVER_TIMEOUT, DEFAULT_PROD_CDN_URL } from "./config.ts";

/**
 * Wiring pins for the two Playwright runners.
 *
 * The постановка of the harness fix: both webServer lists must serve the
 * corpus directory resolved by `resolveCdnSource()` (never the hardcoded
 * relative `../cdn`), start the corpus server only when a corpus exists,
 * and give the app server the shared cold-build-aware startup budget.
 * CI runs with a corpus at the repo-local default, so a silent revert to
 * the hardcoded path would stay green there — these pins fail on it.
 *
 * The config modules read `process.env` at import time, and Node caches
 * a module instance per URL: each case imports the config through a
 * unique query string to get a fresh evaluation against the case env.
 */

interface WebServerEntry {
  command: string;
  timeout?: number;
  env?: Record<string, string>;
}

interface ConfigShape {
  webServer: WebServerEntry[];
}

type EnvSnapshot = Map<(typeof ENV_KEYS)[number], string | undefined>;

const ENV_KEYS = [
  "ORIGA_CDN_CACHE_DIR",
  "E2E_CDN_PORT",
  "SHOT_CDN_BASE_URL",
  "E2E_APP_SERVER_TIMEOUT",
] as const;

function caseEnv(corpusDir: string | null): EnvSnapshot {
  const saved: EnvSnapshot = new Map(
    ENV_KEYS.map((key) => [key, process.env[key]]),
  );
  process.env.ORIGA_CDN_CACHE_DIR =
    corpusDir ?? join(tmpdir(), "origa-no-such-corpus-pin");
  delete process.env.E2E_CDN_PORT;
  delete process.env.SHOT_CDN_BASE_URL;
  delete process.env.E2E_APP_SERVER_TIMEOUT;
  return saved;
}

function restoreEnv(saved: EnvSnapshot): void {
  for (const [key, value] of saved) {
    if (value === undefined) {
      delete process.env[key];
    } else {
      process.env[key] = value;
    }
  }
}

function load(spec: string): Promise<ConfigShape> {
  return import(spec).then(
    (mod) => mod.default,
  ) as unknown as Promise<ConfigShape>;
}

function assertNoHardcodedCdnPath(entries: WebServerEntry[]): void {
  for (const entry of entries) {
    assert.ok(
      !entry.command.includes("../cdn"),
      `webServer command must not hardcode ../cdn: ${entry.command}`,
    );
  }
}

test("both runners serve the resolved corpus dir with the shared timeout", async () => {
  const corpus = mkdtempSync(join(tmpdir(), "origa-corpus-pin-"));
  const token = caseEnv(corpus);
  try {
    assert.ok(existsSync(corpus));
    const expectedPort = 8080; // E2E_CDN_PORT removed — the documented default

    const bdd = (await load("../playwright.config.ts?pin-corpus-bdd"))
      .webServer;
    assert.equal(bdd.length, 2, "BDD runner: corpus server + app server");
    assert.ok(
      bdd[0].command.includes(`serve "${corpus}"`),
      `BDD CDN command: ${bdd[0].command}`,
    );
    const bddApp = bdd.at(-1);
    assert.equal(bddApp?.timeout, APP_SERVER_TIMEOUT);
    assert.equal(
      bddApp?.env?.ORIGA_CDN_BASE_URL,
      `http://localhost:${expectedPort}`,
    );
    assertNoHardcodedCdnPath(bdd);

    const shots = (
      await load("../playwright.screenshots.config.ts?pin-corpus-shots")
    ).webServer;
    assert.equal(shots.length, 2, "shots runner: corpus server + app server");
    assert.ok(
      shots[0].command.includes(`serve "${corpus}"`),
      `shots CDN command: ${shots[0].command}`,
    );
    const shotsApp = shots.at(-1);
    assert.equal(shotsApp?.timeout, APP_SERVER_TIMEOUT);
    assert.equal(
      shotsApp?.env?.ORIGA_CDN_BASE_URL,
      `http://localhost:${expectedPort}`,
    );
    assertNoHardcodedCdnPath(shots);
  } finally {
    rmSync(corpus, { recursive: true, force: true });
    restoreEnv(token);
  }
});

test("both runners skip the corpus server and fall back to the prod CDN without a corpus", async () => {
  const token = caseEnv(null); // nonexistent ORIGA_CDN_CACHE_DIR → serveLocally=false
  try {
    const bdd = (await load("../playwright.config.ts?pin-nocorpus-bdd"))
      .webServer;
    assert.equal(bdd.length, 1, "BDD runner: only the app server starts");
    const bddApp = bdd[0];
    assert.equal(
      bddApp.command.includes("serve"),
      true,
      "app server still starts",
    );
    assert.equal(bddApp.timeout, APP_SERVER_TIMEOUT);
    assert.equal(bddApp.env?.ORIGA_CDN_BASE_URL, DEFAULT_PROD_CDN_URL);
    assertNoHardcodedCdnPath(bdd);

    const shots = (
      await load("../playwright.screenshots.config.ts?pin-nocorpus-shots")
    ).webServer;
    assert.equal(shots.length, 1, "shots runner: only the app server starts");
    assert.equal(shots[0].timeout, APP_SERVER_TIMEOUT);
    assert.equal(shots[0].env?.ORIGA_CDN_BASE_URL, DEFAULT_PROD_CDN_URL);
    assertNoHardcodedCdnPath(shots);
  } finally {
    restoreEnv(token);
  }
});
