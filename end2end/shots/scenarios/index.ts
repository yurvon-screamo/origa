import type { ShotScenario } from "../scenario.ts";
import { acquaintanceCounterSlideScenario } from "./acquaintance-counter-slide.ts";
import { consentScenario } from "./consent.ts";
import { counterScoringScenario } from "./counter-scoring.ts";
import { countersDetailScenario } from "./counters-detail.ts";
import { countersScenario } from "./counters.ts";
import { examplesScenario } from "./examples.ts";
import { feedbackFormScenario } from "./feedback-form.ts";
import { grammarDetailScenario } from "./grammar-detail.ts";
import { grammarPageScenario } from "./grammar-page.ts";
import { homeDueCounterScenario } from "./home-due-counter.ts";
import { homeAccuracyChartScenario } from "./home-accuracy-chart.ts";
import { homeJlptDetailsScenario } from "./home-jlpt-details.ts";
import { homeScenario } from "./home.ts";
import { jftCardScenario } from "./jft-card.ts";
import { kanjiScenario } from "./kanji.ts";
import { kanjiWritingPracticeScenario } from "./kanji-writing-practice.ts";
import { lessonAcquaintanceGrammarScenario } from "./lesson-acquaintance-grammar.ts";
import { lessonEmptyScenario } from "./lesson-empty.ts";
import { lessonAcquaintanceKanjiScenario } from "./lesson-acquaintance-kanji.ts";
import { lessonAcquaintanceWordScenario } from "./lesson-acquaintance-word.ts";
import { lessonAudioRecallScenario } from "./lesson-audio-recall.ts";
import { lessonExampleAudioScenario } from "./lesson-example-audio.ts";
import { lessonPhraseListenScenario } from "./lesson-phrase-listen.ts";
import { lessonQuizMultiScenario } from "./lesson-quiz-multi.ts";
import { lessonCounterPackScenario } from "./lesson-counter-pack.ts";
import { lessonExampleRecallScenario } from "./lesson-example-recall.ts";
import { lessonExampleScenario } from "./lesson-example.ts";
import { lessonReadingQuizScenario } from "./lesson-reading-quiz.ts";
import { lessonReviewBodyScenario } from "./lesson-review-body.ts";
import { lessonUnificationScenario } from "./lesson-unification.ts";
import { lessonYesnoResultScenario } from "./lesson-yesno-result.ts";
import { listsEmptyScenario } from "./lists-empty.ts";
import { loginScenario } from "./login.ts";
import { onboardingImportProgressScenario } from "./onboarding-import-progress.ts";
import { onboardingStepsScenario } from "./onboarding-steps.ts";
import { phraseDashRunScenario } from "./phrase-dash-run.ts";
import { profileScenario } from "./profile.ts";
import { setsScenario } from "./sets.ts";
import { tokenPopupScenario } from "./token-popup.ts";
import { wordExamplesScenario } from "./word-examples.ts";
import { wordsAddScenario } from "./words-add.ts";
import { wordsGridScenario } from "./words-grid.ts";
import { wordsScenario } from "./words.ts";

/**
 * The browser-side implementations of the catalog.
 *
 * The metadata lives in `shots/catalog.ts` — this map only adds the
 * steps that reach the screen.
 */
export const implementations: Record<string, ShotScenario> = {
  [homeScenario.id]: homeScenario,
  [wordExamplesScenario.id]: wordExamplesScenario,
  [wordsScenario.id]: wordsScenario,
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
  [lessonAcquaintanceWordScenario.id]: lessonAcquaintanceWordScenario,
  [lessonUnificationScenario.id]: lessonUnificationScenario,
  [lessonReviewBodyScenario.id]: lessonReviewBodyScenario,
  [lessonReadingQuizScenario.id]: lessonReadingQuizScenario,
  [lessonYesnoResultScenario.id]: lessonYesnoResultScenario,
  [lessonCounterPackScenario.id]: lessonCounterPackScenario,
  [grammarPageScenario.id]: grammarPageScenario,
  [countersDetailScenario.id]: countersDetailScenario,
  [wordsGridScenario.id]: wordsGridScenario,
  [onboardingImportProgressScenario.id]: onboardingImportProgressScenario,
  [phraseDashRunScenario.id]: phraseDashRunScenario,
  [profileScenario.id]: profileScenario,
  [grammarDetailScenario.id]: grammarDetailScenario,
  [wordsAddScenario.id]: wordsAddScenario,
  [loginScenario.id]: loginScenario,
  [consentScenario.id]: consentScenario,
  [examplesScenario.id]: examplesScenario,
  [countersScenario.id]: countersScenario,
  [kanjiScenario.id]: kanjiScenario,
  [setsScenario.id]: setsScenario,
  [lessonEmptyScenario.id]: lessonEmptyScenario,
  [listsEmptyScenario.id]: listsEmptyScenario,
  [onboardingStepsScenario.id]: onboardingStepsScenario,
  [acquaintanceCounterSlideScenario.id]: acquaintanceCounterSlideScenario,
  [lessonExampleAudioScenario.id]: lessonExampleAudioScenario,
  [lessonPhraseListenScenario.id]: lessonPhraseListenScenario,
  [lessonAudioRecallScenario.id]: lessonAudioRecallScenario,
  [lessonQuizMultiScenario.id]: lessonQuizMultiScenario,
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
