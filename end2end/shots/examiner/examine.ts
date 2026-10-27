import { createHash } from "node:crypto";
import { appendFile, readdir, readFile, rm } from "node:fs/promises";
import { join } from "node:path";
import type { Page } from "@playwright/test";

import { askD3, D3Error, type AskD3Result, type D3Answer, type D3Usage } from "./d3.ts";
import { loadExamModel, questionsForScreen, type ExamModel, type ScreenExamPlan } from "./model.ts";
import { DEFECT_CHECKLIST, type ExamQuestion } from "./questions.ts";
import { formatDomSnapshot, observeVisibleNodes, type ObservedNode } from "./observe.ts";
import type { ShotEntry } from "../manifest.ts";

/**
 * The examiner: after a scenario reaches a screen, asks the local d3
 * decision model whether that screen shows a defect.
 *
 * Contract: `examineScreen` never throws — a failing model call lands
 * in `record.error` so one broken screen cannot take the capture run
 * down; the CLI surfaces the errors loudly at the end.
 */
export const DEFAULT_NUM_CTX = 16_384;

/** Stable fingerprint of the checklist — the provenance of every verdict. */
export function checklistFingerprint(): string {
    return createHash("sha256").update(JSON.stringify(DEFECT_CHECKLIST)).digest("hex").slice(0, 16);
}

export type ExamVerdict =
    | {
          readonly kind: "noul";
          readonly questionId: string;
          readonly label: string;
          readonly pTrue: number | null;
          readonly threshold: number;
          readonly flagged: boolean;
      }
    | {
          readonly kind: "choice";
          readonly questionId: string;
          readonly label: string;
          readonly expected: readonly string[];
          readonly actual: string | null;
          /** Probability the model gave to the winning option. */
          readonly pWinner: number | null;
          /** Probability the model gave to the expected option(s). */
          readonly pExpected: number | null;
          readonly flagged: boolean;
      };

export interface ExamRecord {
    readonly capturedAt: string;
    readonly screen: {
        readonly scenario: string;
        readonly screen: string;
        readonly viewport: string;
        readonly locale: string;
        readonly route: string;
        readonly file: string;
        readonly sha256: string;
        readonly width: number;
        readonly height: number;
        readonly fullPage: boolean;
    };
    readonly examiner: {
        readonly model: string;
        readonly checklist: string;
        readonly numCtx: number;
        readonly threshold: number;
    };
    readonly dom: { readonly chars: number; readonly nodes: number; readonly truncatedNodes: number };
    readonly usage: { readonly inputTokens: number; readonly outputTokens: number } | null;
    readonly latencyMs: number;
    readonly verdicts: readonly ExamVerdict[];
    readonly contextOverflow: boolean;
    readonly error: string | null;
}

export interface ExaminerOptions {
    readonly outDir: string;
    readonly modelTag: string;
    readonly ollamaUrl: string;
    readonly examModel: ExamModel;
    readonly numCtx?: number;
}

/** Environment contract between the CLI and the Playwright driver. */
export interface ExaminerEnv {
    readonly SHOT_EXAMINE?: string;
    readonly SHOT_EXAM_MODEL?: string;
    readonly SHOT_EXAM_OLLAMA_URL?: string;
    readonly SHOT_EXAM_MODEL_JSON?: string;
    readonly SHOT_EXAM_NUM_CTX?: string;
}

/**
 * Builds the examiner from the driver environment — `null` when the run
 * does not examine (the byte-for-byte default path of the tool).
 */
export async function createExaminerFromEnv(
    env: ExaminerEnv,
    outDir: string,
): Promise<Examiner | null> {
    if (env.SHOT_EXAMINE !== "1") return null;
    const modelJsonPath =
        env.SHOT_EXAM_MODEL_JSON ?? join(import.meta.dirname, "model.json");
    const numCtx = env.SHOT_EXAM_NUM_CTX ? parsePositiveInt("SHOT_EXAM_NUM_CTX", env.SHOT_EXAM_NUM_CTX) : undefined;
    return createExaminer({
        outDir,
        modelTag: env.SHOT_EXAM_MODEL ?? "d3:2b",
        ollamaUrl: env.SHOT_EXAM_OLLAMA_URL ?? "http://localhost:11434",
        examModel: await loadExamModel(modelJsonPath),
        numCtx,
    });
}

/** Fail-loud numeric env parsing: a garbage value must not become NaN. */
export function parsePositiveInt(name: string, raw: string): number {
    const value = Number(raw);
    if (!Number.isInteger(value) || value <= 0) {
        throw new Error(`${name} должно быть целым положительным числом, получено: "${raw}"`);
    }
    return value;
}

export interface Examiner {
    /** `null` when the screen is not in the test model (an explicit skip). */
    examineScreen(page: Page, entry: ShotEntry): Promise<ExamRecord | null>;
}

const STATE_PREAMBLE =
    "You are examining a screenshot of Origa, a Japanese-language learning web app.\n" +
    "The interface language is Russian; Japanese text (kanji and kana) is content, not a defect.\n" +
    "Brand fonts: Cormorant Garamond (serif) for headings and DM Mono for interface text — " +
    "plain sans-serif body text indicates a font-loading defect.\n" +
    "Visible page elements:\n";

export function createExaminer(options: ExaminerOptions): Examiner {
    const numCtx = options.numCtx ?? DEFAULT_NUM_CTX;

    return {
        async examineScreen(page, entry) {
            const plan = questionsForScreen(options.examModel, entry.scenario, entry.screen);
            if (!plan) return null;

            const record = await runExam(page, entry, plan, options, numCtx);
            await appendFile(
                join(options.outDir, `exam.${entry.scenario}.jsonl`),
                `${JSON.stringify(record)}\n`,
                "utf-8",
            );
            return record;
        },
    };
}

async function runExam(
    page: Page,
    entry: ShotEntry,
    plan: ScreenExamPlan,
    options: ExaminerOptions,
    numCtx: number,
): Promise<ExamRecord> {
    const startedAt = Date.now();
    const base = {
        capturedAt: new Date().toISOString(),
        screen: {
            scenario: entry.scenario,
            screen: entry.screen,
            viewport: entry.viewport,
            locale: entry.locale,
            route: entry.route,
            file: entry.file,
            sha256: entry.sha256,
            width: entry.width,
            height: entry.height,
            fullPage: entry.fullPage,
        },
        examiner: {
            model: options.modelTag,
            checklist: checklistFingerprint(),
            numCtx,
            threshold: plan.threshold,
        },
    };

    try {
        const nodes = (await page.evaluate(observeVisibleNodes)) as ObservedNode[];
        const dom = formatDomSnapshot({
            route: entry.route,
            locale: entry.locale,
            viewport: entry.viewport,
            nodes,
        });
        const imageBase64 = (await readFile(entry.file)).toString("base64");

        const questions = new Map(
            plan.questions.map((question) => [
                question.id,
                question.kind === "noul"
                    ? { kind: "noul" as const, instructions: question.instructions, criteria: question.criteria }
                    : { kind: "choice" as const, instructions: question.instructions, criteria: question.options },
            ]),
        );
        const result = await withRetry(() =>
            askD3({
                ollamaUrl: options.ollamaUrl,
                model: options.modelTag,
                state: STATE_PREAMBLE + dom.text,
                images: [imageBase64],
                questions,
                numCtx,
            }),
        );

        const verdicts = mapVerdicts(plan.questions, result.answers, plan.expect);

        return {
            ...base,
            dom: { chars: dom.text.length, nodes: dom.nodeCount, truncatedNodes: dom.truncatedNodes },
            usage: result.usage,
            latencyMs: Date.now() - startedAt,
            verdicts,
            contextOverflow: isContextOverflow(result.usage, numCtx),
            error: null,
        };
    } catch (error) {
        return {
            ...base,
            dom: { chars: 0, nodes: 0, truncatedNodes: 0 },
            usage: null,
            latencyMs: Date.now() - startedAt,
            verdicts: [],
            contextOverflow: false,
            error: error instanceof Error ? error.message : String(error),
        };
    }
}

/** One retry absorbs a transient local-model hiccup; the second failure is recorded. */
export async function withRetry(request: () => Promise<AskD3Result>): Promise<AskD3Result> {
    try {
        return await request();
    } catch (error) {
        if (!(error instanceof D3Error)) throw error;
        return await request();
    }
}

/**
 * Turns the model answers into verdicts.
 *
 * A noul question the model left unanswered keeps `pTrue: null` and is
 * never flagged — the report and the CLI surface such verdicts
 * separately, because a silently skipped question would read as «no
 * defect». A choice question is flagged when the winning option is not
 * one of the expected ones.
 */
export function mapVerdicts(
    questions: readonly ExamQuestion[],
    answers: ReadonlyMap<string, D3Answer>,
    expect: Readonly<Record<string, readonly string[]>> = {},
): ExamVerdict[] {
    return questions.map((question) => {
        const answer = answers.get(question.id);
        if (question.kind === "noul") {
            const flagged = answer?.kind === "noul" && answer.pTrue !== undefined && answer.pTrue >= question.threshold;
            return {
                kind: "noul" as const,
                questionId: question.id,
                label: question.label,
                pTrue: answer?.kind === "noul" ? (answer.pTrue ?? null) : null,
                threshold: question.threshold,
                flagged,
            };
        }
        const expected = expect[question.id] ?? [];
        const actual = answer?.kind === "choice" ? (answer.choiceId ?? null) : null;
        const pWinner =
            answer?.kind === "choice" && answer.choiceId !== undefined
                ? (answer.probabilities?.[answer.choiceId] ?? null)
                : null;
        const pExpected =
            answer?.kind === "choice" && answer.probabilities
                ? Math.max(0, ...expected.map((option) => answer.probabilities?.[option] ?? 0))
                : null;
        const flagged = expected.length > 0 && actual !== null && !expected.includes(actual);
        return {
            kind: "choice" as const,
            questionId: question.id,
            label: question.label,
            expected,
            actual,
            pWinner,
            pExpected,
            flagged,
        };
    });
}

/** A prompt near the window means answers may be silently truncated. */
export function isContextOverflow(usage: D3Usage | null, numCtx: number): boolean {
    return usage !== null && usage.inputTokens > numCtx * 0.9;
}

/**
 * Removes the exam traces of the scenarios this run is about to examine.
 *
 * The trace appends, the manifest overwrites — without this reset a
 * re-run with the same label would read the previous run's findings
 * back into the report (and into `--file-issues`).
 */
export async function resetExamTraces(outDir: string, scenarios: readonly string[]): Promise<void> {
    for (const scenario of scenarios) {
        await rm(join(outDir, `exam.${scenario}.jsonl`), { force: true });
    }
}

/** Reads every exam trace of a run (one JSONL file per scenario). */
export async function readExamRecords(outDir: string): Promise<ExamRecord[]> {
    const files = (await readdir(outDir)).filter(
        (name) => name.startsWith("exam.") && name.endsWith(".jsonl"),
    );
    const records: ExamRecord[] = [];
    for (const file of files) {
        const content = await readFile(join(outDir, file), "utf-8");
        for (const line of content.split("\n")) {
            if (line.trim().length === 0) continue;
            try {
                records.push(JSON.parse(line) as ExamRecord);
            } catch (error) {
                // A torn line (a crash mid-append) must not kill the
                // whole report — but it must not pass silently either.
                console.error(`битая строка трейса в ${file}: ${error instanceof Error ? error.message : String(error)}`);
            }
        }
    }
    return records.sort(
        (left, right) =>
            left.screen.scenario.localeCompare(right.screen.scenario) ||
            left.capturedAt.localeCompare(right.capturedAt),
    );
}
