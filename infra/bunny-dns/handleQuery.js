/**
 * Geo-steering for Origa (.net zone) — Bunny DNS script 91629 "origa-s3-geo".
 *
 * Attached via NS records (zone 873317, TTL 60): origa, app.origa,
 * s3.origa, content.origa. Nothing else must reach this script: it answers
 * ONLY the four names above and returns undefined for any other hostname —
 * a newly attached name without a branch here must fail loudly (grey
 * answer), not silently resolve to the RF IP.
 *
 * Current topology (ADR-061, cutover 2026-09-30):
 *  - RF clients    -> A 193.233.217.243 (RF VM, nginx SNI-demux -> Railway/Tigris)
 *  - world         -> direct: Railway edge (API/landing), CloudFront (CDN)
 *  - content.origa -> CNAME Tigris custom domain (diagnostic name, no RF branch)
 *
 * s3.origa world branch: CloudFront (free tier) in front of the public
 * Tigris bucket. The direct Tigris custom domain (origa.t3.tigrisbucket.io)
 * is parked: the Tigris edge still serves TLS alert 80 on that SNI —
 * verified 2026-09-20 by a real client path (curl via live CNAME fails with
 * 000) AND direct openssl probes on multiple edge IPs. Flip the s3 world
 * branch to CnameRecord("origa.t3.tigrisbucket.io") only after a real-client
 * test returns 200 from a Tigris edge IP.
 */
export default function handleQuery(query) {
  var h = (query.request.hostname || "").replace(/\.$/, ""); // strip FQDN trailing dot

  // Guard: answer only the names attached to this script (see header).
  if (
    h !== "origa.uwuwu.net" &&
    h !== "app.origa.uwuwu.net" &&
    h !== "s3.origa.uwuwu.net" &&
    h !== "content.origa.uwuwu.net"
  ) {
    return undefined; // not ours — do NOT fall through to the RF A-record
  }

  if (h === "content.origa.uwuwu.net") {
    return new CnameRecord("origa.t3.tigrisbucket.io", 300);
  }

  var geo = query.request.geoLocation;
  var isRF = geo && geo.country === "RU";
  if (isRF) {
    return new ARecord("193.233.217.243", 60);
  }

  if (h === "s3.origa.uwuwu.net") {
    return new CnameRecord("d3gbi3wo8j4c2w.cloudfront.net", 300);
  }
  if (h === "app.origa.uwuwu.net") {
    return new CnameRecord("9v15a3ov.up.railway.app", 300);
  }
  return new CnameRecord("vl080mt6.up.railway.app", 300); // origa.uwuwu.net
}
