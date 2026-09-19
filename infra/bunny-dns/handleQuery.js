/**
 * Geo-steering for Origa (.net zone):
 *  - RF clients   -> Aeza VPS (Caddy proxies to Tigris / Railway)
 *  - everyone else -> CloudFront distributions (edge cache worldwide)
 * Health-gated: if the VPS is down, RF falls back to CloudFront as well.
 */
export default function handleQuery(query) {
  var aezaOnline = false;
  try { aezaOnline = Monitoring.getStatus("85.192.63.249").isOnline === true; } catch (e) { aezaOnline = false; }

  var cloudfront = "dsl8eedfp23ee.cloudfront.net"; // s3.origa (CDN content)
  var h = query.request.hostname || "";
  if (h.indexOf("origa.uwuwu.net") === 0) {
    cloudfront = "d3o6p7y31je36k.cloudfront.net"; // landing
  }

  var geo = query.request.geoLocation;
  if (geo && geo.country === "RU" && aezaOnline) {
    return new ARecord("85.192.63.249", 60);
  }
  return new CNameRecord(cloudfront, 300);
}
