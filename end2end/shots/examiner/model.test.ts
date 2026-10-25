import assert from "node:assert/strict";
import { join } from "node:path";
import { test } from "node:test";

import { descriptors } from "../catalog.ts";
import { loadExamModel, parseExamModel, questionsForScreen, screenKey } from "./model.ts";
import { findQuestion } from "./questions.ts";

const VALID = {
    version: 1,
    defaults: { threshold: 0.75 },
    screens: {
        "home:home": { questions: ["text-overlap"] },
    },
};

test("a valid model parses", () => {
    const model = parseExamModel(VALID);
    assert.equal(model.defaults.threshold, 0.75);
    assert.deepEqual(model.screens["home:home"].questions, ["text-overlap"]);
});

test("an unknown version is rejected", () => {
    assert.throws(() => parseExamModel({ ...VALID, version: 2 }));
});

test("an unknown question id fails loudly instead of silently examining nothing", () => {
    assert.throws(
        () => parseExamModel({ ...VALID, screens: { "home:home": { questions: ["no-such-question"] } } }),
        /unknown question/,
    );
});

test("a malformed screen key is rejected", () => {
    assert.throws(() => parseExamModel({ ...VALID, screens: { "home": { questions: ["text-overlap"] } } }));
});

test("an out-of-range threshold is rejected", () => {
    assert.throws(() =>
        parseExamModel({ ...VALID, screens: { "home:home": { questions: ["text-overlap"], threshold: 1.5 } } }),
    );
    assert.throws(() => parseExamModel({ ...VALID, defaults: { threshold: 0 } }));
});

test("a screen outside the model is a skip, not an error", () => {
    const model = parseExamModel(VALID);
    assert.equal(questionsForScreen(model, "home", "other-screen"), null);
    assert.equal(questionsForScreen(model, "other", "home"), null);
});

test("a per-screen threshold is rejected — thresholds are calibrated per question", () => {
    assert.throws(
        () => parseExamModel({ ...VALID, screens: { "home:home": { questions: ["text-overlap"], threshold: 0.9 } } }),
        /thresholds belong to the checklist/,
    );
});

test("the committed model is valid and names known scenarios", async () => {
    const path = join(import.meta.dirname, "model.json");
    const model = await loadExamModel(path);

    const known = new Set(Object.keys(descriptors));
    for (const key of Object.keys(model.screens)) {
        const scenario = key.split(":")[0];
        assert.ok(
            known.has(scenario),
            `model.json examines "${scenario}" which is not in the catalog — a typo silently examines nothing`,
        );
    }
    // Every screen asks only canary-proven questions: the measured blind
    // spots of the current d3 builds (wrong-typeface, text-clipped,
    // layout-misaligned — see exam-canary.spec.ts) stay out until a model
    // or formulation clears its canary.
    const blindSpots = ["wrong-typeface", "text-clipped", "layout-misaligned"];
    for (const exam of Object.values(model.screens)) {
        for (const blind of blindSpots) {
            assert.ok(!exam.questions.includes(blind), `${blind} is a measured d3 blind spot — enable only with a passing canary`);
        }
        assert.ok(exam.questions.length >= 3, `screen asks only ${exam.questions.length} questions`);
        for (const id of exam.questions) {
            assert.ok(findQuestion(id) !== undefined, `unknown question "${id}"`);
        }
    }
});

test("screenKey joins scenario and screen names", () => {
    assert.equal(screenKey("home", "due-counter"), "home:due-counter");
});
