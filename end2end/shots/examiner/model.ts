import { readFile } from "node:fs/promises";

import { findQuestion, type NoulQuestion } from "./questions.ts";

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

/** The trace key of a screen: the scenario id and the `sink.shot()` name. */
export function screenKey(scenario: string, screen: string): string {
    return `${scenario}:${screen}`;
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
        if (!/^[A-Za-z0-9._-]+:[A-Za-z0-9._-]+$/.test(key)) {
            throw new ExamModelError(`screen key "${key}" must look like scenario:screen`);
        }
        if (typeof screen !== "object" || screen === null) {
            throw new ExamModelError(`screen "${key}" must be an object`);
        }
        const screenValue = screen as { questions?: unknown; threshold?: unknown };
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
        screens[key] = { questions: [...screenValue.questions] as string[] };
    }

    return { version: 1, defaults: { threshold: defaults.threshold }, screens };
}

export async function loadExamModel(path: string): Promise<ExamModel> {
    // readFile + JSON.parse instead of a JSON import: the tool runs under
    // Node type stripping, where JSON imports need import attributes.
    return parseExamModel(JSON.parse(await readFile(path, "utf-8")));
}

export interface ScreenExamPlan {
    readonly questions: readonly NoulQuestion[];
    readonly threshold: number;
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
    const exam = model.screens[screenKey(scenario, screen)];
    if (!exam) return null;
    // The parser guarantees every id is known; the filter only keeps the
    // compiler honest about hand-built models in tests.
    const questions = exam.questions
        .map((id) => findQuestion(id))
        .filter((question): question is NoulQuestion => question !== undefined);
    return { questions, threshold: model.defaults.threshold };
}
