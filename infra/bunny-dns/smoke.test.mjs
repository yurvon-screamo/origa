/**
 * Branch-matrix smoke test for handleQuery.js (Bunny DNS script 91629).
 *
 * Run: node infra/bunny-dns/smoke.test.mjs
 * Plain node:test + node:assert — no dependencies. The Bunny runtime
 * globals (ARecord, CnameRecord, Monitoring) are stubbed on globalThis
 * before the script under test is imported; the script only touches them
 * inside handleQuery, so import order is safe.
 *
 * Lives outside CI on purpose: infra/** is deliberately not in ci.yml
 * path filters (docs/scripts-only PRs skip CI entirely) — adding a CI
 * component for one DNS script is not worth it. The Bunny-failover task
 * documents this decision.
 */

import { strict as assert } from "node:assert";
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
      globalThis.Monitoring = { getStatus: () => ({ isOnline: true, latency: 42 }) };
    });

    after(() => {
      delete globalThis.ARecord;
      delete globalThis.CnameRecord;
      delete globalThis.Monitoring;
    });

    it("answers RF resolvers with the RF VM A record", async () => {
      const handleQuery = await import("./handleQuery.js").then((m) => m.default);

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
        const handleQuery = await import("./handleQuery.js").then((m) => m.default);

        const answer = handleQuery(query("s3.origa.uwuwu.net", "US"));

        assert.ok(answer instanceof CnameRecord);
        assert.equal(answer.hostname, S3_WORLD_CNAME);
        assert.equal(answer.ttl, 300);
      } finally {
        globalThis.Monitoring = onlineStub; // keep sibling tests isolated
      }
    });

    it("answers RF resolvers with the world CNAME when the resolver geo is unknown", async () => {
      const handleQuery = await import("./handleQuery.js").then((m) => m.default);

      const answer = handleQuery(query("s3.origa.uwuwu.net", undefined));

      assert.ok(answer instanceof CnameRecord);
      assert.equal(answer.hostname, S3_WORLD_CNAME);
      assert.equal(answer.ttl, 300);
    });

    it("answers content.origa with the Tigris CNAME for everyone", async () => {
      const handleQuery = await import("./handleQuery.js").then((m) => m.default);

      for (const country of ["RU", "DE"]) {
        const answer = handleQuery(query("content.origa.uwuwu.net", country));
        assert.ok(answer instanceof CnameRecord);
        assert.equal(answer.hostname, CONTENT_CNAME);
        assert.equal(answer.ttl, 300);
      }
    });

    it("grey-answers hostnames outside the attached pair", async () => {
      const handleQuery = await import("./handleQuery.js").then((m) => m.default);

      assert.equal(handleQuery(query("origa.uwuwu.net", "RU")), undefined);
      assert.equal(handleQuery(query("app.origa.uwuwu.net", "RU")), undefined);
      assert.equal(handleQuery(query("evil.example.com", "RU")), undefined);
    });

    it("strips the FQDN trailing dot before matching", async () => {
      const handleQuery = await import("./handleQuery.js").then((m) => m.default);

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
      globalThis.Monitoring = { getStatus: () => ({ isOnline: false, latency: 0 }) };
    });

    after(() => {
      delete globalThis.ARecord;
      delete globalThis.CnameRecord;
      delete globalThis.Monitoring;
    });

    it("fails RF resolvers over to the CloudFront CNAME", async () => {
      const handleQuery = await import("./handleQuery.js").then((m) => m.default);

      const answer = handleQuery(query("s3.origa.uwuwu.net", "RU"));

      assert.ok(answer instanceof CnameRecord);
      assert.equal(answer.hostname, S3_WORLD_CNAME);
      assert.equal(answer.ttl, 300);
    });

    it("still answers world resolvers with the CloudFront CNAME", async () => {
      const handleQuery = await import("./handleQuery.js").then((m) => m.default);

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
      const handleQuery = await import("./handleQuery.js").then((m) => m.default);

      const answer = handleQuery(query("s3.origa.uwuwu.net", "RU"));

      assert.ok(answer instanceof CnameRecord);
      assert.equal(answer.hostname, S3_WORLD_CNAME);
      assert.equal(answer.ttl, 300);
    });
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
      const handleQuery = await import("./handleQuery.js").then((m) => m.default);

      const answer = handleQuery(query("s3.origa.uwuwu.net", "RU"));

      assert.ok(answer instanceof CnameRecord);
      assert.equal(answer.hostname, S3_WORLD_CNAME);
      assert.equal(answer.ttl, 300);
    });

    it("keeps the world branch working", async () => {
      const handleQuery = await import("./handleQuery.js").then((m) => m.default);

      const answer = handleQuery(query("s3.origa.uwuwu.net", "GB"));

      assert.ok(answer instanceof CnameRecord);
      assert.equal(answer.hostname, S3_WORLD_CNAME);
      assert.equal(answer.ttl, 300);
    });
  });
});
