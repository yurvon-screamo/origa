import assert from "node:assert/strict";
import { test } from "node:test";

import type { ExamRecord } from "./examine.ts";
import { collectFindings, findingFingerprint, renderIssueDraft, renderReport } from "./report.ts";

function record(overrides: Partial<ExamRecord> = {}): ExamRecord {
    return {
        capturedAt: "2026-10-25T10:00:00.000Z",
        screen: {
            scenario: "home",
            screen: "home",
            viewport: "desktop",
            locale: "ru-RU",
            route: "/home",
            file: "/tmp/opencode/shots/rc/home--home--desktop--ru-RU.png",
            sha256: "abc123",
            width: 1280,
            height: 800,
            fullPage: false,
        },
        examiner: { model: "d3:2b", checklist: "ck", numCtx: 16384, threshold: 0.75 },
        dom: { chars: 100, nodes: 10, truncatedNodes: 0 },
        usage: { inputTokens: 1500, outputTokens: 0 },
        latencyMs: 4200,
        verdicts: [
            { questionId: "text-overlap", label: "Наложение текста или элементов", pTrue: 0.91, threshold: 0.75, flagged: true },
            { questionId: "tofu-glyphs", label: "Битые глифы (тофу)", pTrue: 0.2, threshold: 0.75, flagged: false },
            { questionId: "broken-state", label: "Пустой или сломанный экран", pTrue: null, threshold: 0.75, flagged: false },
        ],
        contextOverflow: false,
        error: null,
        ...overrides,
    };
}

test("only flagged verdicts become findings", () => {
    const findings = collectFindings([record()]);
    assert.equal(findings.length, 1);
    assert.equal(findings[0].questionId, "text-overlap");
    assert.equal(findings[0].pTrue, 0.91);
});

test("the fingerprint ignores everything but screen and question", () => {
    const left = findingFingerprint({ scenario: "home", screen: "home", viewport: "desktop", locale: "ru-RU", questionId: "text-overlap" });
    const right = findingFingerprint({ scenario: "home", screen: "home", viewport: "desktop", locale: "ru-RU", questionId: "text-overlap" });
    assert.equal(left, right);
    assert.notEqual(
        left,
        findingFingerprint({ scenario: "home", screen: "home", viewport: "mobile", locale: "ru-RU", questionId: "text-overlap" }),
    );
});

test("two runs of the same defect produce the same fingerprint (issue dedup)", () => {
    const first = collectFindings([record()]);
    const second = collectFindings([
        record({
            capturedAt: "2026-10-26T10:00:00.000Z",
            screen: { ...record().screen, sha256: "different-hash-after-a-fix-attempt" },
        }),
    ]);
    assert.equal(first[0].fingerprint, second[0].fingerprint);
});

test("the report summarizes screens, verdicts and findings", () => {
    const markdown = renderReport([record()], { label: "rc-0.7.1", modelTag: "d3:2b" });
    assert.ok(markdown.includes("# Экзамен экранов: rc-0.7.1"));
    assert.ok(markdown.includes("находок: 1"));
    assert.ok(markdown.includes("Наложение текста или элементов"));
    assert.ok(markdown.includes("требует ручного триажа"));
});

test("the report lists model errors instead of hiding them", () => {
    const markdown = renderReport(
        [record({ error: "Ollama request failed", verdicts: [] })],
        { label: "rc", modelTag: "d3:2b" },
    );
    assert.ok(markdown.includes("## Ошибки экзамена"));
    assert.ok(markdown.includes("Ollama request failed"));
});

test("the report warns when the prompt nearly filled the context window", () => {
    const markdown = renderReport(
        [record({ contextOverflow: true, usage: { inputTokens: 15_500, outputTokens: 0 } })],
        { label: "rc", modelTag: "d3:2b" },
    );
    assert.ok(markdown.includes("Контекст модели близок к лимиту"));
});

test("an issue draft carries verified fields only and the fingerprint line", () => {
    const finding = collectFindings([record()])[0];
    const draft = renderIssueDraft(finding, "https://s3.origa.uwuwu.net/x.png", "rc-0.7.1");
    assert.ok(draft.title.startsWith("[manual-test] home/home (desktop): "));
    assert.ok(draft.body.includes("https://s3.origa.uwuwu.net/x.png"));
    assert.ok(draft.body.includes(`fingerprint: ${finding.fingerprint}`));
    assert.ok(draft.body.includes("подтверди дефект глазами"));
});
