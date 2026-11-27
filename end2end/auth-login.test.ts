import assert from "node:assert/strict";
import { registerHooks } from "node:module";
import { test } from "node:test";
import type { Page } from "@playwright/test";

const end2endRoot = new URL("./", import.meta.url).href;

// The helpers import local modules without extensions. Node's ESM resolver
// does not add them; limit the hook to this package so node_modules CJS
// resolution stays untouched.
registerHooks({
  resolve(specifier, context, nextResolve) {
    const parent = context.parentURL ?? "";
    if (
      specifier.startsWith(".") &&
      !/\.[cm]?[jt]s$/i.test(specifier) &&
      parent.startsWith(end2endRoot) &&
      !parent.includes("/node_modules/")
    ) {
      return nextResolve(`${specifier}.ts`, context);
    }
    return nextResolve(specifier, context);
  },
});

interface FakeState {
  session: boolean;
  consent: boolean;
  reloaded: boolean;
}

interface WaitOptions {
  state?: "visible" | "hidden";
  timeout?: number;
}

class FakePage {
  readonly state: FakeState = {
    session: false,
    consent: false,
    reloaded: false,
  };
  reloadCalls = 0;
  formWaits = 0;

  asPage(): Page {
    return this as unknown as Page;
  }

  context(): { addInitScript(): Promise<void> } {
    return { addInitScript: async () => undefined };
  }

  async goto(): Promise<void> {
    // A retry reloads the app but keeps the session in localStorage.
  }

  async reload(): Promise<void> {
    this.reloadCalls += 1;
    this.state.reloaded = true;
  }

  async waitForURL(): Promise<void> {
    throw new Error("URL stayed on root");
  }

  async waitForTimeout(): Promise<void> {}

  async evaluate(): Promise<{ session: boolean }> {
    return { session: this.state.session };
  }

  getByTestId(testId: string): FakeLocator {
    return new FakeLocator(this, [testId]);
  }

  locator(): FakeLocator {
    return new FakeLocator(this, ["email-input", "password-input"]);
  }

  async fill(): Promise<void> {}

  async click(selector?: string): Promise<void> {
    if (selector?.includes("submit")) {
      this.state.session = true;
    }
  }
}

class FakeLocator {
  private readonly page: FakePage;
  private readonly testIds: string[];

  constructor(page: FakePage, testIds: string[]) {
    this.page = page;
    this.testIds = testIds;
  }

  or(other: FakeLocator): FakeLocator {
    return new FakeLocator(this.page, [...this.testIds, ...other.testIds]);
  }

  async waitFor(options: WaitOptions): Promise<void> {
    const visible = this.testIds.some((id) => this.isVisible(id, options));
    if (options.state === "hidden") {
      if (visible) throw new Error(`${this.testIds.join("/")} still visible`);
      return;
    }
    if (!visible) {
      throw new Error(`${this.testIds.join("/")} not visible`);
    }
  }

  async click(): Promise<void> {
    if (this.testIds.includes("login-password-toggle")) {
      this.page.formWaits += 1;
    }
  }

  private isVisible(testId: string, options: WaitOptions): boolean {
    const timeout = options.timeout ?? 0;
    switch (testId) {
      case "login-password-toggle":
        return !this.page.state.session;
      case "home-page":
        return false;
      case "home-content":
        return this.page.state.reloaded || timeout >= 300_000;
      case "onboarding-stepper":
        return this.page.state.reloaded;
      case "resource-consent-download-button":
        return this.page.state.consent;
      default:
        return !this.page.state.session;
    }
  }
}

test("shots accepts a settled home marker on root with an explicit budget", async () => {
  const { waitForLoginComplete } = await import("./helpers/auth.ts");
  const page = new FakePage();
  page.state.session = true;

  await waitForLoginComplete(page.asPage(), {
    expected: "app",
    settleTimeout: 300_000,
  } as never);

  assert.equal(page.reloadCalls, 0);
});

test("shots restores an authenticated session with one reload, not a form retry", async () => {
  const { uiLogin } = await import("./helpers/auth.ts");
  const page = new FakePage();

  const result = await uiLogin(page.asPage(), "fresh@origa.local", "secret", {
    baseURL: "http://localhost:1421",
    expected: "app",
    settleTimeout: 90_000,
    recoverOnStall: true,
  } as never);

  assert.equal(page.reloadCalls, 1);
  assert.equal(page.formWaits, 1);
  assert.equal(result.restarted, true);
});

test("the consent screen is terminal for the consent scenario", async () => {
  const { waitForLoginComplete } = await import("./helpers/auth.ts");
  const page = new FakePage();
  page.state.session = true;
  page.state.consent = true;

  await waitForLoginComplete(page.asPage(), {
    expected: "consent",
    settleTimeout: 90_000,
  } as never);

  assert.equal(page.reloadCalls, 0);
});
