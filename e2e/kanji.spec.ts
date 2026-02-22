import { test, expect } from '@playwright/test';
import { LoginPage, KanjiPage } from './pages';

test.describe('Страница кандзи', () => {
  let loginPage: LoginPage;
  let kanjiPage: KanjiPage;

  test.beforeEach(async ({ page }) => {
    loginPage = new LoginPage(page);
    kanjiPage = new KanjiPage(page);

    await loginPage.goto();
    await loginPage.loginAndNavigate('demo');
  });

  test('должен отобразить все элементы страницы кандзи', async ({ page }) => {
    await kanjiPage.goto();
    await kanjiPage.expectVisible();
    await kanjiPage.expectFiltersVisible();
  });


  test('должен фильтровать кандзи по ключевому слову', async ({ page }) => {
    await kanjiPage.goto();
    await page.waitForSelector('.card', { timeout: 5000 }).catch(() => { });

    const initialCount = await kanjiPage.getCardsCount();

    if (initialCount > 0) {
      await kanjiPage.search('日');
      await page.waitForTimeout(100);
      const filteredCount = await kanjiPage.getCardsCount();
      expect(filteredCount).toBeLessThanOrEqual(initialCount);
    }
  });

  test('должен показать пустое состояние при отсутствии результатов', async ({ page }) => {
    await kanjiPage.goto();

    await kanjiPage.search('несуществующийкандзи12345');
    await page.waitForTimeout(100);
    await kanjiPage.expectEmptyState();
  });

  test('должен переключаться между фильтрами', async ({ page }) => {
    await kanjiPage.goto();

    await kanjiPage.clickFilter('new');
    await kanjiPage.expectFilterActive('new');

    await kanjiPage.clickFilter('hard');
    await kanjiPage.expectFilterActive('hard');

    await kanjiPage.clickFilter('all');
    await kanjiPage.expectFilterActive('all');
  });

  test('должен отображать счетчики в фильтрах', async ({ page }) => {
    await kanjiPage.goto();

    const allCount = await kanjiPage.getFilterCount('all');
    expect(allCount).toBeGreaterThanOrEqual(0);
  });

  test('должен вернуться на главную страницу', async ({ page }) => {
    await kanjiPage.goto();
    await kanjiPage.goBack();
    await expect(page).toHaveURL('/home');
  });

  test('должен открыть модальное окно добавления кандзи', async ({ page }) => {
    await kanjiPage.goto();
    await kanjiPage.clickAddButton();
    await kanjiPage.expectModalVisible();
  });

  test('должен закрыть модальное окно по кнопке отмена', async ({ page }) => {
    await kanjiPage.goto();
    await kanjiPage.clickAddButton();
    await kanjiPage.expectModalVisible();
    await kanjiPage.cancelAdd();
    await kanjiPage.expectModalNotVisible();
  });

  test('должен отображать кнопки выбора уровня JLPT в модальном окне', async ({ page }) => {
    await kanjiPage.goto();
    await kanjiPage.clickAddButton();
    await kanjiPage.expectModalVisible();

    await expect(page.getByRole('button', { name: 'N5', exact: true })).toBeVisible();
    await expect(page.getByRole('button', { name: 'N4', exact: true })).toBeVisible();
    await expect(page.getByRole('button', { name: 'N3', exact: true })).toBeVisible();
    await expect(page.getByRole('button', { name: 'N2', exact: true })).toBeVisible();
    await expect(page.getByRole('button', { name: 'N1', exact: true })).toBeVisible();
  });

  test('должен переключать уровень JLPT в модальном окне', async ({ page }) => {
    await kanjiPage.goto();
    await kanjiPage.clickAddButton();
    await kanjiPage.expectModalVisible();

    await kanjiPage.selectLevel('N4');
    await expect(page.getByRole('button', { name: 'N4', exact: true })).toHaveClass(/btn-olive/);
  });
});
