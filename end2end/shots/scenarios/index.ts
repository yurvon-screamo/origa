import type { ShotScenario } from "../scenario.ts";
import { counterScoringScenario } from "./counter-scoring.ts";
import { feedbackFormScenario } from "./feedback-form.ts";
import { homeDueCounterScenario } from "./home-due-counter.ts";
import { homeInsightsScenario } from "./home-insights.ts";
import { homeScenario } from "./home.ts";
import { jftCardScenario } from "./jft-card.ts";
import { lessonAcquaintanceGrammarScenario } from "./lesson-acquaintance-grammar.ts";
import { lessonExampleRecallScenario } from "./lesson-example-recall.ts";
import { lessonExampleScenario } from "./lesson-example.ts";
import { lessonReadingQuizScenario } from "./lesson-reading-quiz.ts";
import { lessonReviewBodyScenario } from "./lesson-review-body.ts";
import { lessonUnificationScenario } from "./lesson-unification.ts";
import { stitchSessionScenario } from "./stitch-session.ts";
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
    [lessonExampleScenario.id]: lessonExampleScenario,
    [lessonExampleRecallScenario.id]: lessonExampleRecallScenario,
    [counterScoringScenario.id]: counterScoringScenario,
    [homeDueCounterScenario.id]: homeDueCounterScenario,
    [homeInsightsScenario.id]: homeInsightsScenario,
    [stitchSessionScenario.id]: stitchSessionScenario,
    [feedbackFormScenario.id]: feedbackFormScenario,
    [tokenPopupScenario.id]: tokenPopupScenario,
    [jftCardScenario.id]: jftCardScenario,
    [lessonAcquaintanceGrammarScenario.id]: lessonAcquaintanceGrammarScenario,
    [lessonUnificationScenario.id]: lessonUnificationScenario,
    [lessonReviewBodyScenario.id]: lessonReviewBodyScenario,
    [lessonReadingQuizScenario.id]: lessonReadingQuizScenario,
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
