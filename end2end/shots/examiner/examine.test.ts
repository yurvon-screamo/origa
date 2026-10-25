import assert from "node:assert/strict";
import { mkdtemp, writeFile } from "node:fs/promises";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { test } from "node:test";

import { D3Error } from "./d3.ts";
import { createExaminerFromEnv, isContextOverflow, mapVerdicts, parsePositiveInt, withRetry } from "./examine.ts";
import { collectUnanswered } from "./report.ts";

const QUESTIONS = [
    { id: "q-flagged", label: "Флаг", threshold: 0.75 },
    { id: "q-calm", label: "Спокойно", threshold: 0.75 },
    { id: "q-silent", label: "Молчание", threshold: 0.75 },
] as const;

test("mapVerdicts flags above the threshold and keeps the rest calm", () => {
    const answers = new Map([
        ["q-flagged", { pTrue: 0.91 }],
        ["q-calm", { pTrue: 0.2 }],
    ]);
    const verdicts = mapVerdicts(QUESTIONS, answers);
    assert.deepEqual(
        verdicts.map((verdict) => [verdict.questionId, verdict.flagged]),
        [["q-flagged", true], ["q-calm", false], ["q-silent", false]],
    );
    // The unanswered question is never flagged and keeps pTrue null.
    assert.equal(verdicts[2].pTrue, null);
});

test("unanswered verdicts surface in the report data", () => {
    const verdicts = mapVerdicts(QUESTIONS, new Map([["q-calm", { pTrue: 0.1 }]]));
    const unanswered = collectUnanswered([
        {
            capturedAt: "now",
            screen: { scenario: "home", screen: "home", viewport: "desktop", locale: "ru-RU", route: "/home", file: "/tmp/x.png", sha256: "a", width: 1, height: 1, fullPage: false },
            examiner: { model: "d3:2b", checklist: "ck", numCtx: 16384, threshold: 0.75 },
            dom: { chars: 1, nodes: 1, truncatedNodes: 0 },
            usage: null,
            latencyMs: 1,
            verdicts,
            contextOverflow: false,
            error: null,
        },
    ]);
    assert.equal(unanswered.length, 2);
    assert.deepEqual(unanswered.map((item) => item.questionId).sort(), ["q-flagged", "q-silent"]);
});

test("isContextOverflow trips at 90% of the window", () => {
    assert.equal(isContextOverflow({ inputTokens: 15_000, outputTokens: 0 }, 16_384), true);
    assert.equal(isContextOverflow({ inputTokens: 14_000, outputTokens: 0 }, 16_384), false);
    assert.equal(isContextOverflow(null, 16_384), false);
});

test("withRetry retries a D3Error once and passes the second attempt through", async () => {
    let attempts = 0;
    const result = await withRetry(async () => {
        attempts += 1;
        if (attempts === 1) throw new D3Error("transient");
        return { answers: new Map(), usage: null };
    });
    assert.equal(attempts, 2);
    assert.equal(result.answers.size, 0);
});

test("withRetry gives up after the second D3Error", async () => {
    await assert.rejects(
        withRetry(async () => {
            throw new D3Error("down");
        }),
        D3Error,
    );
});

test("withRetry does not retry non-model errors", async () => {
    let attempts = 0;
    await assert.rejects(
        withRetry(async () => {
            attempts += 1;
            throw new Error("programmer error");
        }),
        Error,
    );
    assert.equal(attempts, 1);
});

test("parsePositiveInt rejects garbage instead of becoming NaN", () => {
    assert.equal(parsePositiveInt("N", "2048"), 2048);
    assert.throws(() => parsePositiveInt("N", "big"), /целым положительным/);
    assert.throws(() => parsePositiveInt("N", "-5"), /целым положительным/);
});

test("createExaminerFromEnv returns null without SHOT_EXAMINE", async () => {
    assert.equal(await createExaminerFromEnv({}, "/tmp"), null);
    assert.equal(await createExaminerFromEnv({ SHOT_EXAMINE: "0" }, "/tmp"), null);
});

test("createExaminerFromEnv builds an examiner from a valid model file", async () => {
    const dir = await mkdtemp(join(tmpdir(), "exam-env-"));
    const modelPath = join(dir, "model.json");
    await writeFile(
        modelPath,
        JSON.stringify({ version: 1, defaults: { threshold: 0.75 }, screens: { "home:home": { questions: ["text-overlap"] } } }),
        "utf-8",
    );
    const examiner = await createExaminerFromEnv(
        { SHOT_EXAMINE: "1", SHOT_EXAM_MODEL: "d3:4b", SHOT_EXAM_MODEL_JSON: modelPath, SHOT_EXAM_NUM_CTX: "8192" },
        dir,
    );
    assert.notEqual(examiner, null);
});

test("createExaminerFromEnv fails loudly on a garbage num_ctx", async () => {
    const dir = await mkdtemp(join(tmpdir(), "exam-env-"));
    await assert.rejects(
        createExaminerFromEnv({ SHOT_EXAMINE: "1", SHOT_EXAM_NUM_CTX: "huge" }, dir),
        /целым положительным/,
    );
});
