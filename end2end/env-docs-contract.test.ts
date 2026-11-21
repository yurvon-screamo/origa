import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { test } from "node:test";

/**
 * Pin for the documented .env contract (PR #955, task origa#416).
 *
 * The harness builds every service URL from the E2E_* port variables and
 * the Playwright webServer injects the compile-time TRAILBASE_URL /
 * ORIGA_CDN_BASE_URL into the app build — an `TRAILBASE_URL=` line in
 * `.env` is read by nothing. The docs used to carry that dead line as a
 * required setting, sending the next fresh-checkout reader to configure
 * a variable no code consumes. This pin guards the contract carriers
 * (.env.example and the .env blocks of AGENTS.md / README.md) against
 * the dead lines returning; prose mentions and bash examples that pass
 * the URL to `trunk serve` at build time stay outside the pin on
 * purpose.
 */
const deadEnvLines =
  /^(TRAILBASE_URL|ORIGA_CDN_BASE_URL|TEST_USER_EMAIL|TEST_USER_PASSWORD)=/m;

const fencedBlocks = (markdown: string, lang: string): string[] =>
  [...markdown.matchAll(new RegExp(`^\`\`\`${lang}\\n([\\s\\S]*?)^\`\`\`$`, "gm"))].map(
    (m) => m[1],
  );

test(".env.example documents only admin credentials", () => {
  const example = readFileSync(
    new URL("./.env.example", import.meta.url),
    "utf8",
  );

  assert.doesNotMatch(example, deadEnvLines);
});

test("AGENTS.md ini blocks (the .env contract) carry no dead service-URL lines", () => {
  const doc = readFileSync(new URL("./AGENTS.md", import.meta.url), "utf8");
  const blocks = fencedBlocks(doc, "ini");

  assert.ok(blocks.length > 0, "the required .env block went missing");
  for (const block of blocks) assert.doesNotMatch(block, deadEnvLines);
});

test("README.md .env block carries no dead service-URL lines", () => {
  const doc = readFileSync(new URL("./README.md", import.meta.url), "utf8");
  // The required .env block is the only untagged fenced block in the file.
  const blocks = fencedBlocks(doc, "");

  assert.ok(blocks.length > 0, "the required .env block went missing");
  for (const block of blocks) assert.doesNotMatch(block, deadEnvLines);
});
