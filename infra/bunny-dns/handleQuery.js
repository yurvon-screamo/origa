/**
 * Geo-steering for Origa (.net zone):
 *  - RF clients    -> Aeza VPS (Caddy proxies Tigris / Railway)
 *  - everyone else -> direct origins: Railway edge (API/landing),
 *                     Tigris bucket custom domain (CDN content)
 */
export default function handleQuery(query) {
  var h = query.request.hostname || "";
  var world;
  if (h.indexOf("app.origa.uwuwu.net") === 0) {
    world = new CnameRecord("9v15a3ov.up.railway.app", 300);   // TrailBase API
  } else if (h.indexOf("origa.uwuwu.net") === 0) {
    world = new CnameRecord("vl080mt6.up.railway.app", 300);   // landing
  } else {
    world = new CnameRecord("origa.t3.tigrisbucket.io", 300);  // s3: Tigris custom domain
  }
  var geo = query.request.geoLocation;
  if (geo && geo.country === "RU") {
    return new ARecord("85.192.63.249", 60);
  }
  return world;
}
