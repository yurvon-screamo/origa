/**
 * Reveal and advance controls of the review-body card walk.
 *
 * The arrays live here instead of `review-body.ts` so a scenario that
 * walks cards does not depend on a helper export from a parallel PR.
 */
export const REVEALS = [
  "lesson-show-answer-btn",
  "quiz-option-0",
  "yesno-no-btn",
  "audio-recall-reveal-btn",
  "lesson-example-reveal",
  "counter-binding-show-answer-btn",
];

export const ADVANCES = [
  "lesson-card-next-btn",
  "quiz-submit-btn",
  "lesson-rating-btn-again",
  "lesson-phrase-rating-understood",
  "lesson-phrase-rating-did-not-understand",
  "yesno-yes-btn",
  "yesno-no-btn",
  "audio-recall-know-btn",
  "lesson-example-understood",
  "lesson-example-next",
];
