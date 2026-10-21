import type { PresetId } from "./presets/registry.ts";
import { DEFAULT_VIEWPORTS, type ViewportTag } from "./viewports.ts";

/**
 * Everything about a screen except HOW to reach it.
 *
 * The catalog holds the metadata only, so the CLI can list the screens
 * without loading the browser-side modules. The implementations in
 * `shots/scenarios/` spread these descriptors and add the `run` step —
 * one source of truth for the metadata.
 */
export interface ScenarioDescriptor {
    /** Stable identifier — the registry key and the CLI argument. */
    id: string;
    /** Human title (Russian) for the registry and the reports. */
    title: string;
    /** Route the screen belongs to. */
    route: string;
    locale: "ru-RU" | "en-US";
    /** Viewports to capture; the desktop/phone pair by default. */
    viewports?: ViewportTag[];
    /** Alternative build to capture (a before/after pair on another port). */
    baseURL?: string;
    /** Data state the screen is shown in. */
    preset: PresetId;
}

export const descriptors: Record<string, ScenarioDescriptor> = {
    home: {
        id: "home",
        title: "Главный экран",
        route: "/home",
        locale: "ru-RU",
        viewports: DEFAULT_VIEWPORTS,
        preset: "fresh",
    },
    "word-examples": {
        id: "word-examples",
        title: "Слово с примерами",
        route: "/words/:word",
        locale: "ru-RU",
        viewports: DEFAULT_VIEWPORTS,
        preset: "onboarded",
    },
    "lesson-example": {
        id: "lesson-example",
        title: "Урок: знакомство и ответ",
        route: "/lesson",
        locale: "ru-RU",
        viewports: DEFAULT_VIEWPORTS,
        preset: "with-word",
    },
    "counter-scoring": {
        id: "counter-scoring",
        title: "Карточка счётчика в разметке",
        route: "/onboarding",
        locale: "ru-RU",
        viewports: DEFAULT_VIEWPORTS,
        preset: "scoring-n5",
    },
    "home-due-counter": {
        id: "home-due-counter",
        title: "Счётчик просроченных карточек",
        route: "/home",
        locale: "ru-RU",
        viewports: DEFAULT_VIEWPORTS,
        preset: "onboarded",
    },
    "home-insights": {
        id: "home-insights",
        title: "Панели наблюдений",
        route: "/home",
        locale: "ru-RU",
        viewports: DEFAULT_VIEWPORTS,
        preset: "onboarded-insights",
    },
    "stitch-session": {
        id: "stitch-session",
        title: "Леджер наблюдений целиком",
        route: "/home",
        locale: "ru-RU",
        viewports: DEFAULT_VIEWPORTS,
        preset: "ledger",
    },
    "feedback-form": {
        id: "feedback-form",
        title: "Форма обратной связи в уроке",
        route: "/lesson",
        locale: "ru-RU",
        viewports: DEFAULT_VIEWPORTS,
        preset: "with-word",
    },
    "token-popup": {
        id: "token-popup",
        title: "Поповер токена с кнопкой жалобы",
        route: "/phrases",
        locale: "ru-RU",
        viewports: DEFAULT_VIEWPORTS,
        preset: "onboarded",
    },
    "jft-card": {
        id: "jft-card",
        title: "Карточка готовности JFT-Basic",
        route: "/home",
        locale: "ru-RU",
        viewports: DEFAULT_VIEWPORTS,
        preset: "irodori-nyuumon",
    },
    "lesson-unification": {
        id: "lesson-unification",
        title: "Единый урок: знакомство, тренировка, тело",
        route: "/lesson",
        locale: "ru-RU",
        viewports: DEFAULT_VIEWPORTS,
        preset: "lesson-ready",
    },
};

export function listDescriptors(): ScenarioDescriptor[] {
    return Object.values(descriptors).sort((a, b) => a.id.localeCompare(b.id));
}
