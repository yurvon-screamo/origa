/**
 * The defect checklist the examiner asks d3 about every screen.
 *
 * A question is a typed `noul` (yes/no with a probability) request: the
 * instructions state what to look for, the criteria spell out when the
 * answer is true. d3 scores clear, distinct criteria best — a vague
 * question produces noise, and a question that is never asked finds
 * nothing (the model only answers what it is asked).
 */
export interface NoulQuestion {
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
 * The default defect classes of a pre-release manual pass: what a human
 * tester would scan for but the scripted e2e suite cannot see.
 */
export const DEFECT_CHECKLIST: readonly NoulQuestion[] = [
    {
        id: "text-overlap",
        label: "Наложение текста или элементов",
        instructions:
            "Look at the screenshot. Do any texts or UI elements overlap each other so that one visibly hides part of another?",
        criteria: {
            true: "At least one text or element is rendered on top of another text or element, hiding part of it.",
            false: "No texts or elements overlap; everything has its own clear space.",
        },
        threshold: 0.75,
    },
    {
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
export function findQuestion(id: string): NoulQuestion | undefined {
    return DEFECT_CHECKLIST.find((question) => question.id === id);
}
