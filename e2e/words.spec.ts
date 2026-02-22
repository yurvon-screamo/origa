import { test, expect } from '@playwright/test';
import { LoginPage, WordsPage } from './pages';

test.describe('Страница слов', () => {
  let loginPage: LoginPage;
  let wordsPage: WordsPage;

  test.beforeEach(async ({ page }) => {
    loginPage = new LoginPage(page);
    wordsPage = new WordsPage(page);

    await loginPage.goto();
    await loginPage.loginAndNavigate('demo');
  });

  test('должен отобразить все элементы страницы слов', async ({ page }) => {
    await wordsPage.goto();
    await wordsPage.expectVisible();
    await wordsPage.expectFiltersVisible();
  });

  test('должен отобразить список карточек слов', async ({ page }) => {
    await wordsPage.goto();
    await wordsPage.expectNotEmpty();
  });

  test('должен фильтровать слова по ключевому слову', async ({ page }) => {
    await wordsPage.goto();

    const initialCount = await wordsPage.getCardsCount();

    await wordsPage.search('Кошка');
    await wordsPage.expectCardVisible('猫');

    await wordsPage.clearSearch();
    const finalCount = await wordsPage.getCardsCount();
    expect(finalCount).toBe(initialCount);
  });

  test('должен фильтровать слова по переводу', async ({ page }) => {
    await wordsPage.goto();

    await wordsPage.search('Собака');
    await wordsPage.expectCardVisible('犬');
  });

  test('должен показать пустое состояние при отсутствии результатов', async ({ page }) => {
    await wordsPage.goto();

    await wordsPage.search('несуществующееслово12345');
    await wordsPage.expectEmptyState();
  });

  test('должен переключаться между фильтрами', async ({ page }) => {
    await wordsPage.goto();

    await wordsPage.clickFilter('new');
    await wordsPage.expectFilterActive('new');

    await wordsPage.clickFilter('hard');
    await wordsPage.expectFilterActive('hard');

    await wordsPage.clickFilter('all');
    await wordsPage.expectFilterActive('all');
  });

  test('должен отображать счетчики в фильтрах', async ({ page }) => {
    await wordsPage.goto();

    const allCount = await wordsPage.getFilterCount('all');
    expect(allCount).toBeGreaterThan(0);
  });

  test('должен вернуться на главную страницу', async ({ page }) => {
    await wordsPage.goto();
    await wordsPage.goBack();
    await expect(page).toHaveURL('/home');
  });
});
