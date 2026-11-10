import assert from "node:assert/strict";
import { test } from "node:test";

import { APP_SERVER_TIMEOUT, parseTimeout } from "./config.ts";

test("parseTimeout accepts positive milliseconds as number and string", () => {
  assert.equal(parseTimeout("E2E_APP_SERVER_TIMEOUT", 1500), 1500);
  assert.equal(parseTimeout("E2E_APP_SERVER_TIMEOUT", "3600000"), 3600000);
});

test("parseTimeout rejects zero, negative, non-finite and garbage", () => {
  const rejected: Array<string | number> = [
    0,
    -1,
    "0",
    "-5",
    Number.NaN,
    Infinity,
    "abc",
    "",
  ];
  for (const raw of rejected) {
    assert.throws(
      () => parseTimeout("E2E_APP_SERVER_TIMEOUT", raw),
      /должно быть положительным числом миллисекунд/,
      `raw: ${String(raw)}`,
    );
  }
});

test("APP_SERVER_TIMEOUT resolves the cold-build default without an override", () => {
  // The unit run has no E2E_APP_SERVER_TIMEOUT in its environment, so the
  // constant must carry the hour ceiling (the cold wasm-build budget).
  assert.equal(APP_SERVER_TIMEOUT, 3_600_000);
});
