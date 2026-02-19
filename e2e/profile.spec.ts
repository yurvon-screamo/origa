import { test, expect } from '@playwright/test';
import { LoginPage, HomePage, ProfilePage } from './pages';

test.describe('Страница профиля', () => {
  let loginPage: LoginPage;
  let homePage: HomePage;
  let profilePage: ProfilePage;

  test.beforeEach(async ({ page }) => {
    loginPage = new LoginPage(page);
    homePage = new HomePage(page);
    profilePage = new ProfilePage(page);

    await loginPage.goto();
    await loginPage.loginAndNavigate('demo');
  });

  test('должен отобразить все элементы страницы профиля', async ({ page }) => {
    await profilePage.goto();
    await profilePage.expectVisible();
  });

  test('должен отобразить имя пользователя в заголовке', async ({ page }) => {
    await profilePage.goto();
    await profilePage.expectHeadingContains('demo');
  });

  test('должен отобразить имя пользователя в поле ввода', async ({ page }) => {
    await profilePage.goto();
    await profilePage.expectUsername('demo');
  });

  test('должен позволить изменить Duolingo токен', async ({ page }) => {
    await profilePage.goto();
    const newToken = 'test-token-12345';

    await profilePage.setDuolingoToken(newToken);
    await profilePage.expectDuolingoToken(newToken);
  });

  test('должен позволить переключить напоминания', async ({ page }) => {
    await profilePage.goto();

    await profilePage.toggleReminders();
    await profilePage.expectRemindersEnabled(false);
  });

  test('должен сохранить изменения', async ({ page }) => {
    await profilePage.goto();
    const newToken = 'new-duolingo-token';

    await profilePage.setDuolingoToken(newToken);
    await profilePage.saveChanges();
    await profilePage.expectDuolingoToken(newToken);
  });

  test('должен выйти из аккаунта', async ({ page }) => {
    await profilePage.goto();
    await profilePage.logout();

    await expect(page).toHaveURL('/');
    await loginPage.expectVisible();
  });

  test('должен показать кнопку сохранения в состоянии загрузки', async ({ page }) => {
    await profilePage.goto();
    const newToken = 'save-state-token';

    await profilePage.setDuolingoToken(newToken);
    await profilePage.saveButton.click();

    await expect(profilePage.saveButton).toHaveText('Сохранение...');
  });
});
