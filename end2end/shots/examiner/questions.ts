/**
 * The defect checklist the examiner asks d3 about every screen.
 *
 * A question is a typed `noul` (yes/no with a probability) request: the
 * instructions state what to look for, the criteria spell out when the
 * answer is true. d3 scores clear, distinct criteria best — a vague
 * question produces noise, and a question that is never asked finds
 * nothing (the model only answers what it is asked).
 */
/**
 * A yes/no checklist question with a calibrated flag threshold.
 */
export interface NoulQuestion {
    readonly kind: "noul";
    /** Stable identifier — the key in the test model and the trace. */
    readonly id: string;
    /** Human label (Russian) for the reports and the filed issues. */
    readonly label: string;
    readonly instructions: string;
    readonly criteria: { readonly true: string; readonly false: string };
    /** `pTrue >= threshold` means the defect is present (a finding). */
    readonly threshold: number;
}

/**
 * A choice question: the model picks one of the named options.
 *
 * Used as a semantic oracle over a flow: the test model states which
 * option a healthy screen must win with (see `expect` in model.json).
 */
export interface ChoiceQuestion {
    readonly kind: "choice";
    readonly id: string;
    readonly label: string;
    readonly instructions: string;
    /** Option id → when this option is the right pick. */
    readonly options: Readonly<Record<string, string>>;
}

export type ExamQuestion = NoulQuestion | ChoiceQuestion;

/**
 * The default defect classes of a pre-release manual pass: what a human
 * tester would scan for but the scripted e2e suite cannot see.
 */
export const DEFECT_CHECKLIST: readonly NoulQuestion[] = [
    {
        kind: "noul",
        id: "text-overlap",
        label: "Наложение текста или элементов",
        instructions:
            "Look at the screenshot. Do any texts or UI elements overlap each other so that one visibly hides part of another?",
        criteria: {
            true: "At least one text or element is rendered on top of another text or element, hiding part of it.",
            false: "No texts or elements overlap; everything has its own clear space.",
        },
        // Calibrated between the clean-screen noise (p up to 0.44) and
        // the canary injection (p 0.70–0.82 across runs) — the gate must
        // sit clear of both observed edges, not on one of them.
        threshold: 0.65,
    },
    {
        kind: "noul",
        id: "text-clipped",
        label: "Обрезка текста или элементов",
        instructions:
            "Look at the screenshot. Is any text or UI element cut off at an edge of the screen or its container (a word, button or card missing a visible part)?",
        criteria: {
            true: "At least one text or element is visibly cut off by the viewport or its container edge.",
            false: "All texts and elements fit fully inside their containers and the screen.",
        },
        // Measured blind spot of d3 (2b and 4b): clip-path, off-screen
        // overflow and hard viewport cuts all stay under the flag
        // threshold on light screens. Needs a canary pass before any
        // screen enables it.
        threshold: 0.75,
    },
    {
        kind: "noul",
        id: "tofu-glyphs",
        label: "Битые глифы (тофу)",
        instructions:
            "Look at the text in the screenshot. Does any of it render as tofu (empty rectangles □□), question marks or other broken placeholder glyphs instead of real characters?",
        criteria: {
            true: "Some text renders as empty boxes, placeholder boxes or visibly broken glyphs.",
            false: "All text renders as real readable characters, including the Japanese text.",
        },
        threshold: 0.75,
    },
    {
        kind: "noul",
        id: "layout-misaligned",
        label: "Перекос или смещение вёрстки",
        instructions:
            "Look at the overall layout of the screenshot. Does it look visibly misaligned, skewed or broken — elements rotated, shifted out of place or their alignment clearly wrong?",
        criteria: {
            true: "The layout is visibly skewed, rotated or misaligned.",
            false: "The layout is straight and aligned; elements sit where a designed screen would place them.",
        },
        // Measured blind spot of d3 (2b and 4b): rotation, skew and large
        // shifts of the content block stay under the flag threshold.
        // Needs a canary pass before any screen enables it.
        threshold: 0.75,
    },
    {
        kind: "noul",
        id: "broken-state",
        label: "Пустой или сломанный экран",
        instructions:
            "Look at the screenshot. Does the screen look empty, broken or stuck in an unfinished state — no meaningful content, a blank area where content belongs, or a stuck loader?",
        criteria: {
            true: "The screen is empty, blank or clearly stuck without meaningful content.",
            false: "The screen shows meaningful, fully rendered content.",
        },
        threshold: 0.75,
    },
    {
        kind: "noul",
        id: "wrong-typeface",
        label: "Чужой шрифт вместо фирменного",
        instructions:
            "This app brands itself with a serif font (Cormorant Garamond) for headings and monospace fonts (DM Mono or IBM Plex Mono) for interface text; a monospaced look is NORMAL here. Look at the typefaces in the screenshot. Did the text fall back to a plain sans-serif system font (Arial/Helvetica-like) — the branded serif and monospace fonts missing everywhere?",
        criteria: {
            true: "The text is set in a plain sans-serif system font; no serif or monospace branding is visible anywhere on the screen.",
            false: "A monospaced typeface and/or the serif brand font is visibly in use (monospace interface text is the expected look).",
        },
        // d3:2b cannot clear this question: a Comic Sans injection scored
        // BELOW the clean screen's noise floor. Enable on screens only
        // after the model can tell the faces apart (d3:4b target).
        threshold: 0.9,
    },
];

/** Finds a checklist question by id. */
export function findQuestion(id: string): ExamQuestion | undefined {
    return DEFECT_CHECKLIST.find((question) => question.id === id) ?? LESSON_QUESTIONS.find((question) => question.id === id);
}

/**
 * The lesson flow questions: a semantic oracle over the lesson walk.
 *
 * The stage question is the workhorse: the test model states which stage
 * a healthy screen must show, and a mix-up (the wrong stage after a
 * transition) is a flow bug the scripted asserts cannot see through the
 * randomization. The invariant questions check the card anatomy.
 */
export const LESSON_QUESTIONS: readonly ExamQuestion[] = [
    {
        kind: "choice",
        id: "lesson-stage",
        label: "Стадия урока",
        instructions:
            "Look at the screenshot of this Japanese-learning app. Which stage of a lesson flow does the screen show? Tell the stages apart by their CONTROLS, not by the content: what the screen asks the user to DO is the stage.",
        options: {
            presentation:
                "Presentation: an introductory slide showing new material (word, kanji, grammar, counter) with a plain NEXT control — no reveal, no rating, no options to pick.",
            "training-question":
                "Training card before the answer: a recall prompt with a REVEAL / show-answer control (and no answer visible yet).",
            "training-revealed":
                "Training card after the answer: the prompt and its revealed answer together, with a SELF-RATING control (again/remember/difficulty) or an example-understood control.",
            "quiz-question":
                "Multiple-choice quiz before the answer: a question with several OPTIONS to pick from (or to assemble), nothing marked as answered yet, no reveal control.",
            "quiz-answer":
                "Multiple-choice quiz after the answer: the picked option is marked, with feedback (correct/wrong, missing/extra) and a submit or next control.",
            complete:
                "Lesson completion: a summary or congratulations screen, the lesson is over, no card prompt is on screen.",
            "not-lesson":
                "Not a lesson screen at all: a home screen, a list, settings, or a dialog over something else.",
        },
    },
    {
        kind: "noul",
        id: "quiz-single-highlight",
        label: "Ровно одна опция помечена после ответа",
        instructions:
            "This is a quiz screen after an answer was given. Is exactly one option visually marked as the picked or correct one (highlight, border, check), while the others stay plain?",
        criteria: {
            true: "Exactly one option is visually marked; all other options are plain.",
            false: "No option is marked, or several options are marked at once.",
        },
        threshold: 0.75,
    },
    {
        kind: "noul",
        id: "card-has-japanese",
        label: "Карточка без японского контента",
        instructions:
            "Every card of this app's lesson must have Japanese text (kanji, kana, or a Japanese word) as its subject — the thing the card teaches or asks about. Does this card FAIL that: no Japanese content serves as the card's subject?",
        criteria: {
            true: "The card's subject is not Japanese text — no kanji, kana or Japanese word is what the card teaches or asks about.",
            false: "The card's subject is Japanese text (a word, kanji or phrase).",
        },
        threshold: 0.75,
    },
];
