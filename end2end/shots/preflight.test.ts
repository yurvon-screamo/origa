import assert from "node:assert/strict";
import { test } from "node:test";

import { evaluatePreflight, type PreflightFacts } from "./preflight.ts";

function facts(overrides: Partial<PreflightFacts> = {}): PreflightFacts {
    return {
        serveMode: "build",
        nodeModulesPresent: true,
        fontsReady: true,
        dictPresent: true,
        distPoisoned: false,
        corpusProbe: { path: "dictionaries/JmdictFurigana.rkyv", present: true },
        listeners: [],
        appBaseUrl: "http://localhost:1420",
        cdnBaseUrl: "http://localhost:8080",
        cdnCacheDir: "/repo/cdn",
        repoRoot: "/repo",
        awsProfilePresent: null,
        trailBaseUrlConfigured: true,
        adminPasswordConfigured: true,
        outDirInsideRepo: false,
        ...overrides,
    };
}

test("a ready environment produces no errors", () => {
    const report = evaluatePreflight(facts());

    assert.deepEqual(report.errors, []);
});

test("a missing corpus font fails the capture", () => {
    const report = evaluatePreflight(facts({ fontsReady: false }));

    assert.equal(report.errors.length, 1);
    assert.match(report.errors[0], /cdn\/fonts\//);
});

test("a missing tokenizer dictionary only warns", () => {
    const report = evaluatePreflight(facts({ dictPresent: false }));

    assert.deepEqual(report.errors, []);
    assert.equal(report.warnings.length, 1);
});

test("a stale corpus only warns", () => {
    const report = evaluatePreflight(facts({ corpusProbe: { path: "grammar/x.json", present: false } }));

    assert.deepEqual(report.errors, []);
    assert.match(report.warnings[0], /отстаёт/);
});

test("a poisoned bundle fails the capture", () => {
    const report = evaluatePreflight(facts({ serveMode: "dist", distPoisoned: true }));

    assert.equal(report.errors.length, 1);
    assert.match(report.errors[0], /trunk build --release/);
});

test("an output directory inside the repository fails the capture", () => {
    const report = evaluatePreflight(facts({ outDirInsideRepo: true }));

    assert.match(report.errors[0], /внутри репозитория/);
});

test("a missing node_modules fails the capture", () => {
    const report = evaluatePreflight(facts({ nodeModulesPresent: false }));

    assert.match(report.errors[0], /npm ci/);
});

test("a missing e2e configuration fails the capture before the browser starts", () => {
    const report = evaluatePreflight(
        facts({ trailBaseUrlConfigured: false, adminPasswordConfigured: false }),
    );

    assert.equal(report.errors.length, 2);
    assert.match(report.errors[0], /TRAILBASE_URL/);
    assert.match(report.errors[1], /ORIGA_ADMIN_PASSWORD/);
});

test("a missing upload profile fails the capture before it starts", () => {
    const report = evaluatePreflight(facts({ awsProfilePresent: false }));

    assert.match(report.errors[0], /origa-t3/);
});

test("a listener from another checkout warns about a zombie worktree", () => {
    const report = evaluatePreflight(
        facts({ listeners: [{ port: 1420, pid: 4242, cwd: "/home/other/worktree" }] }),
    );

    assert.match(report.warnings[0], /зомби-ворктри/);
});

test("a listener of the same checkout is not reported", () => {
    const report = evaluatePreflight(
        facts({ listeners: [{ port: 1420, pid: 4242, cwd: "/repo" }] }),
    );

    assert.deepEqual(report.warnings, []);
});
