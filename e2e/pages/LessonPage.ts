import { Page, Locator, expect } from '@playwright/test';
import { BasePage } from './BasePage';

export class LessonPage extends BasePage {
  readonly progressBar: Locator;
  readonly cardContainer: Locator;
  readonly answerButton: Locator;
  readonly ratingAgain: Locator;
  readonly ratingHard: Locator;
  readonly ratingGood: Locator;
  readonly ratingEasy: Locator;
  readonly completeScreen: Locator;
  readonly nextLessonButton: Locator;
  readonly goHomeButton: Locator;
  readonly noCardsMessage: Locator;
  readonly quizOptions: Locator;

  constructor(page: Page) {
    super(page);
    this.progressBar = page.locator('.progress, [role="progressbar"]').first();
    this.cardContainer = page.locator('.lesson-card, [data-testid="lesson-card"]').first();
    this.answerButton = page.getByRole('button', { name: /показать ответ|ответ/i });
    this.ratingAgain = page.getByRole('button', { name: /не знаю|\[1\]/i });
    this.ratingHard = page.getByRole('button', { name: /плохо|\[2\]/i });
    this.ratingGood = page.getByRole('button', { name: /знаю|\[3\]/i });
    this.ratingEasy = page.getByRole('button', { name: /идеально|\[4\]/i });
    this.completeScreen = page.getByText(/пройдено|урок завершен/i);
    this.nextLessonButton = page.getByRole('button', { name: /следующий урок/i });
    this.goHomeButton = page.getByRole('button', { name: /на главную/i });
    this.noCardsMessage = page.getByText(/нет карточек для изучения/i);
    this.quizOptions = page.locator('.quiz-option, [data-testid="quiz-option"]');
  }

  async goto(mode: 'lesson' | 'fixation' = 'lesson') {
    const url = mode === 'fixation' ? '/lesson?mode=fixation' : '/lesson';
    await super.goto(url);
    await this.waitForCardsOrComplete();
  }

  async waitForCardsOrComplete() {
    await super.waitForLoading();
    try {
      await this.cardContainer.waitFor({ state: 'visible', timeout: 30000 });
    } catch {
      await this.noCardsMessage.waitFor({ state: 'visible', timeout: 10000 }).catch(() => {});
      await this.completeScreen.waitFor({ state: 'visible', timeout: 10000 }).catch(() => {});
    }
  }

  async showAnswer() {
    await this.answerButton.click();
    await this.expectRatingButtonsVisible();
  }

  async rate(rating: 'again' | 'hard' | 'good' | 'easy') {
    const button = {
      again: this.ratingAgain,
      hard: this.ratingHard,
      good: this.ratingGood,
      easy: this.ratingEasy,
    }[rating];
    await button.click();
    await this.page.waitForTimeout(500);
  }

  async expectRatingButtonsVisible() {
    await expect(this.ratingAgain).toBeVisible({ timeout: 30000 });
    await expect(this.ratingGood).toBeVisible({ timeout: 30000 });
  }

  async expectCardVisible() {
    await expect(this.cardContainer).toBeVisible({ timeout: 30000 });
  }

  async expectCompleteScreen() {
    await expect(this.completeScreen).toBeVisible({ timeout: 30000 });
  }

  async expectNoCards() {
    await expect(this.noCardsMessage).toBeVisible({ timeout: 30000 });
  }

  async goToNextLesson() {
    await this.nextLessonButton.click();
    await this.waitForCardsOrComplete();
  }

  async goHome() {
    await this.goHomeButton.click();
    await this.waitForUrl('/home');
  }

  async completeFullLesson(maxCards: number = 50): Promise<number> {
    let completed = 0;
    while (completed < maxCards) {
      await this.waitForCardsOrComplete();
      
      const isComplete = await this.completeScreen.isVisible().catch(() => false);
      const hasNoCards = await this.noCardsMessage.isVisible().catch(() => false);
      
      if (isComplete || hasNoCards) break;
      
      const hasCard = await this.cardContainer.isVisible().catch(() => false);
      if (!hasCard) break;
      
      const hasAnswerButton = await this.answerButton.isVisible().catch(() => false);
      if (hasAnswerButton) {
        await this.showAnswer();
      }
      
      await this.rate('good');
      completed++;
    }
    return completed;
  }

  async selectQuizOption(index: number) {
    const options = await this.quizOptions.all();
    if (options[index]) {
      await options[index].click();
    }
  }
}
