import { expect, type Page } from "@playwright/test";
import { When, Then } from "../fixtures";
import { LOCAL_CDN_PORT } from "../../shots/config";

/**
 * Offline bundle precache scenarios (#1451): the consent-gated profile
 * download must ride the deflated v3 blobs (`phrases/*.v3.rkyv`) instead of
 * the raw phrase JSON, and after extraction the Cache API must not keep the
 * bundle copies — the extracted chunks are the single offline source (the
 * double-storage defect is a per-device ~79 MB regression).
 *
 * A second open of the profile must restore the downloaded state from the
 * cache marker without refetching anything (idempotent precache).
 */

/** Cache API store owned by the dictionary layer (cdn_provider.rs). */
const CDN_CACHE_NAME = "origa-cdn-v1";

/** Marker key written after a successful pass (precache_loader.rs,
 * BUNDLE_DOWNLOADED_KEY v3 — keep in sync). */
const BUNDLE_DOWNLOADED_MARKER = "__origa_bundle_downloaded_v3__";

/** CDN origin under test — same override chain as the CDN webServer. */
const CDN_ORIGIN = `http://localhost:${LOCAL_CDN_PORT}`;

/** The download card lives on /profile (OfflineBundleCard). */
async function openProfile(page: Page): Promise<void> {
  await page.goto("/profile", { waitUntil: "domcontentloaded" });
  await expect(page.getByTestId("download-bundle-btn")).toBeVisible({
    timeout: 60_000,
  });
}

/** Every Cache API request URL in the app's CDN store. */
async function cdnCacheKeys(page: Page): Promise<string[]> {
  return page.evaluate(async (cacheName) => {
    const cache = await caches.open(cacheName);
    const requests = await cache.keys();
    return requests.map((request) => request.url);
  }, CDN_CACHE_NAME);
}

When(
  "пользователь скачивает офлайн-пакет в профиле",
  async ({ page, cdnRequestLog }) => {
    await openProfile(page);

    // Attach the listener before the click, so the log covers the whole
    // download+extraction pass (click → fetch → Cache API put/delete).
    page.on("request", (request) => cdnRequestLog.push(request.url()));

    await page.getByTestId("download-bundle-btn").click();

    // The full ~145 MB pass over localhost: bounded by the slow-startup
    // budget rather than the default scenario timeout.
    await expect(page.getByText("Пакет скачан")).toBeVisible({
      timeout: 420_000,
    });
    // The progress block detaches when the pass completes.
    await expect(page.getByTestId("bundle-progress")).toBeHidden({
      timeout: 30_000,
    });
  },
);

Then(
  "фразовый precache запросил блобы вместо JSON",
  async ({ cdnRequestLog }) => {
    const cdnRequests = cdnRequestLog.filter((url) => url.includes(CDN_ORIGIN));

    const bundleBlobs = cdnRequests.filter((url) =>
      /phrases\/data_bundle_\d+\.v3\.rkyv$/.test(url),
    );
    expect(
      bundleBlobs,
      `all 6 bundle blobs must be fetched, got: ${bundleBlobs.join(", ")}`,
    ).toHaveLength(6);

    const phraseJson = cdnRequests.filter((url) =>
      /phrases\/(phrase_index|data_bundle_\d+)\.json$/.test(url),
    );
    expect(
      phraseJson,
      `precache must not fetch phrase JSON (blob-only contract), got: ${phraseJson.join(", ")}`,
    ).toHaveLength(0);

    // The legacy v2 phrase blob never rides the new path.
    expect(
      cdnRequests.some((url) => url.endsWith("phrases/phrase_index.rkyv")),
      "the legacy v2 phrase blob must not be fetched",
    ).toBe(false);
  },
);

Then("бандл-копии не хранятся в кэше после распаковки", async ({ page }) => {
  const keys = await cdnCacheKeys(page);

  const bundleCopies = keys.filter((url) =>
    url.includes("phrases/data_bundle_"),
  );
  expect(
    bundleCopies,
    `bundle copies (blob AND legacy JSON) must be released from the cache after extraction, got: ${bundleCopies.join(", ")}`,
  ).toHaveLength(0);

  expect(
    keys.some((url) => url.includes(BUNDLE_DOWNLOADED_MARKER)),
    "the downloaded marker must be stored",
  ).toBe(true);

  // The v3 index blob is the offline phrase-index source: after the
  // cache wipe above, only the precache pass could have put it there
  // (a warm HTTP-cache may serve it without a network event, so the
  // presence is asserted in the Cache API, not in the request log).
  expect(
    keys.some((url) => url.endsWith("phrases/phrase_index.v3.rkyv")),
    "the v3 phrase index blob must be in the cache as the offline source",
  ).toBe(true);

  const chunks = keys.filter((url) => url.includes("phrases/data/p"));
  expect(
    chunks.length,
    "extracted phrase chunks must be the cached offline source",
  ).toBeGreaterThan(0);
  expect(
    chunks.every((url) => url.includes("?v=")),
    `chunks must be keyed with their content hash (?v=), got: ${chunks[0] ?? "?"}`,
  ).toBe(true);
});

When(
  "пользователь заново открывает профиль",
  async ({ page, cdnRequestLog }) => {
    // Full reload: the restored state must survive a fresh boot, and the
    // request log since the reload must stay clean of CDN traffic.
    cdnRequestLog.length = 0;
    await openProfile(page);
  },
);

Then(
  "пакет восстановлен без повторного скачивания",
  async ({ page, cdnRequestLog }) => {
    await expect(page.getByText("Пакет скачан")).toBeVisible({
      timeout: 60_000,
    });

    const cdnRequests = cdnRequestLog.filter((url) => url.includes(CDN_ORIGIN));
    const refetches = cdnRequests.filter(
      (url) =>
        url.includes("phrases/") && !url.includes(BUNDLE_DOWNLOADED_MARKER),
    );
    expect(
      refetches,
      `the downloaded state must restore from the marker, no phrase refetches allowed, got: ${refetches.join(", ")}`,
    ).toHaveLength(0);
  },
);
