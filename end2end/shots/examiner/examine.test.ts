import assert from "node:assert/strict";
import { mkdtemp, writeFile } from "node:fs/promises";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { test } from "node:test";

import { D3Error } from "./d3.ts";
import { createExaminerFromEnv, isContextOverflow, mapVerdicts, parsePositiveInt, withRetry } from "./examine.ts";
import { collectUnanswered } from "./report.ts";

const QUESTIONS = [
    { kind: "noul", id: "q-flagged", label: "Флаг", instructions: "i", criteria: { true: "t", false: "f" }, threshold: 0.75 },
    { kind: "noul", id: "q-calm", label: "Спокойно", instructions: "i", criteria: { true: "t", false: "f" }, threshold: 0.75 },
    { kind: "noul", id: "q-silent", label: "Молчание", instructions: "i", criteria: { true: "t", false: "f" }, threshold: 0.75 },
] as const;

test("mapVerdicts flags above the threshold and keeps the rest calm", () => {
    const answers = new Map([
        ["q-flagged", { kind: "noul" as const, pTrue: 0.91 }],
        ["q-calm", { kind: "noul" as const, pTrue: 0.2 }],
    ]);
    const verdicts = mapVerdicts(QUESTIONS, answers);
    assert.deepEqual(
        verdicts.map((verdict) => [verdict.questionId, verdict.flagged]),
        [["q-flagged", true], ["q-calm", false], ["q-silent", false]],
    );
    // The unanswered question is never flagged and keeps pTrue null.
    assert.equal(verdicts[2].kind, "noul");
    assert.equal((verdicts[2] as { pTrue: number | null }).pTrue, null);
});

test("unanswered verdicts surface in the report data", () => {
    const verdicts = mapVerdicts(QUESTIONS, new Map([["q-calm", { kind: "noul" as const, pTrue: 0.1 }]]));
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

test("mapVerdicts flags a choice question when the winner is unexpected", () => {
    const questions = [
        {
            kind: "choice",
            id: "stage",
            label: "Стадия урока",
            instructions: "Which stage?",
            options: { presentation: "p", complete: "c" },
        },
    ] as const;
    const answers = new Map([["stage", { kind: "choice" as const, choiceId: "complete", probabilities: { presentation: 0.2, complete: 0.8 } }]]);

    const pass = mapVerdicts(questions, answers, { stage: ["complete"] });
    assert.equal(pass[0].kind, "choice");
    assert.equal((pass[0] as { flagged: boolean }).flagged, false);

    const fail = mapVerdicts(questions, answers, { stage: ["presentation"] });
    assert.equal((fail[0] as { flagged: boolean }).flagged, true);
    assert.equal((fail[0] as { actual: string | null }).actual, "complete");
    // pExpected is the probability of the expected option, not the winner.
    assert.equal((fail[0] as { pExpected: number | null }).pExpected, 0.2);
});

test("a choice question without an expectation is never flagged", () => {
    const questions = [
        { kind: "choice" as const, id: "stage", label: "Стадия", instructions: "Which?", options: { a: "a", b: "b" } },
    ];
    const answers = new Map([["stage", { kind: "choice" as const, choiceId: "b", probabilities: { a: 0.1, b: 0.9 } }]]);
    const verdicts = mapVerdicts(questions, answers);
    assert.equal(verdicts[0].kind, "choice");
    assert.equal((verdicts[0] as { flagged: boolean }).flagged, false);
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
