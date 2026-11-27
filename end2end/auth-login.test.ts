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
  settled: boolean;
}

interface WaitOptions {
  state?: "visible" | "hidden";
  timeout?: number;
}

class FakePage {
  readonly state: FakeState = {
    session: false,
    consent: false,
    settled: false,
  };
  gotoCalls = 0;
  formWaits = 0;
  urlWaits = 0;
  waits: string[] = [];

  asPage(): Page {
    return this as unknown as Page;
  }

  context(): { addInitScript(): Promise<void> } {
    return { addInitScript: async () => undefined };
  }

  async goto(): Promise<void> {
    this.gotoCalls += 1;
  }

  async waitForURL(): Promise<void> {
    this.urlWaits += 1;
    throw new Error("URL stayed on root");
  }

  async waitForTimeout(): Promise<void> {}

  async evaluate(): Promise<Record<string, boolean | string>> {
    return {
      url: "/",
      session: this.state.session,
      consent: this.state.consent,
      wizard: this.state.settled,
      content: this.state.settled,
      toggle: !this.state.session,
      loading: false,
      loadError: false,
    };
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
      this.state.settled = true;
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
    this.page.waits.push(`${options.state}:${this.testIds.join("/")}`);
    const visible = this.testIds.some((id) => this.isVisible(id));
    if (options.state === "hidden") {
      if (visible) throw new Error(`${this.testIds.join("/")} still visible`);
      return;
    }
    if (!visible) {
      throw new Error(`${this.testIds.join("/")} not visible`);
    }
  }

  async click(): Promise<void> {
    this.page.formWaits += 1;
  }

  private isVisible(testId: string): boolean {
    switch (testId) {
      case "login-password-toggle":
        return !this.page.state.session;
      case "home-page":
        return false;
      case "home-content":
      case "onboarding-stepper":
        return this.page.state.settled;
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
  page.state.settled = true;

  const result = await waitForLoginComplete(page.asPage(), {
    expected: "app",
    settleTimeout: 300_000,
  });

  assert.equal(result.elapsedMs >= 0, true);
  assert.equal(page.formWaits, 0);
});

test("uiLogin waits once for a stored session instead of retrying the form", async () => {
  const { uiLogin } = await import("./helpers/auth.ts");
  const page = new FakePage();
  page.state.session = true;
  page.state.settled = true;

  const result = await uiLogin(page.asPage(), "fresh@origa.local", "secret", {
    baseURL: "http://localhost:1421",
    expected: "app",
  });

  assert.equal(page.gotoCalls, 2);
  assert.equal(page.formWaits, 0);
  assert.equal(result.elapsedMs >= 0, true);
});

test("a stalled login reports elapsed time, page loads and overlay state", async () => {
  const { waitForLoginComplete } = await import("./helpers/auth.ts");
  const page = new FakePage();
  page.state.session = true;

  await assert.rejects(
    waitForLoginComplete(page.asPage(), {
      expected: "app",
      settleTimeout: 5,
      navigations: 2,
    }),
    (error: unknown) => {
      const message = error instanceof Error ? error.message : String(error);
      assert.match(message, /in \d+ ms after 2 page load\(s\)/);
      assert.match(message, /session=true/);
      assert.match(message, /loading=false loadError=false/);
      return true;
    },
  );
});

test("the consent screen is terminal for the consent scenario", async () => {
  const { waitForLoginComplete } = await import("./helpers/auth.ts");
  const page = new FakePage();
  page.state.session = true;
  page.state.consent = true;

  const result = await waitForLoginComplete(page.asPage(), {
    expected: "consent",
    settleTimeout: 90_000,
  });

  assert.equal(result.elapsedMs >= 0, true);
  assert.equal(page.formWaits, 0);
});

test("the argument-free BDD contract still falls back from URL to the root marker", async () => {
  const { waitForLoginComplete } = await import("./helpers/auth.ts");
  const page = new FakePage();
  page.state.session = true;
  page.state.settled = true;

  const result = await waitForLoginComplete(page.asPage());

  assert.equal(page.urlWaits, 1);
  assert.deepEqual(page.waits, [
    "hidden:login-password-toggle",
    "visible:home-page/onboarding-stepper",
  ]);
  assert.equal(result.elapsedMs >= 0, true);
});

test("a clean start submits the form once and settles on the app marker", async () => {
  const { uiLogin } = await import("./helpers/auth.ts");
  const page = new FakePage();

  const result = await uiLogin(page.asPage(), "fresh@origa.local", "secret", {
    baseURL: "http://localhost:1421",
    expected: "app",
  });

  assert.equal(page.gotoCalls, 1);
  assert.equal(page.formWaits, 1);
  assert.equal(result.elapsedMs >= 0, true);
});

test("a stored session that never settles fails after one wait, not three retries", async () => {
  const { uiLogin } = await import("./helpers/auth.ts");
  const page = new FakePage();
  page.state.session = true;
  page.state.settled = false;

  await assert.rejects(
    uiLogin(page.asPage(), "fresh@origa.local", "secret", {
      baseURL: "http://localhost:1421",
      expected: "app",
      settleTimeout: 5,
    }),
    (error: unknown) => {
      const message = error instanceof Error ? error.message : String(error);
      assert.match(message, /Login did not reach app terminal/);
      assert.match(message, /session=true/);
      return true;
    },
  );

  assert.equal(page.gotoCalls, 2);
  assert.equal(page.formWaits, 0);
  assert.equal(
    page.waits.filter((entry) => entry.startsWith("visible:home-content"))
      .length,
    1,
  );
});
