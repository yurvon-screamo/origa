import type { ShotScenario } from "../scenario.ts";
import { counterScoringScenario } from "./counter-scoring.ts";
import { countersDetailScenario } from "./counters-detail.ts";
import { feedbackFormScenario } from "./feedback-form.ts";
import { grammarPageScenario } from "./grammar-page.ts";
import { homeDueCounterScenario } from "./home-due-counter.ts";
import { homeAccuracyChartScenario } from "./home-accuracy-chart.ts";
import { homeJlptDetailsScenario } from "./home-jlpt-details.ts";
import { homeScenario } from "./home.ts";
import { jftCardScenario } from "./jft-card.ts";
import { kanjiWritingPracticeScenario } from "./kanji-writing-practice.ts";
import { lessonAcquaintanceGrammarScenario } from "./lesson-acquaintance-grammar.ts";
import { lessonAcquaintanceKanjiScenario } from "./lesson-acquaintance-kanji.ts";
import { lessonCounterPackScenario } from "./lesson-counter-pack.ts";
import { lessonExampleRecallScenario } from "./lesson-example-recall.ts";
import { lessonExampleScenario } from "./lesson-example.ts";
import { lessonReadingQuizScenario } from "./lesson-reading-quiz.ts";
import { lessonReviewBodyScenario } from "./lesson-review-body.ts";
import { lessonUnificationScenario } from "./lesson-unification.ts";
import { tokenPopupScenario } from "./token-popup.ts";
import { wordExamplesScenario } from "./word-examples.ts";

/**
 * The browser-side implementations of the catalog.
 *
 * The metadata lives in `shots/catalog.ts` — this map only adds the
 * steps that reach the screen.
 */
export const implementations: Record<string, ShotScenario> = {
  [homeScenario.id]: homeScenario,
  [wordExamplesScenario.id]: wordExamplesScenario,
  [kanjiWritingPracticeScenario.id]: kanjiWritingPracticeScenario,
  [lessonExampleScenario.id]: lessonExampleScenario,
  [lessonExampleRecallScenario.id]: lessonExampleRecallScenario,
  [counterScoringScenario.id]: counterScoringScenario,
  [homeDueCounterScenario.id]: homeDueCounterScenario,
  [homeAccuracyChartScenario.id]: homeAccuracyChartScenario,
  [homeJlptDetailsScenario.id]: homeJlptDetailsScenario,
  [feedbackFormScenario.id]: feedbackFormScenario,
  [tokenPopupScenario.id]: tokenPopupScenario,
  [jftCardScenario.id]: jftCardScenario,
  [lessonAcquaintanceGrammarScenario.id]: lessonAcquaintanceGrammarScenario,
  [lessonAcquaintanceKanjiScenario.id]: lessonAcquaintanceKanjiScenario,
  [lessonUnificationScenario.id]: lessonUnificationScenario,
  [lessonReviewBodyScenario.id]: lessonReviewBodyScenario,
  [lessonReadingQuizScenario.id]: lessonReadingQuizScenario,
  [lessonCounterPackScenario.id]: lessonCounterPackScenario,
  [grammarPageScenario.id]: grammarPageScenario,
  [countersDetailScenario.id]: countersDetailScenario,
};

export function getImplementation(id: string): ShotScenario {
  const scenario = implementations[id];
  if (!scenario) {
    throw new Error(
      `Unknown scenario "${id}". Known: ${Object.keys(implementations).sort().join(", ")}`,
    );
  }
  return scenario;
}
