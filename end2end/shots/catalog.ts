import type { PresetId } from "./presets/registry.ts";

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
    preset: "fresh",
  },
  "word-examples": {
    id: "word-examples",
    title: "Слово с примерами",
    route: "/words/:word",
    locale: "ru-RU",
    preset: "onboarded",
  },
  "words-grid": {
    id: "words-grid",
    title: "Экран «Слова»: сетка карточек",
    route: "/words",
    locale: "ru-RU",
    preset: "with-word",
  },
  "lesson-acquaintance-grammar": {
    id: "lesson-acquaintance-grammar",
    title: "Урок: ответ знакомства (грамматика)",
    route: "/lesson",
    locale: "ru-RU",
    preset: "acquaintance-hand",
  },
  "lesson-acquaintance-kanji": {
    id: "lesson-acquaintance-kanji",
    title: "Урок: слайд знакомства (кандзи)",
    route: "/lesson",
    locale: "ru-RU",
    preset: "acquaintance-hand",
  },
  "lesson-example": {
    id: "lesson-example",
    title: "Урок: знакомство и ответ",
    route: "/lesson",
    locale: "ru-RU",
    preset: "with-word",
  },
  "lesson-example-recall": {
    id: "lesson-example-recall",
    title: "Урок: карта-пример",
    route: "/lesson",
    locale: "ru-RU",
    preset: "due-examples",
  },
  "lesson-reading-quiz": {
    id: "lesson-reading-quiz",
    title: "Урок: квиз чтений кандзи",
    route: "/lesson",
    locale: "ru-RU",
    preset: "due-corpus",
  },
  "lesson-yesno-result": {
    id: "lesson-yesno-result",
    title: "Урок: результат да/нет-квиза и разворот карточки",
    route: "/lesson",
    locale: "ru-RU",
    preset: "due-corpus",
  },
  "counter-scoring": {
    id: "counter-scoring",
    title: "Карточка счётчика в разметке",
    route: "/onboarding",
    locale: "ru-RU",
    preset: "scoring-n5",
  },
  "counters-detail": {
    id: "counters-detail",
    title: "Детальная карточка счётного суффикса",
    route: "/counters/:id",
    locale: "ru-RU",
    preset: "counter-due",
  },
  "home-due-counter": {
    id: "home-due-counter",
    title: "Счётчик просроченных карточек",
    route: "/home",
    locale: "ru-RU",
    preset: "onboarded",
  },
  "home-accuracy-chart": {
    id: "home-accuracy-chart",
    title: "График точности за месяц",
    route: "/home",
    locale: "ru-RU",
    preset: "onboarded-history",
  },
  "feedback-form": {
    id: "feedback-form",
    title: "Форма обратной связи в уроке",
    route: "/lesson",
    locale: "ru-RU",
    preset: "with-word",
  },
  "token-popup": {
    id: "token-popup",
    title: "Поповер токена с кнопкой жалобы",
    route: "/phrases",
    locale: "ru-RU",
    preset: "onboarded",
  },
  "jft-card": {
    id: "jft-card",
    title: "Карточка готовности JFT-Basic",
    route: "/home",
    locale: "ru-RU",
    preset: "irodori-nyuumon",
  },
  "home-jlpt-details": {
    id: "home-jlpt-details",
    title: "JLPT-карточка: раскрытая матрица категорий",
    route: "/home",
    locale: "ru-RU",
    preset: "onboarded",
  },
  "lesson-counter-pack": {
    id: "lesson-counter-pack",
    title: "Пачка связок счётного суффикса в уроке",
    route: "/lesson",
    locale: "ru-RU",
    preset: "counter-due",
  },
  "lesson-unification": {
    id: "lesson-unification",
    title: "Единый урок: знакомство, тренировка, тело",
    route: "/lesson",
    locale: "ru-RU",
    preset: "with-word",
  },
  "kanji-writing-practice": {
    id: "kanji-writing-practice",
    title: "Практика письма кандзи",
    route: "/kanji/:id",
    locale: "ru-RU",
    preset: "acquaintance-hand",
  },
  "lesson-review-body": {
    id: "lesson-review-body",
    title: "Тело повторения урока (due-профиль)",
    route: "/lesson",
    locale: "ru-RU",
    preset: "due-corpus",
  },
  "grammar-page": {
    id: "grammar-page",
    title: "Страница грамматики",
    route: "/grammar",
    locale: "ru-RU",
    preset: "acquaintance-hand",
  },
};

export function listDescriptors(): ScenarioDescriptor[] {
  return Object.values(descriptors).sort((a, b) => a.id.localeCompare(b.id));
}
