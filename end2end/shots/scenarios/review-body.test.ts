import assert from "node:assert/strict";
import { test } from "node:test";

import type { BrowserContext, Locator, Page } from "@playwright/test";

import { captureReviewBody } from "./review-body.ts";
import { lessonReviewBodyScenario } from "./lesson-review-body.ts";
import type { ShotContext, ShotSink } from "../types.ts";
import type { ShotEntry } from "../manifest.ts";

// The fake page models a lesson as a linear list of screens: every click
// on a visible element advances to the next screen. That is enough for
// the contracts under test here — stop conditions, captured counts,
// frame names and the reroll budget. What it deliberately cannot see is
// the ORDER of a reveal click vs the sink.shot that follows it (the fake
// advances on click, so both orders look identical): the reveal-before-
// capture ordering is pinned by the live capture run, not by these units.
// Unsupported selectors fail loudly so a new selector shape in the walk
// cannot silently pass against a broken model.

interface FakeElement {
    tag: string;
    testid?: string;
    classes?: string[];
}

type FakeScreen = FakeElement[];

/** Resolves the CSS shapes QUESTION_MARKERS and REVEALS/ADVANCES use. */
function matchesSelector(element: FakeElement, selector: string): boolean {
    const attr = selector.match(/^(?:([a-z]+))?\[data-testid(\^?)="([^"]+)"\]$/);
    if (attr) {
        const [, tag, caret, value] = attr;
        if (tag && element.tag !== tag) return false;
        const testid = element.testid ?? "";
        return caret === "^" ? testid.startsWith(value) : testid === value;
    }
    const classSel = selector.match(/^([a-z]+)\.([a-z-]+)$/);
    if (classSel) {
        const [, tag, cls] = classSel;
        return element.tag === tag && (element.classes ?? []).includes(cls);
    }
    throw new Error(`fake page: unsupported selector "${selector}"`);
}

class FakeLocator {
    private page: FakePage;
    private matches: (element: FakeElement) => boolean;

    constructor(page: FakePage, matches: (element: FakeElement) => boolean) {
        this.page = page;
        this.matches = matches;
    }

    first(): FakeLocator {
        return this;
    }

    async waitFor(_options: { state: string; timeout?: number }): Promise<Locator> {
        if (this.visible()) return this as unknown as Locator;
        throw new Error("fake page: element not visible");
    }

    async click(_options?: { timeout?: number }): Promise<void> {
        if (!this.visible()) throw new Error("fake page: element not clickable");
        this.page.advance();
    }

    private visible(): boolean {
        return this.page.currentScreen().some(this.matches);
    }
}

class FakePage {
    /** Elements visible on every screen regardless of the lesson state. */
    chrome: FakeElement[] = [];

    private lessons: FakeScreen[][];
    private lessonIndex = 0;
    private screenIndex = 0;
    private loaded = false;

    constructor(lessons: FakeScreen[][]) {
        this.lessons = lessons;
    }

    /**
     * The first goto loads the initial lesson script; every next goto is
     * a reload — the views regenerate, the next script takes over.
     */
    goto(_url: string): Promise<unknown> {
        if (this.loaded && this.lessonIndex < this.lessons.length - 1) this.lessonIndex += 1;
        this.loaded = true;
        this.screenIndex = 0;
        return Promise.resolve();
    }

    currentScreen(): FakeElement[] {
        const lesson = this.lessons[Math.min(this.lessonIndex, this.lessons.length - 1)];
        return [...this.chrome, ...(lesson[Math.min(this.screenIndex, lesson.length - 1)] ?? [])];
    }

    advance(): void {
        this.screenIndex += 1;
    }

    getByTestId(testid: string): FakeLocator {
        return new FakeLocator(this, (element) => element.testid === testid);
    }

    locator(selector: string): FakeLocator {
        return new FakeLocator(this, (element) => matchesSelector(element, selector));
    }

    waitForTimeout(_ms: number): Promise<void> {
        return Promise.resolve();
    }
}

/** The `ShotSink` a scenario writes its screens into. */
function fakeSink(): ShotSink & { names: string[] } {
    const names: string[] = [];
    return {
        names,
        entries: [],
        shot: async (name: string) => {
            names.push(name);
            return { screen: name } as unknown as ShotEntry;
        },
    };
}

const el = (testid: string): FakeElement => ({ tag: "div", testid });
const revealScreen: FakeScreen = [el("lesson-show-answer-btn")];
const ratingScreen: FakeScreen = [el("lesson-rating-btn-again"), el("lesson-card-next-btn")];
const completeScreen: FakeScreen = [el("lesson-complete-screen")];
const writingScreen: FakeScreen = [{ tag: "canvas", classes: ["kanji-drawing-canvas"] }];

function normalLesson(cards = 2): FakeScreen[] {
    const screens: FakeScreen[] = [];
    for (let i = 0; i < cards; i++) screens.push(revealScreen, ratingScreen);
    return [...screens, completeScreen];
}

test("a writing card holding the first screen stops the walk honestly — no stale-selector throw", async () => {
    // One lesson: the writing card opens the review body; whatever could
    // follow never gets reached — the walk must stop before it.
    const page = new FakePage([[writingScreen, revealScreen]]);
    const sink = fakeSink();

    const captured = await captureReviewBody(page as unknown as Page, sink, 6);

    assert.equal(captured, 0);
    assert.deepEqual(sink.names, []);
});

test("a run of normal cards is captured and counted", async () => {
    const page = new FakePage([normalLesson(2)]);
    const sink = fakeSink();

    const captured = await captureReviewBody(page as unknown as Page, sink, 6);

    assert.equal(captured, 2);
    assert.deepEqual(sink.names, ["review-body-normal-1", "review-body-normal-2"]);
});

test("a writing card deeper in the lesson stops with the frames already collected", async () => {
    const page = new FakePage([[...normalLesson(2).slice(0, 4), writingScreen]]);
    const sink = fakeSink();

    const captured = await captureReviewBody(page as unknown as Page, sink, 6);

    assert.equal(captured, 2);
    assert.deepEqual(sink.names, ["review-body-normal-1", "review-body-normal-2"]);
});

test("a genuinely unknown first card must still fail loudly", async () => {
    const page = new FakePage([[[el("some-future-widget")]]]);
    const sink = fakeSink();

    await assert.rejects(
        captureReviewBody(page as unknown as Page, sink, 6),
        /did not reveal its first card/,
    );
});

test("an unknown kind with a reveal control is still captured", async () => {
    const page = new FakePage([
        [[el("lesson-example-reveal"), el("lesson-example-next")], completeScreen],
    ]);
    const sink = fakeSink();

    const captured = await captureReviewBody(page as unknown as Page, sink, 6);

    assert.equal(captured, 1);
    assert.deepEqual(sink.names, ["review-body-unknown-1"]);
});

test("no advance control after a frame ends the walk quietly, keeping the frame", async () => {
    const page = new FakePage([
        [[revealScreen[0], el("lesson-unknown-aftermath")], completeScreen],
    ]);
    const sink = fakeSink();

    const captured = await captureReviewBody(page as unknown as Page, sink, 6);

    assert.equal(captured, 1);
    assert.deepEqual(sink.names, ["review-body-normal-1"]);
});

test("the scenario rerolls a writing-first lesson and captures the next one", async () => {
    // Attempt 0 opens on writing; attempt 1 is a fresh normal lesson.
    const page = new FakePage([[writingScreen], normalLesson(2)]);
    const sink = fakeSink();

    await lessonReviewBodyScenario.run(scenarioContext(page), sink);

    assert.deepEqual(sink.names, ["review-body-normal-1", "review-body-normal-2"]);
});

test("the scenario gives up with an honest error when every attempt opens unwalkably", async () => {
    const allWriting: FakeScreen[][] = Array.from({ length: 5 }, () => [writingScreen]);
    const page = new FakePage(allWriting);
    const sink = fakeSink();

    await assert.rejects(
        lessonReviewBodyScenario.run(scenarioContext(page), sink),
        /no review-body frames/,
    );
    assert.deepEqual(sink.names, []);
});

const scenarioContext = (page: FakePage) => {
    page.chrome = [el("lesson-page")];
    return {
        page: page as unknown as Page,
        context: {} as unknown as BrowserContext,
        baseURL: "http://localhost:1420",
        locale: "ru-RU",
        viewport: "desktop",
    } as unknown as ShotContext;
};
