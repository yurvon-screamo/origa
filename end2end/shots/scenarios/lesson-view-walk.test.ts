import assert from "node:assert/strict";
import test from "node:test";

import type { Page } from "@playwright/test";

import { walkLessonCards } from "./lesson-view-walk.ts";

function action(visible: boolean, onClick: () => void, clickFails = false) {
  return {
    async waitFor() {
      if (!visible) throw new Error("not visible");
      return true;
    },
    async click() {
      onClick();
      if (clickFails) throw new Error("detached element");
    },
    async isVisible() {
      return visible;
    },
  };
}

function fakePage(clicked: string[], clickFails = false): Page {
  return {
    // The boot-splash wait resolves immediately in these tests.
    locator: () => ({
      ...action(true, () => undefined),
      first: () => action(false, () => undefined),
    }),
    getByTestId: (id: string) => {
      if (id === "lesson-complete-screen")
        return action(false, () => undefined);
      if (id === "lesson-show-answer-btn" || id === "quiz-option-0") {
        return action(true, () => clicked.push(`reveal:${id}`), clickFails);
      }
      return action(true, () => clicked.push(`advance:${id}`));
    },
  } as unknown as Page;
}

test("walkLessonCards stops as soon as the discriminator is found", async () => {
  const clicked: string[] = [];
  let visits = 0;
  const found = await walkLessonCards(fakePage(clicked), async () => {
    visits += 1;
    return true;
  });
  assert.equal(found, true);
  assert.equal(visits, 1);
  assert.deepEqual(clicked, []);
});

test("walkLessonCards advances cards until the discriminator appears", async () => {
  const clicked: string[] = [];
  let visits = 0;
  const found = await walkLessonCards(fakePage(clicked), async () => {
    visits += 1;
    return visits === 3;
  });
  assert.equal(found, true);
  assert.equal(visits, 3);
  assert.ok(clicked.some((item) => item.startsWith("reveal:")));
  assert.ok(clicked.some((item) => item.startsWith("advance:")));
});

test("a lost click is a miss, not a failed capture", async () => {
  const clicked: string[] = [];
  const found = await walkLessonCards(
    fakePage(clicked, true),
    async () => false,
    2,
  );
  assert.equal(found, false);
  assert.ok(clicked.some((item) => item.startsWith("reveal:")));
  assert.ok(clicked.some((item) => item.startsWith("advance:")));
});
