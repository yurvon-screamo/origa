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
  var geo = query.request.geoLocation;
  var isRF = geo && geo.country === "RU";

  // s3.origa world branch: PARKED on Aeza until the Tigris custom-domain
  // certificate actually lands on their edge (2026-09-20: UI flipped back
  // to "Processing Certificate"; edge serves TLS alert 80). Flip to
  // CnameRecord("origa.t3.tigrisbucket.io") once https://s3.origa.uwuwu.net
  // answers from Tigris directly.
  if (h === "app.origa.uwuwu.net" && !isRF) {
    return new CnameRecord("9v15a3ov.up.railway.app", 300);
  }
  if (h === "origa.uwuwu.net" && !isRF) {
    return new CnameRecord("vl080mt6.up.railway.app", 300);
  }
  return new ARecord("85.192.63.249", 60);
}
