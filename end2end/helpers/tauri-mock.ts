import type { BrowserContext } from "@playwright/test";

/**
 * Transport mock for the Tauri WebView boundary, built to the exact surface
 * `origa_ui/src/core/tauri.rs` reads:
 *
 * - `__TAURI__.event.listen(name, handler)` — oauth listener registers on the
 *   host re-emit channel `deep-link-received` (plain string payload);
 * - `__TAURI__.core.invoke(cmd, args)` — `get_current_deep_link` (cold poll +
 *   resume poll) and the `auth_store_*` session/verifier store commands;
 * - `__TAURI__.opener.openUrl(url)` — external browser hand-off (OAuth start).
 *
 * Everything else resolves `null` (never rejects) so the app boots cleanly
 * in "Tauri mode" inside a plain chromium: no session, no auth — the exact
 * state the #1482 bug needs (unauthenticated user on /login).
 *
 * The mock handle is exposed as `window.__tauriMock` for steps to dispatch
 * events and read state. Unknown invoke commands resolve `null` — a new
 * app-side command must not crash the harness.
 */
export interface TauriMockWindow {
  __tauriMock: {
    currentDeepLink: string;
    store: Map<string, string>;
    events: Map<string, Array<(event: { payload: unknown }) => void>>;
    openedUrls: string[];
  };
}

export interface TauriMockOptions {
  /** Value the plugin reports through `get_current_deep_link` (cold-poll input). */
  currentDeepLink?: string;
  /** Seed the PKCE verifier so a real auth-callback reaches the token exchange. */
  pkceVerifier?: string;
}

export async function installTauriMock(
  context: BrowserContext,
  options: TauriMockOptions = {},
): Promise<void> {
  await context.addInitScript(
    ({ currentDeepLink, pkceVerifier }) => {
      const mock = {
        currentDeepLink,
        store: new Map<string, string>(),
        events: new Map<string, Array<(event: { payload: unknown }) => void>>(),
        openedUrls: [] as string[],
      };
      if (pkceVerifier !== undefined) {
        // Key mirrors PKCE_VERIFIER_KEY in origa_ui/src/repository/session.rs.
        mock.store.set("pkce_verifier", pkceVerifier);
      }
      (window as unknown as { __tauriMock: unknown }).__tauriMock = mock;
      (window as unknown as { __TAURI__: unknown }).__TAURI__ = {
        core: {
          invoke: (cmd: string, args?: { key?: string; value?: string }) => {
            if (cmd === "get_current_deep_link") {
              return Promise.resolve(mock.currentDeepLink);
            }
            if (cmd === "auth_store_get") {
              return Promise.resolve(mock.store.get(args?.key ?? "") ?? null);
            }
            if (cmd === "auth_store_set") {
              mock.store.set(args?.key ?? "", args?.value ?? "");
              return Promise.resolve(null);
            }
            if (cmd === "auth_store_delete") {
              mock.store.delete(args?.key ?? "");
              return Promise.resolve(null);
            }
            return Promise.resolve(null);
          },
        },
        event: {
          listen: (
            name: string,
            handler: (event: { payload: unknown }) => void,
          ) => {
            if (!mock.events.has(name)) mock.events.set(name, []);
            mock.events.get(name)!.push(handler);
            return Promise.resolve(() => {});
          },
        },
        opener: {
          openUrl: (url: string) => {
            mock.openedUrls.push(url);
            return Promise.resolve();
          },
        },
      };
    },
    {
      currentDeepLink: options.currentDeepLink ?? "",
      pkceVerifier: options.pkceVerifier,
    },
  );
}

/** Dispatches a fake `deep-link-received` event to every registered handler. */
export async function dispatchDeepLink(
  context: BrowserContext,
  url: string,
): Promise<void> {
  const pages = context.pages();
  await pages[0].evaluate((u) => {
    const mock = (window as unknown as TauriMockWindow).__tauriMock;
    for (const handler of mock.events.get("deep-link-received") ?? []) {
      handler({ payload: u });
    }
  }, url);
}
