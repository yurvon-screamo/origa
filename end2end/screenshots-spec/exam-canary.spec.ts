/**
 * The examiner canary: proves the d3 exam can actually catch the defect
 * classes it claims to catch.
 *
 * Each case injects a synthetic visual defect into the live home screen
 * and asserts the examiner flags it; the clean screen must stay
 * unflagged. Every case runs twice: a decision model's single lucky
 * answer is not proof, a stable one is.
 *
 * Known blind spots (measured, d3:2b AND d3:4b — see the PR report):
 * `text-clipped`, `layout-misaligned` and `wrong-typeface` injections
 * stay below the flag threshold on this light home screen (clip-path,
 * off-screen overflow, shift+rotate and Comic Sans all tried; p(true)
 * 0.27–0.66 vs the 0.75 gate). Those questions are documented in
 * `questions.ts` and stay out of the committed screen model until a
 * model or formulation clears its canary.
 *
 * Needs a live Ollama with the exam model pulled — that is why it lives
 * outside the CI suites (`npm run exam:canary`, never `npm test`).
 */
import { mkdir, readFile } from "node:fs/promises";
import { join } from "node:path";
import { test, type Browser, type Page } from "@playwright/test";

import { createExaminer, type ExamRecord, type Examiner } from "../shots/examiner/examine.ts";
import { parseExamModel } from "../shots/examiner/model.ts";
import { DEFECT_CHECKLIST } from "../shots/examiner/questions.ts";
import { readPngSize, sha256Hex, type ShotEntry } from "../shots/manifest.ts";
import { openShotSession } from "../shots/session.ts";
import type { ViewportTag } from "../shots/viewports.ts";

const ALL_QUESTIONS = DEFECT_CHECKLIST.map((question) => question.id);
const CANARY_MODEL = parseExamModel({
    version: 1,
    defaults: { threshold: 0.75 },
    screens: Object.fromEntries(
        [
            "clean",
            "overlap",
            "broken",
        ].map((screen) => [`canary:${screen}`, { questions: ALL_QUESTIONS }]),
    ),
});

const OUT_DIR = "/tmp/opencode/exam-canary";
const EXAM_MODEL_TAG = process.env.SHOT_EXAM_MODEL ?? process.env.OLLAMA_MODEL ?? "d3:2b";
const OLLAMA_URL = process.env.SHOT_EXAM_OLLAMA_URL ?? process.env.OLLAMA_URL ?? "http://localhost:11434";

/** The synthetic defect of each case: CSS injected into the live screen. */
const CASES: readonly {
    screen: string;
    css: string;
    /** The question that must flag this defect. */
    expectFlagged: string;
    viewport?: ViewportTag;
}[] = [
    {
        screen: "broken",
        css: "body { background: #fff !important; } body > * { display: none !important; }",
        expectFlagged: "broken-state",
    },
    {
        screen: "overlap",
        // DOM-agnostic: skewed semi-transparent text bands rendered
        // straight over the content — overlap that cannot read as design.
        css: `body::before, body::after { content: "НАЛОЖЕНИЕ ТЕКСТА НА КОНТЕНТ"; position: fixed !important; left: -5% !important; right: -5% !important; z-index: 99999 !important; font-size: 30px !important; color: #3d4552 !important; opacity: 0.85 !important; transform: skewY(-6deg) !important; text-align: center !important; background: transparent !important; } body::before { top: 30% !important; } body::after { bottom: 28% !important; font-size: 22px !important; transform: skewY(4deg) !important; }`,
        expectFlagged: "text-overlap",
    },
];

let examiner: Examiner;

test.beforeAll(async () => {
    await mkdir(OUT_DIR, { recursive: true });
    examiner = createExaminer({
        outDir: OUT_DIR,
        modelTag: EXAM_MODEL_TAG,
        ollamaUrl: OLLAMA_URL,
        examModel: CANARY_MODEL,
    });
});

async function openHome(
    browser: Browser,
    viewport: ViewportTag,
): Promise<{ page: Page; close: () => Promise<void> }> {
    const session = await openShotSession({
        browser,
        baseURL: "http://localhost:1420",
        locale: "ru-RU",
        viewport,
        preset: "fresh",
    });
    await session.page.goto("/home");
    await session.page.getByTestId("home-page").waitFor({ state: "visible", timeout: 120_000 });
    return { page: session.page, close: session.close };
}

async function captureEntry(page: Page, screen: string, viewport: ViewportTag): Promise<ShotEntry> {
    const file = join(OUT_DIR, `canary--${screen}--${viewport}--ru-RU.png`);
    await page.screenshot({ path: file });
    const data = await readFile(file);
    const size = readPngSize(data);
    return {
        scenario: "canary",
        title: "Канарка экзаменатора",
        screen,
        route: "/home",
        viewport,
        locale: "ru-RU",
        width: size.width,
        height: size.height,
        fullPage: false,
        bytes: data.byteLength,
        sha256: sha256Hex(data),
        file,
        capturedAt: new Date().toISOString(),
        commit: "canary",
        baseURL: "http://localhost:1420",
        cdnBaseUrl: "canary",
    };
}

function describeRecord(record: ExamRecord): string {
    if (record.error) return `error: ${record.error}`;
    return record.verdicts
        .map((verdict) => `${verdict.questionId}=${verdict.pTrue?.toFixed(3) ?? "?"}`)
        .join(", ");
}

/** One exam pass; the css (when set) is injected before the capture. */
async function examOnce(
    page: Page,
    screen: string,
    viewport: ViewportTag,
    css: string | null,
): Promise<ExamRecord> {
    await page.goto("/home");
    await page.getByTestId("home-page").waitFor({ state: "visible", timeout: 120_000 });
    if (css) await page.addStyleTag({ content: css });
    const entry = await captureEntry(page, screen, viewport);
    const record = await examiner.examineScreen(page, entry);
    if (!record) throw new Error(`canary screen "${screen}" is not in the canary model`);
    return record;
}

/** Two passes, both must agree — a single lucky answer is not proof. */
async function examTwice(
    page: Page,
    screen: string,
    viewport: ViewportTag,
    css: string | null,
): Promise<ExamRecord[]> {
    return [await examOnce(page, screen, viewport, css), await examOnce(page, screen, viewport, css)];
}

test.describe("examiner canary", () => {
    test.describe.configure({ mode: "serial" });

    test("the clean screen stays unflagged in both passes", async ({ browser }) => {
        const session = await openHome(browser, "desktop");
        try {
            const records = await examTwice(session.page, "clean", "desktop", null);
            for (const [index, record] of records.entries()) {
                const flagged = record.verdicts.filter((verdict) => verdict.flagged);
                if (record.error || flagged.length > 0) {
                    throw new Error(
                        `clean screen pass ${index + 1}: ${flagged.length} false flags — ${describeRecord(record)}`,
                    );
                }
            }
        } finally {
            await session.close();
        }
    });

    for (const defect of CASES) {
        const viewport = defect.viewport ?? "desktop";
        test(`synthetic defect "${defect.screen}" is caught in both passes (${viewport})`, async ({
            browser,
        }) => {
            test.setTimeout(600_000);
            const session = await openHome(browser, viewport);
            try {
                const records = await examTwice(session.page, defect.screen, viewport, defect.css);
                for (const [index, record] of records.entries()) {
                    if (record.error) {
                        throw new Error(`pass ${index + 1} errored: ${record.error}`);
                    }
                    const verdict = record.verdicts.find((v) => v.questionId === defect.expectFlagged);
                    if (!verdict || !verdict.flagged) {
                        throw new Error(
                            `"${defect.expectFlagged}" not flagged on pass ${index + 1} — ${describeRecord(record)}`,
                        );
                    }
                }
            } finally {
                await session.close();
            }
        });
    }
});
