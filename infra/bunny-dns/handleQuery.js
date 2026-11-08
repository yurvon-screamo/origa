/**
 * Geo-steering + RF failover for Origa (.net zone) — Bunny DNS script 91629
 * "origa-s3-geo".
 *
 * Live topology (ADR-072, 2026-11-06): the script is attached ONLY to
 * s3.origa and content.origa (SCRIPT records, TTL 60). Everything else in
 * the zone stays out of the script:
 *  - origa / app.origa / umami.origa are plain CNAMEs to the Railway edge —
 *    script-attached names break LE-issuance paths that resolve through
 *    Google resolvers (ADR-072), so never re-attach them;
 *  - pass is a plain A record and was never in the blast radius.
 * A name newly attached in Bunny without a branch below grey-answers for
 * everyone (fail-loud by design), never silently resolves to another
 * name's record.
 *
 * s3.origa (ADR-061 RF delivery branch):
 *  - RF resolvers -> A 193.233.217.243 (RF VM, nginx SNI-demux -> Tigris),
 *    but ONLY while Bunny's built-in background monitoring sees the VM
 *    online (Monitoring.getStatus, ICMP-based);
 *  - VM offline -> RF clients get the same CloudFront CNAME the world gets:
 *    degraded (evening TSPU throttling slows it) but working, instead of
 *    silently dead. Any non-true status is treated as offline (fail-closed).
 *  - world -> CNAME d3gbi3wo8j4c2w.cloudfront.net (CloudFront in front of
 *    the public Tigris bucket). The direct Tigris custom domain
 *    (origa.t3.tigrisbucket.io) stays parked: its edge still serves TLS
 *    alert 80 on that SNI — flip only after a real-client 200 from a Tigris
 *    edge IP (verified dead 2026-09-20).
 *
 * Monitoring semantics (Bunny docs): ICMP ping only, 700 ms x 3 retries,
 * one reply = online; actively queried IPs re-checked ~every 10 s; the
 * status is per DNS server ("reachable from the PoP answering the query").
 * ICMP-only means an nginx crash on a live VM is NOT seen here — that class
 * is covered by the uptime poller's HTTP probe (alert), not by DNS.
 *
 * TTLs are asymmetric on purpose: the RF A answer carries TTL 60 (fast
 * failover), the CloudFront CNAME carries TTL 300 — so recovery back to A
 * converges up to ~5 minutes after Bunny sees the VM alive again. See
 * infra/bunny-dns/README.md for the drill procedure and measured numbers.
 *
 * content.origa: CNAME to the direct Tigris custom domain for everyone
 * (diagnostic name, no RF branch).
 */

var RF_VM_IP = "193.233.217.243";
var S3_WORLD_CNAME = "d3gbi3wo8j4c2w.cloudfront.net";
var CONTENT_CNAME = "origa.t3.tigrisbucket.io";

/**
 * True only when Bunny's background monitoring reports the RF VM online.
 * The typeof-guard keeps a future platform-side removal of the Monitoring
 * global from throwing (a throw grey-answers the query — for the world
 * branch too); degrading RF users to the working world path is the safer
 * default, that is the whole point of this gate.
 */
function isRfVmOnline() {
  if (typeof Monitoring === "undefined") {
    return false;
  }
  return Monitoring.getStatus(RF_VM_IP).isOnline === true;
}

export default function handleQuery(query) {
  var hostname = (query.request.hostname || "").replace(/\.$/, ""); // strip FQDN trailing dot

  // Guard: answer only the names attached to this script (see header).
  if (
    hostname !== "s3.origa.uwuwu.net" &&
    hostname !== "content.origa.uwuwu.net"
  ) {
    return undefined; // not ours — do NOT fall through to any record
  }

  if (hostname === "content.origa.uwuwu.net") {
    return new CnameRecord(CONTENT_CNAME, 300);
  }

  // RF geo branch: the RF VM while Bunny sees it alive...
  var geo = query.request.geoLocation;
  if (geo && geo.country === "RU" && isRfVmOnline()) {
    return new ARecord(RF_VM_IP, 60);
  }

  // ...otherwise the world path: world clients always, RF clients while
  // the VM is dead (failover).
  return new CnameRecord(S3_WORLD_CNAME, 300);
}
