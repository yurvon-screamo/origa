/**
 * Geo-steering for Origa (.net zone):
 *  - RF clients -> Aeza VPS (Caddy proxies Tigris / Railway)
 *  - world      -> direct: Railway edge (API/landing)
 *  - s3.origa   -> world temporarily also via Aeza until the Tigris
 *                  custom domain is validated (no broken-TLS window);
 *                  flip the world branch to CnameRecord("origa.t3.tigrisbucket.io")
 *                  when the console accepts the domain.
 */
export default function handleQuery(query) {
  var h = (query.request.hostname || "").replace(/\.$/, ""); // strip FQDN trailing dot
  // content.origa: diagnostic/validation name for the Tigris custom-domain
  // certificate (everyone gets the CNAME — no RF branch needed).
  if (h === "content.origa.uwuwu.net") {
    return new CnameRecord("origa.t3.tigrisbucket.io", 300);
  }
  var geo = query.request.geoLocation;
  var isRF = geo && geo.country === "RU";

  // s3.origa world branch: CloudFront (free tier) in front of the public
  // Tigris bucket. RF stays on the VPS; the world rides CF edges.
  if (h === "s3.origa.uwuwu.net" && !isRF) {
    return new CnameRecord("d3gbi3wo8j4c2w.cloudfront.net", 300);
  }
  // content.origa: diagnostic/validation name for the Tigris custom-domain
  // certificate (everyone gets the CNAME — no RF branch needed).
  if (h === "content.origa.uwuwu.net") {
    return new CnameRecord("origa.t3.tigrisbucket.io", 300);
  }
  var geo = query.request.geoLocation;
  var isRF = geo && geo.country === "RU";

  // s3.origa world branch: PARKED on Aeza. Tigris UI claims the certificate
  // is "completed" (valid till 2026-12-18, issuer YE1) but their edge still
  // serves TLS alert 80 on the SNI — verified 2026-09-20 by a real client
  // path (curl via live CNAME to 130.61.20.236 fails with 000) AND direct
  // openssl probes on multiple edge IPs. Flip back to
  // CnameRecord("origa.t3.tigrisbucket.io") only after a real-client test
  // returns 200 from a Tigris edge IP.
  if (h === "app.origa.uwuwu.net" && !isRF) {
    return new CnameRecord("9v15a3ov.up.railway.app", 300);
  }
  if (h === "origa.uwuwu.net" && !isRF) {
    return new CnameRecord("vl080mt6.up.railway.app", 300);
  }
  return new ARecord("85.192.63.249", 60);
}
