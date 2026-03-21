import { test as setup } from '@playwright/test';
import path from 'path';
import fs from 'fs';
import { TEST_KANJI, TEST_VOCABULARY, TEST_GRAMMAR_RULES } from '../fixtures/test-data';

const authFile = path.join(__dirname, '..', 'playwright', '.auth', 'user.json');
const dataSeedFile = path.join(__dirname, '..', 'playwright', '.auth', 'data-seeded.json');

setup('seed test data', async ({ browser }) => {
  setup.setTimeout(180000);

  if (!fs.existsSync(authFile)) {
    throw new Error('Auth file not found. Run auth.setup.ts first.');
  }

  if (fs.existsSync(dataSeedFile)) {
    console.log('✅ Data already seeded, skipping...');
    return;
  }

  const context = await browser.newContext({
    storageState: authFile,
  });
  const page = await context.newPage();

  await setup.step('Navigate to home and verify auth', async () => {
    await page.goto('/home');
    await page.waitForLoadState('domcontentloaded', { timeout: 60000 }).catch(() => {});
    await page.waitForTimeout(3000);
  });

  await setup.step('Seed kanji cards', async () => {
    await page.goto('/kanji');
    await page.waitForLoadState('domcontentloaded', { timeout: 60000 }).catch(() => {});
    await page.waitForTimeout(3000);

    for (const kanji of TEST_KANJI.slice(0, 3)) {
      const addKanjiButton = page.getByRole('button', { name: /добавить.*кандзи/i });
      const hasButton = await addKanjiButton.isVisible({ timeout: 5000 }).catch(() => false);

      if (!hasButton) {
        console.log('⚠️ Add kanji button not found, skipping kanji seeding');
        break;
      }

      await addKanjiButton.click();
      await page.waitForTimeout(500);

      const modal = page.locator('[role="dialog"], .modal');
      const modalInput = modal.locator('input').first();

      if (await modal.isVisible({ timeout: 5000 }).catch(() => false)) {
        await modalInput.fill(kanji.character);
        await modal.getByRole('button', { name: /добавить|сохранить/i }).click();
        await page.waitForTimeout(1000);
        await modal.getByRole('button', { name: /закрыть|отмена/i }).click().catch(() => {});
      }
    }
  });

  await setup.step('Seed vocabulary cards', async () => {
    await page.goto('/words');
    await page.waitForLoadState('domcontentloaded', { timeout: 60000 }).catch(() => {});
    await page.waitForTimeout(3000);

    for (const vocab of TEST_VOCABULARY.slice(0, 3)) {
      const addWordButton = page.getByRole('button', { name: /добавить.*слово/i });
      const hasButton = await addWordButton.isVisible({ timeout: 5000 }).catch(() => false);

      if (!hasButton) {
        console.log('⚠️ Add word button not found, skipping vocabulary seeding');
        break;
      }

      await addWordButton.click();
      await page.waitForTimeout(500);

      const modal = page.locator('[role="dialog"], .modal');
      const modalInput = modal.locator('input').first();

      if (await modal.isVisible({ timeout: 5000 }).catch(() => false)) {
        await modalInput.fill(vocab.word);
        await modal.getByRole('button', { name: /добавить|сохранить/i }).click();
        await page.waitForTimeout(1000);
        await modal.getByRole('button', { name: /закрыть|отмена/i }).click().catch(() => {});
      }
    }
  });

  await setup.step('Seed grammar cards', async () => {
    await page.goto('/grammar');
    await page.waitForLoadState('domcontentloaded', { timeout: 60000 }).catch(() => {});
    await page.waitForTimeout(3000);

    for (const grammar of TEST_GRAMMAR_RULES.slice(0, 2)) {
      const addGrammarButton = page.getByRole('button', { name: /добавить.*грамматик/i });
      const hasButton = await addGrammarButton.isVisible({ timeout: 5000 }).catch(() => false);

      if (!hasButton) {
        console.log('⚠️ Add grammar button not found, skipping grammar seeding');
        break;
      }

      await addGrammarButton.click();
      await page.waitForTimeout(500);

      const modal = page.locator('[role="dialog"], .modal');
      const inputs = modal.locator('input, textarea');

      if (await modal.isVisible({ timeout: 5000 }).catch(() => false)) {
        const inputCount = await inputs.count();
        if (inputCount > 0) {
          await inputs.nth(0).fill(grammar.name);
        }
        if (inputCount > 1) {
          await inputs.nth(1).fill(grammar.structure);
        }
        if (inputCount > 2) {
          await inputs.nth(2).fill(grammar.meaning);
        }

        await modal.getByRole('button', { name: /добавить|сохранить/i }).click();
        await page.waitForTimeout(1000);
        await modal.getByRole('button', { name: /закрыть|отмена/i }).click().catch(() => {});
      }
    }
  });

  await setup.step('Mark data as seeded', async () => {
    const dir = path.dirname(dataSeedFile);
    if (!fs.existsSync(dir)) {
      fs.mkdirSync(dir, { recursive: true });
    }
    fs.writeFileSync(dataSeedFile, JSON.stringify({ seeded: true, timestamp: new Date().toISOString() }));
    console.log('✅ Data seeding completed');
  });

  await context.close();
});
