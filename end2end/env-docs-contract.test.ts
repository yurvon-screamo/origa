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
  /^\s*(TRAILBASE_URL|ORIGA_CDN_BASE_URL|TEST_USER_EMAIL|TEST_USER_PASSWORD)=/m;

/**
 * The content of every fenced block whose opening fence carries `lang`.
 *
 * A state machine, not a regex pair: `^```\n([\s\S]*?)^```$` mis-pairs
 * the closing fence of a preceding tagged block as the opening fence of
 * an untagged one (and vice versa), so the real `.env` block of README.md
 * never landed in a capture and its pin could not fail. Openings switch
 * the machine on (```` ``` ```` optionally with an info string), bare
 * fences switch it off.
 */
const fencedBlocks = (markdown: string, lang: string): string[] => {
  const blocks: string[] = [];
  let inBlock = false;
  let openLang = "";
  let current: string[] = [];
  for (const line of markdown.split("\n")) {
    const fence = line.match(/^```(\w*)\s*$/);
    if (!fence) {
      if (inBlock) current.push(line);
      continue;
    }
    if (inBlock) {
      if (openLang === lang) blocks.push(current.join("\n"));
      inBlock = false;
    } else {
      inBlock = true;
      openLang = fence[1];
      current = [];
    }
  }
  return blocks;
};

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
