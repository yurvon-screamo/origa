import { readFile } from "node:fs/promises";

import { findQuestion, type ExamQuestion } from "./questions.ts";

/**
 * The test model: which screens get examined and with which questions.
 *
 * The model is a committed artefact of the repository — the examiner's
 * knowledge about the app is versioned next to the code it checks (the
 * deterministic replacement for a planner in this slice). A screen is
 * addressed as `scenario:screen` — the same names `sink.shot()` uses.
 */
export interface ScreenExam {
    /** Checklist question ids asked about this screen (a full replacement set). */
    readonly questions: readonly string[];
    /**
     * Choice question id → the option(s) a healthy screen must win with
     * (normalized to arrays by the parser). A mismatch is a finding.
     */
    readonly expect?: Readonly<Record<string, readonly string[]>>;
}

export interface ExamModel {
    readonly version: 1;
    readonly defaults: { readonly threshold: number };
    readonly screens: Readonly<Record<string, ScreenExam>>;
}

export class ExamModelError extends Error {
    constructor(message: string) {
        super(message);
        this.name = "ExamModelError";
    }
}

/** Normalizes the expect value to arrays of allowed options. */
function normalizeExpect(expect: Record<string, unknown>): Record<string, readonly string[]> {
    const result: Record<string, readonly string[]> = {};
    for (const [questionId, expected] of Object.entries(expect)) {
        result[questionId] = Array.isArray(expected) ? [...expected] : [expected];
    }
    return result;
}

/** The trace key of a screen: the scenario id and the `sink.shot()` name. */
export function screenKey(scenario: string, screen: string): string {
    return `${scenario}:${screen}`;
}

/**
 * Resolves the model entry for a screen.
 *
 * A key may end with `*`: it then matches every screen of the scenario
 * whose name starts with the prefix — the lesson review body names its
 * captures dynamically (`review-body-<kind>-<card>`). Exact keys win
 * over wildcard keys.
 */
export function findScreenExam(
    model: ExamModel,
    scenario: string,
    screen: string,
): ScreenExam | null {
    const exact = model.screens[screenKey(scenario, screen)];
    if (exact) return exact;
    for (const [key, exam] of Object.entries(model.screens)) {
        const separator = key.indexOf(":");
        if (separator === -1) continue;
        const keyScenario = key.slice(0, separator);
        const pattern = key.slice(separator + 1);
        if (keyScenario !== scenario || !pattern.endsWith("*")) continue;
        const prefix = pattern.slice(0, -1);
        if (prefix.length > 0 && screen.startsWith(prefix)) return exam;
    }
    return null;
}

/** Parses and validates a raw model (a JSON value or an inline object in tests). */
export function parseExamModel(raw: unknown): ExamModel {
    if (typeof raw !== "object" || raw === null) {
        throw new ExamModelError("the test model must be a JSON object");
    }
    const value = raw as Record<string, unknown>;

    if (value.version !== 1) {
        throw new ExamModelError(`unsupported test model version: ${String(value.version)}`);
    }

    const defaults = value.defaults as { threshold?: unknown } | undefined;
    if (typeof defaults?.threshold !== "number" || defaults.threshold <= 0 || defaults.threshold >= 1) {
        throw new ExamModelError("defaults.threshold must be a number between 0 and 1 (exclusive)");
    }

    if (typeof value.screens !== "object" || value.screens === null) {
        throw new ExamModelError("the test model must have a screens object");
    }

    const screens: Record<string, ScreenExam> = {};
    for (const [key, screen] of Object.entries(value.screens as Record<string, unknown>)) {
        if (!/^[A-Za-z0-9._-]+:[A-Za-z0-9._-]+\*?$/.test(key)) {
            throw new ExamModelError(`screen key "${key}" must look like scenario:screen (a trailing * marks a prefix match)`);
        }
        if (typeof screen !== "object" || screen === null) {
            throw new ExamModelError(`screen "${key}" must be an object`);
        }
        const screenValue = screen as { questions?: unknown; threshold?: unknown; expect?: unknown };
        if (!Array.isArray(screenValue.questions) || screenValue.questions.length === 0) {
            throw new ExamModelError(`screen "${key}" must list at least one question id`);
        }
        for (const id of screenValue.questions) {
            if (typeof id !== "string" || !findQuestion(id)) {
                throw new ExamModelError(`screen "${key}" names an unknown question "${String(id)}"`);
            }
        }
        // Thresholds live per-question in the checklist (calibrated by the
        // canary); a screen-level override would silently discard that
        // calibration, so the parser rejects it outright.
        if (screenValue.threshold !== undefined) {
            throw new ExamModelError(
                `screen "${key}" sets a threshold — thresholds belong to the checklist questions (shots/examiner/questions.ts)`,
            );
        }
        const expectValue = screenValue.expect as Record<string, string | string[]> | undefined;
        if (expectValue !== undefined) {
            if (typeof expectValue !== "object" || expectValue === null) {
                throw new ExamModelError(`screen "${key}" has a malformed expect object`);
            }
            for (const [questionId, expected] of Object.entries(expectValue)) {
                const question = findQuestion(questionId);
                if (!question || question.kind !== "choice") {
                    throw new ExamModelError(`screen "${key}" expects "${questionId}" which is not a choice question`);
                }
                const allowed = Array.isArray(expected) ? expected : [expected];
                if (allowed.length === 0) {
                    throw new ExamModelError(`screen "${key}" expects an empty option set for "${questionId}"`);
                }
                for (const option of allowed) {
                    if (typeof option !== "string" || !(option in question.options)) {
                        throw new ExamModelError(
                            `screen "${key}" expects unknown option "${String(option)}" of "${questionId}"`,
                        );
                    }
                }
            }
        }
        // A choice question without an expectation is an invisible verdict:
        // asked, paid for in tokens, but neither a finding nor unanswered.
        for (const id of screenValue.questions) {
            const question = findQuestion(String(id));
            if (question?.kind === "choice" && expectValue?.[question.id] === undefined) {
                throw new ExamModelError(
                    `screen "${key}" asks "${question.id}" without an expect — a choice verdict needs the healthy option set`,
                );
            }
        }
        screens[key] = {
            questions: [...screenValue.questions] as string[],
            ...(expectValue !== undefined
                ? { expect: normalizeExpect(expectValue as Record<string, string | string[]>) }
                : {}),
        };
    }

    // Two wildcard keys of one scenario whose prefixes overlap would let
    // the JSON key order pick the winner — refuse the ambiguity loudly.
    const wildcards = new Map<string, string[]>();
    for (const key of Object.keys(screens)) {
        const separator = key.indexOf(":");
        const pattern = key.slice(separator + 1);
        if (!pattern.endsWith("*")) continue;
        const scenario = key.slice(0, separator);
        const prefix = pattern.slice(0, -1);
        const prefixes = wildcards.get(scenario) ?? [];
        for (const other of prefixes) {
            if (other.startsWith(prefix) || prefix.startsWith(other)) {
                throw new ExamModelError(
                    `scenario "${scenario}" has overlapping wildcard keys ("${other}*" and "${prefix}*") — the match would be order-dependent`,
                );
            }
        }
        prefixes.push(prefix);
        wildcards.set(scenario, prefixes);
    }

    return { version: 1, defaults: { threshold: defaults.threshold }, screens };
}

export async function loadExamModel(path: string): Promise<ExamModel> {
    // readFile + JSON.parse instead of a JSON import: the tool runs under
    // Node type stripping, where JSON imports need import attributes.
    return parseExamModel(JSON.parse(await readFile(path, "utf-8")));
}

export interface ScreenExamPlan {
    readonly questions: readonly ExamQuestion[];
    readonly threshold: number;
    /** Choice question id → the option(s) a healthy screen must win with. */
    readonly expect: Readonly<Record<string, readonly string[]>>;
}

/**
 * Resolves the exam plan for a screen.
 *
 * `null` means the screen is not examined — the model opts screens in
 * explicitly, so an unlisted capture is a skip, not an error.
 */
export function questionsForScreen(
    model: ExamModel,
    scenario: string,
    screen: string,
): ScreenExamPlan | null {
    const exam = findScreenExam(model, scenario, screen);
    if (!exam) return null;
    // The parser guarantees every id is known; the filter only keeps the
    // compiler honest about hand-built models in tests.
    const questions = exam.questions
        .map((id) => findQuestion(id))
        .filter((question): question is ExamQuestion => question !== undefined);
    return { questions, threshold: model.defaults.threshold, expect: exam.expect ?? {} };
}
