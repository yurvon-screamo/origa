/**
 * Unit tests for the Bunny Scriptable DNS script (handleQuery.js, ID 91629).
 *
 * The script runs inside Bunny's sandbox: it is ESM ("export default
 * function handleQuery") and relies on two injected globals (CnameRecord,
 * ARecord). These tests reproduce that context with node:vm — the file
 * under test is read byte-for-byte from disk (exactly what
 * `bunny dns scripts deploy` uploads), with only the `export default`
 * prefix stripped, because vm Script mode has no ESM.
 *
 * Run: node --test infra/bunny-dns/handleQuery.test.js
 * (by file — a directory run would also try to execute the ESM script
 * under test as a CommonJS test file and fail).
 *
 * Behavior contracts pinned here (variant A, ADR-072; task
 * 20261104060617-ag7v6qr):
 *  - the dormant pre-staged umami geo-split branch: RF -> the RF VM A
 *    record via the common isRF block, world -> the Railway edge CNAME;
 *  - s3 / content branches unchanged;
 *  - cert-bearing names (origa, app.origa) are NOT guarded anymore —
 *    grey answer (undefined), the fail-loud signal for accidental
 *    re-attachment without a branch.
 */
const { test } = require("node:test");
const assert = require("node:assert/strict");
const { readFileSync } = require("node:fs");
const path = require("node:path");
const { runInNewContext } = require("node:vm");

const SCRIPT_PATH = path.join(__dirname, "handleQuery.js");
const README_PATH = path.join(__dirname, "README.md");

const VM_IP = "193.233.217.243";
const UMAMI_TARGET = "07wyrdxw.up.railway.app";
const S3_TARGET = "d3gbi3wo8j4c2w.cloudfront.net";
const CONTENT_TARGET = "origa.t3.tigrisbucket.io";

class CnameRecord {
  constructor(target, ttl) {
    this.type = "CNAME";
    this.target = target;
    this.ttl = ttl;
  }
}

class ARecord {
  constructor(ip, ttl) {
    this.type = "A";
    this.ip = ip;
    this.ttl = ttl;
  }
}

function loadScript() {
  const source = readFileSync(SCRIPT_PATH, "utf8");
  const body = source.replace(/^export default\s+/m, "");
  const sandbox = { CnameRecord, ARecord };
  runInNewContext(body, sandbox);
  assert.equal(typeof sandbox.handleQuery, "function", "handleQuery must be defined by the script");
  return sandbox.handleQuery;
}

/** @param {string} hostname @param {string | undefined} country */
function query(hostname, country) {
  const q = { request: { hostname } };
  if (country !== undefined) q.request.geoLocation = { country };
  return q;
}

// --- Main scenario: umami pre-staged geo-split (dormant until re-attach) ---

test("umami world branch returns the Railway edge CNAME (pre-staged geo-split)", () => {
  const handleQuery = loadScript();
  const rec = handleQuery(query("umami.origa.uwuwu.net", "US"));
  assert.equal(rec.type, "CNAME");
  assert.equal(rec.target, UMAMI_TARGET);
  assert.equal(rec.ttl, 300);
});

test("umami RF branch hits the common isRF block (A record of the RF VM)", () => {
  const handleQuery = loadScript();
  const rec = handleQuery(query("umami.origa.uwuwu.net", "RU"));
  assert.equal(rec.type, "A");
  assert.equal(rec.ip, VM_IP);
  assert.equal(rec.ttl, 60);
});

test("umami without geoLocation (Bunny may omit it) falls through to the world branch", () => {
  const handleQuery = loadScript();
  const rec = handleQuery(query("umami.origa.uwuwu.net"));
  assert.equal(rec.target, UMAMI_TARGET);
});

// --- s3.origa: geo-split (unchanged by the task, guard for regressions) ---

test("s3 RF branch -> RF VM A record", () => {
  const handleQuery = loadScript();
  const rec = handleQuery(query("s3.origa.uwuwu.net", "RU"));
  assert.equal(rec.type, "A");
  assert.equal(rec.ip, VM_IP);
});

test("s3 world branch -> CloudFront CNAME", () => {
  const handleQuery = loadScript();
  const rec = handleQuery(query("s3.origa.uwuwu.net", "DE"));
  assert.equal(rec.type, "CNAME");
  assert.equal(rec.target, S3_TARGET);
  assert.equal(rec.ttl, 300);
});

// --- content.origa: diagnostic name, no RF branch (sits above isRF) ---

test("content -> Tigris CNAME for RF clients too (branch sits above isRF)", () => {
  const handleQuery = loadScript();
  for (const country of ["RU", "US"]) {
    const rec = handleQuery(query("content.origa.uwuwu.net", country));
    assert.equal(rec.type, "CNAME", `country=${country}`);
    assert.equal(rec.target, CONTENT_TARGET, `country=${country}`);
    assert.equal(rec.ttl, 300, `country=${country}`);
  }
});

// --- fail-loud guard: not-attached names must never resolve through the script ---

test("cert-bearing names (origa, app.origa) are no longer guarded — grey answer", () => {
  const handleQuery = loadScript();
  for (const hostname of ["origa.uwuwu.net", "app.origa.uwuwu.net"]) {
    for (const country of ["RU", "US", undefined]) {
      const rec = handleQuery(query(hostname, country));
      assert.equal(rec, undefined, `${hostname} country=${country} must get no record`);
    }
  }
});

test("unknown hostname -> undefined (never the RF IP)", () => {
  const handleQuery = loadScript();
  assert.equal(handleQuery(query("pass.uwuwu.net", "RU")), undefined);
  assert.equal(handleQuery(query("evil.origa.uwuwu.net.evil.com", "RU")), undefined);
});

test("empty or missing hostname -> undefined", () => {
  const handleQuery = loadScript();
  assert.equal(handleQuery(query("", "RU")), undefined);
  assert.equal(handleQuery({ request: { geoLocation: { country: "RU" } } }), undefined);
  assert.equal(handleQuery({ request: {} }), undefined);
});

test("FQDN trailing dot is stripped before matching", () => {
  const handleQuery = loadScript();
  const rec = handleQuery(query("umami.origa.uwuwu.net.", "US"));
  assert.equal(rec.target, UMAMI_TARGET);
  assert.equal(handleQuery(query("origa.uwuwu.net.", "RU")), undefined);
});

// --- docs-contract pin: README and the script must name the same umami target ---

test("README's umami plain-CNAME target matches the script's dormant branch target", () => {
  const readme = readFileSync(README_PATH, "utf8");
  const m = readme.match(/`umami\.origa` → ([a-z0-9]+\.up\.railway\.app)/);
  assert.ok(m, "README must state umami's plain CNAME target");
  assert.equal(m[1], UMAMI_TARGET);
});

test("README attached-names list (Script records) matches the script's guarded names", () => {
  const readme = readFileSync(README_PATH, "utf8");
  const attached = readme.match(/attached \(via Script records[^\n]*\) to \*\*two\*\* names/);
  assert.ok(attached, "README must pin the two attached names (s3, content)");
  const handleQuery = loadScript();
  // s3 and content get answers, everything else in the cert-bearing trio stays grey.
  assert.notEqual(handleQuery(query("s3.origa.uwuwu.net", "US")), undefined);
  assert.notEqual(handleQuery(query("content.origa.uwuwu.net", "US")), undefined);
  assert.equal(handleQuery(query("umami.origa.uwuwu.net", "US")).target, UMAMI_TARGET);
});
