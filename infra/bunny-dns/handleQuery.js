/**
 * Geo-steering for Origa (.net zone) — Bunny DNS script 91629 "origa-s3-geo".
 *
 * Attached via Script records (zone 873317, TTL 60): s3.origa, content.origa.
 * Nothing else must reach this script: it answers ONLY the guarded names
 * below and returns undefined for any other hostname — a newly attached name
 * without a branch here must fail loudly (grey answer), not silently resolve
 * to the RF IP.
 *
 * ⚠ Do NOT attach cert-bearing names (origa, app.origa, umami.origa) to this
 * script while the Bunny NODATA bug stands: Scriptable DNS answers NODATA
 * through Google/8.8.8.8 resolvers (verified 2026-11-08: s3.origa via
 * 8.8.8.8 returns an empty answer 3/3 probes, plain records resolve fine),
 * and LE validators / Railway resolvers sit on Google DNS — script-attached
 * names fail domain validation, so cert issuance and renewal break. This is
 * the diagnosed root cause of the "internal error" issuance outage
 * (task 202611060809, ADR-072: resolved 2026-11-06 by moving those names to
 * plain CNAME records — the standing rule is "no Scriptable DNS on
 * cert-bearing names" until Bunny fixes the Google-resolver compatibility).
 *
 * Current topology (ADR-061; umami routing per variant A, task 202611060809):
 *  - s3.origa      -> RF: A 193.233.217.243 (RF VM, nginx SNI-demux ->
 *                     CloudFront/Tigris), world: CNAME CloudFront
 *  - content.origa -> CNAME Tigris custom domain for everyone (diagnostic
 *                     name, no RF branch)
 *  - origa / app.origa / umami.origa -> plain CNAME records in the zone,
 *                     straight to Railway for EVERYONE (RF included — TSPU
 *                     risk accepted for these three names, variant A,
 *                     ADR-072). The RF VM keeps nginx branches + certs for
 *                     them as dormant fallback (umami cert renewed
 *                     2026-11-08).
 *  - umami.origa   -> pre-staged geo-split branch below isRF (DORMANT: the
 *                     name is not attached to this script, so the branch
 *                     never sees queries; it reactivates only after the
 *                     Bunny NODATA bug is fixed and the name is re-attached)
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
  // origa / app.origa are deliberately NOT guarded: they are plain CNAME
  // records now; if one is ever re-attached without adding a branch, the
  // grey answer here is the fail-loud signal.
  if (
    h !== "s3.origa.uwuwu.net" &&
    h !== "content.origa.uwuwu.net" &&
    h !== "umami.origa.uwuwu.net"
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

  // umami.origa — pre-staged geo-split, DORMANT: the name is a plain zone
  // CNAME -> 07wyrdxw.up.railway.app since 2026-11-06 (variant A, ADR-072,
  // task 202611060809) and is NOT attached to this script, so this branch
  // never sees queries today. It exists so the post-fix return (Bunny
  // NODATA bug fixed, name re-attached) stays a single record operation:
  // RF clients hit the isRF branch above, world clients get the Railway
  // edge CNAME. Do NOT re-attach while the NODATA bug stands — see the
  // header warning (the umami cert renewal would break again). ADR-054 §7.
  if (h === "umami.origa.uwuwu.net") {
    return new CnameRecord("07wyrdxw.up.railway.app", 300);
  }

  if (h === "s3.origa.uwuwu.net") {
    return new CnameRecord("d3gbi3wo8j4c2w.cloudfront.net", 300);
  }
  // unreachable while the guard list above matches the branches; kept as a
  // fail-loud terminal: a name added to the guard without a branch must get
  // a grey answer, never another name's record.
  return undefined;
}
