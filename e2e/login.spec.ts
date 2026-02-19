import { test, expect } from '@playwright/test';
import { LoginPage, HomePage } from './pages';

test.describe('Вход в приложение', () => {
  let loginPage: LoginPage;
  let homePage: HomePage;

  test.beforeEach(async ({ page }) => {
    loginPage = new LoginPage(page);
    homePage = new HomePage(page);
  });

  test('должен войти с username "demo" и перенаправиться на домашнюю страницу', async ({ page }) => {
    await loginPage.goto();
    
    await loginPage.loginAndNavigate('demo');
    
    await homePage.expectVisible();
    await expect(page).toHaveURL('/home');
  });

  test('должен показать ошибку при пустом username', async ({ page }) => {
    await loginPage.goto();
    
    await loginPage.login('');
    
    await loginPage.expectErrorMessage('Введите имя пользователя');
    await expect(page).toHaveURL('/');
  });

  test('должен перенаправиться на home после ввода валидного username', async ({ page }) => {
    await loginPage.goto();
    
    await loginPage.usernameInput.fill('testuser');
    await loginPage.loginButton.click();
    
    await page.waitForURL('/home');
    await expect(page).toHaveURL('/home');
    await homePage.expectVisible();
  });

  test('должен позволить вход с Enter', async ({ page }) => {
    await loginPage.goto();
    
    await loginPage.usernameInput.fill('demo');
    await loginPage.usernameInput.press('Enter');
    
    await page.waitForURL('/home');
    await expect(page).toHaveURL('/home');
    await homePage.expectVisible();
  });
});

test.describe('Домашняя страница', () => {
  test('должен отобразить карточки статистики', async ({ page }) => {
    const loginPage = new LoginPage(page);
    const homePage = new HomePage(page);
    
    await loginPage.goto();
    await loginPage.loginAndNavigate('demo');
    
    await homePage.expectVisible();
    
    const kanjiCount = await homePage.getKanjiCount();
    const wordsCount = await homePage.getWordsCount();
    const level = await homePage.getLevel();
    
    expect(kanjiCount).toBeTruthy();
    expect(wordsCount).toBeTruthy();
    expect(level).toBe('N5');
  });
});
