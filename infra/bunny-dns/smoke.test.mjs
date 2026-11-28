/**
 * Branch-matrix + README doc-contract smoke test for handleQuery.js
 * (Bunny DNS script 91629).
 *
 * Run: node infra/bunny-dns/smoke.test.mjs
 * Plain node:test + node:assert — no dependencies. The Bunny runtime
 * globals (ARecord, CnameRecord, Monitoring) are stubbed on globalThis
 * before the script under test is imported; the script only touches them
 * inside handleQuery, so import order is safe.
 *
 * The doc-contract block compares the live answers of the script against
 * the README: two attached names, their targets and the guard policy.
 * Only stable invariants are pinned — short-lived geo branches are not.
 *
 * Lives outside CI on purpose: infra/** is deliberately not in ci.yml
 * path filters (docs/scripts-only PRs skip CI entirely) — adding a CI
 * component for one DNS script is not worth it. The Bunny-failover task
 * documents this decision.
 */

import { strict as assert } from "node:assert";
import { readFileSync } from "node:fs";
import { after, before, describe, it } from "node:test";

const RF_VM_IP = "193.233.217.243";
const S3_WORLD_CNAME = "d3gbi3wo8j4c2w.cloudfront.net";
const CONTENT_CNAME = "origa.t3.tigrisbucket.io";

class ARecord {
  constructor(ip, ttl) {
    this.ip = ip;
    this.ttl = ttl;
  }
}

class CnameRecord {
  constructor(hostname, ttl) {
    this.hostname = hostname;
    this.ttl = ttl;
  }
}

/** Query stub: hostname + resolver-side geo (Bunny geo follows the resolver). */
function query(hostname, country) {
  return {
    request: {
      hostname,
      geoLocation: country === undefined ? undefined : { country },
    },
  };
}

describe("handleQuery (Bunny script 91629)", () => {
  describe("with Bunny Monitoring reporting the VM online", () => {
    before(() => {
      globalThis.ARecord = ARecord;
      globalThis.CnameRecord = CnameRecord;
      globalThis.Monitoring = {
        getStatus: () => ({ isOnline: true, latency: 42 }),
      };
    });

    after(() => {
      delete globalThis.ARecord;
      delete globalThis.CnameRecord;
      delete globalThis.Monitoring;
    });

    it("answers RF resolvers with the RF VM A record", async () => {
      const handleQuery = await import("./handleQuery.js").then(
        (m) => m.default,
      );

      const answer = handleQuery(query("s3.origa.uwuwu.net", "RU"));

      assert.ok(answer instanceof ARecord);
      assert.equal(answer.ip, RF_VM_IP);
      assert.equal(answer.ttl, 60);
    });

    it("answers world resolvers with the CloudFront CNAME without consulting monitoring", async () => {
      const onlineStub = globalThis.Monitoring;
      globalThis.Monitoring = {
        // the world branch must never reach the monitoring gate — a call
        // here would throw and fail the test
        getStatus: () => {
          throw new Error("getStatus consulted for a world query");
        },
      };
      try {
        const handleQuery = await import("./handleQuery.js").then(
          (m) => m.default,
        );

        const answer = handleQuery(query("s3.origa.uwuwu.net", "US"));

        assert.ok(answer instanceof CnameRecord);
        assert.equal(answer.hostname, S3_WORLD_CNAME);
        assert.equal(answer.ttl, 300);
      } finally {
        globalThis.Monitoring = onlineStub; // keep sibling tests isolated
      }
    });

    it("answers RF resolvers with the world CNAME when the resolver geo is unknown", async () => {
      const handleQuery = await import("./handleQuery.js").then(
        (m) => m.default,
      );

      const answer = handleQuery(query("s3.origa.uwuwu.net", undefined));

      assert.ok(answer instanceof CnameRecord);
      assert.equal(answer.hostname, S3_WORLD_CNAME);
      assert.equal(answer.ttl, 300);
    });

    it("answers content.origa with the Tigris CNAME for everyone", async () => {
      const handleQuery = await import("./handleQuery.js").then(
        (m) => m.default,
      );

      for (const country of ["RU", "DE"]) {
        const answer = handleQuery(query("content.origa.uwuwu.net", country));
        assert.ok(answer instanceof CnameRecord);
        assert.equal(answer.hostname, CONTENT_CNAME);
        assert.equal(answer.ttl, 300);
      }
    });

    it("grey-answers hostnames outside the attached pair", async () => {
      const handleQuery = await import("./handleQuery.js").then(
        (m) => m.default,
      );

      assert.equal(handleQuery(query("origa.uwuwu.net", "RU")), undefined);
      assert.equal(handleQuery(query("app.origa.uwuwu.net", "RU")), undefined);
      assert.equal(handleQuery(query("evil.example.com", "RU")), undefined);
    });

    it("strips the FQDN trailing dot before matching", async () => {
      const handleQuery = await import("./handleQuery.js").then(
        (m) => m.default,
      );

      const answer = handleQuery(query("s3.origa.uwuwu.net.", "RU"));

      assert.ok(answer instanceof ARecord);
      assert.equal(answer.ip, RF_VM_IP);
      assert.equal(answer.ttl, 60);
    });
  });

  describe("with Bunny Monitoring reporting the VM offline", () => {
    before(() => {
      globalThis.ARecord = ARecord;
      globalThis.CnameRecord = CnameRecord;
      globalThis.Monitoring = {
        getStatus: () => ({ isOnline: false, latency: 0 }),
      };
    });

    after(() => {
      delete globalThis.ARecord;
      delete globalThis.CnameRecord;
      delete globalThis.Monitoring;
    });

    it("fails RF resolvers over to the CloudFront CNAME", async () => {
      const handleQuery = await import("./handleQuery.js").then(
        (m) => m.default,
      );

      const answer = handleQuery(query("s3.origa.uwuwu.net", "RU"));

      assert.ok(answer instanceof CnameRecord);
      assert.equal(answer.hostname, S3_WORLD_CNAME);
      assert.equal(answer.ttl, 300);
    });

    it("still answers world resolvers with the CloudFront CNAME", async () => {
      const handleQuery = await import("./handleQuery.js").then(
        (m) => m.default,
      );

      const answer = handleQuery(query("s3.origa.uwuwu.net", "DE"));

      assert.ok(answer instanceof CnameRecord);
      assert.equal(answer.hostname, S3_WORLD_CNAME);
      assert.equal(answer.ttl, 300);
    });
  });

  describe("with Bunny Monitoring returning an unknown status shape", () => {
    before(() => {
      globalThis.ARecord = ARecord;
      globalThis.CnameRecord = CnameRecord;
      globalThis.Monitoring = { getStatus: () => ({ latency: 42 }) };
    });

    after(() => {
      delete globalThis.ARecord;
      delete globalThis.CnameRecord;
      delete globalThis.Monitoring;
    });

    it("treats a non-true status as offline (fail-closed, no RF A answer)", async () => {
      const handleQuery = await import("./handleQuery.js").then(
        (m) => m.default,
      );

      const answer = handleQuery(query("s3.origa.uwuwu.net", "RU"));

      assert.ok(answer instanceof CnameRecord);
      assert.equal(answer.hostname, S3_WORLD_CNAME);
      assert.equal(answer.ttl, 300);
    });
  });

  describe("with non-true isOnline shapes (platform drift)", () => {
    // The gate trusts only strict true: any other value Bunny's platform
    // might hand back must degrade RF to the world path, never answer the A.
    const shapes = [
      ['string "true"', "true"],
      ["number 1", 1],
      ["null", null],
      ["plain object", {}],
      ["NaN", Number.NaN],
    ];

    for (const [label, value] of shapes) {
      it(`treats isOnline=${label} as offline (fail-closed -> world CNAME)`, async () => {
        globalThis.ARecord = ARecord;
        globalThis.CnameRecord = CnameRecord;
        globalThis.Monitoring = {
          getStatus: () => ({ isOnline: value, latency: 42 }),
        };
        try {
          const handleQuery = await import("./handleQuery.js").then(
            (m) => m.default,
          );

          const answer = handleQuery(query("s3.origa.uwuwu.net", "RU"));

          assert.ok(answer instanceof CnameRecord);
          assert.equal(answer.hostname, S3_WORLD_CNAME);
          assert.equal(answer.ttl, 300);
        } finally {
          delete globalThis.ARecord;
          delete globalThis.CnameRecord;
          delete globalThis.Monitoring;
        }
      });
    }
  });

  describe("with the Monitoring global missing (platform drift)", () => {
    before(() => {
      globalThis.ARecord = ARecord;
      globalThis.CnameRecord = CnameRecord;
      assert.equal(typeof globalThis.Monitoring, "undefined");
    });

    after(() => {
      delete globalThis.ARecord;
      delete globalThis.CnameRecord;
    });

    it("does not throw and degrades RF resolvers to the world CNAME", async () => {
      const handleQuery = await import("./handleQuery.js").then(
        (m) => m.default,
      );

      const answer = handleQuery(query("s3.origa.uwuwu.net", "RU"));

      assert.ok(answer instanceof CnameRecord);
      assert.equal(answer.hostname, S3_WORLD_CNAME);
      assert.equal(answer.ttl, 300);
    });

    it("keeps the world branch working", async () => {
      const handleQuery = await import("./handleQuery.js").then(
        (m) => m.default,
      );

      const answer = handleQuery(query("s3.origa.uwuwu.net", "GB"));

      assert.ok(answer instanceof CnameRecord);
      assert.equal(answer.hostname, S3_WORLD_CNAME);
      assert.equal(answer.ttl, 300);
    });
  });
});

describe("README ↔ handleQuery.js doc-contract pins", () => {
  const readme = readFileSync(new URL("./README.md", import.meta.url), "utf8");

  function attachedDocSections() {
    const start = readme.indexOf("attached (SCRIPT records");
    const end = readme.indexOf("Everything else in the zone", start);
    assert.ok(
      start !== -1 && end !== -1,
      "README must document the SCRIPT-attached pair",
    );

    const header = readme.slice(start, end);
    const matches = [...header.matchAll(/^- `([^`]+)`\s*→/gm)];

    return matches.map((match, index) => ({
      name: match[1],
      text: header.slice(
        match.index,
        matches[index + 1]?.index ?? header.length,
      ),
    }));
  }

  before(() => {
    globalThis.ARecord = ARecord;
    globalThis.CnameRecord = CnameRecord;
    globalThis.Monitoring = { getStatus: () => ({ isOnline: true }) };
  });

  after(() => {
    delete globalThis.ARecord;
    delete globalThis.CnameRecord;
    delete globalThis.Monitoring;
  });

  it("pins exactly two attached names in README to the script's hostname guard", async () => {
    const documented = attachedDocSections().map((section) => section.name);
    assert.deepEqual(documented, ["s3.origa", "content.origa"]);
    assert.match(readme, /to \*\*two\*\* names/);

    const handleQuery = await import("./handleQuery.js").then((m) => m.default);
    const candidates = [
      ...documented.map((name) => `${name}.uwuwu.net`),
      "origa.uwuwu.net",
      "app.origa.uwuwu.net",
      "umami.origa.uwuwu.net",
      "pass.uwuwu.net",
    ];
    const answered = candidates.filter(
      (hostname) => handleQuery(query(hostname, "US")) !== undefined,
    );

    assert.deepEqual(
      answered,
      documented.map((name) => `${name}.uwuwu.net`),
    );
  });

  it("pins README targets to the answers the script actually returns", async () => {
    const [s3, content] = attachedDocSections();
    const handleQuery = await import("./handleQuery.js").then((m) => m.default);

    const worldAnswer = handleQuery(query("s3.origa.uwuwu.net", "US"));
    const rfAnswer = handleQuery(query("s3.origa.uwuwu.net", "RU"));
    const contentAnswer = handleQuery(query("content.origa.uwuwu.net", "US"));

    assert.ok(rfAnswer instanceof ARecord);
    assert.ok(worldAnswer instanceof CnameRecord);
    assert.ok(contentAnswer instanceof CnameRecord);
    assert.ok(
      s3.text.includes(rfAnswer.ip),
      "README s3 section must carry the RF VM IP",
    );
    assert.ok(
      s3.text.includes(worldAnswer.hostname),
      "README s3 section must carry the world CNAME",
    );
    assert.ok(
      content.text.includes(contentAnswer.hostname),
      "README content section must carry the Tigris CNAME",
    );
  });

  it("pins the guard policy: README plain names grey-answer", async () => {
    const handleQuery = await import("./handleQuery.js").then((m) => m.default);

    for (const name of ["origa", "app.origa", "umami.origa", "pass"]) {
      assert.ok(readme.includes(`\`${name}\``), `README must list ${name}`);
      assert.equal(
        handleQuery(query(`${name}.uwuwu.net`, "RU")),
        undefined,
        `${name} must grey-answer`,
      );
    }
    assert.match(readme, /answers only the two attached names/);
    assert.match(readme, /returns `undefined` for anything else/);
  });
});
