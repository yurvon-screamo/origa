use std::env;

use chrono::Utc;
use hmac::{Hmac, Mac};
use sha2::{Digest, Sha256};

type HmacSha256 = Hmac<Sha256>;

const PRESIGNED_TTL_SECS: u32 = 604800;

struct CdnConfig {
    base_url: String,
    access_key: String,
    secret_key: String,
    region: String,
}

fn load_cdn_config() -> Result<CdnConfig, String> {
    let base_url = env::var("CDN_BASE_URL").map_err(|_| "CDN_BASE_URL not set")?;
    let access_key = env::var("CDN_ACCESS_KEY").map_err(|_| "CDN_ACCESS_KEY not set")?;
    let secret_key = env::var("CDN_SECRET_KEY").map_err(|_| "CDN_SECRET_KEY not set")?;
    let region = env::var("CDN_REGION").map_err(|_| "CDN_REGION not set")?;
    Ok(CdnConfig {
        base_url,
        access_key,
        secret_key,
        region,
    })
}

fn to_hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

fn hmac_sha256(key: &[u8], data: &[u8]) -> Vec<u8> {
    let mut mac = HmacSha256::new_from_slice(key)
        .expect("invariant: HMAC-SHA256 accepts any key length per RFC 2104");
    mac.update(data);
    mac.finalize().into_bytes().to_vec()
}

fn split_base_url(base: &str) -> (&str, &str) {
    let without_scheme = base.strip_prefix("https://").unwrap_or(base);
    let without_scheme = without_scheme
        .strip_prefix("http://")
        .unwrap_or(without_scheme);
    let (host, path) = without_scheme
        .split_once('/')
        .unwrap_or((without_scheme, ""));
    (host, path)
}

fn derive_signing_key(date: &str, region: &str, secret: &str) -> Vec<u8> {
    let k_date = hmac_sha256(format!("AWS4{}", secret).as_bytes(), date.as_bytes());
    let k_region = hmac_sha256(&k_date, region.as_bytes());
    let k_service = hmac_sha256(&k_region, b"s3");
    hmac_sha256(&k_service, b"aws4_request")
}

fn build_presigned_url(
    method: &str,
    clean_path: &str,
    user_query: Option<&str>,
    config: &CdnConfig,
) -> Result<String, String> {
    let date = Utc::now().format("%Y%m%d").to_string();
    let base = config.base_url.trim_end_matches('/');
    let (host, base_path) = split_base_url(base);

    let canonical_path = format!("/{}{}", base_path, clean_path);
    let timestamp = format!("{}T000000Z", date);
    let credential = format!(
        "{}/{}/{}/s3/aws4_request",
        config.access_key, date, config.region
    );

    let canonical_query = build_canonical_query(&credential, &timestamp, user_query);
    let canonical_request = format!(
        "{}\n{}\n{}\nhost:{}\n\nhost\nUNSIGNED-PAYLOAD",
        method, canonical_path, canonical_query, host,
    );

    let request_hash = to_hex(&Sha256::digest(canonical_request.as_bytes()));
    let string_to_sign = format!(
        "AWS4-HMAC-SHA256\n{}\n{}/{}/s3/aws4_request\n{}",
        timestamp, date, config.region, request_hash,
    );

    let signing_key = derive_signing_key(&date, &config.region, &config.secret_key);
    let signature = to_hex(&hmac_sha256(&signing_key, string_to_sign.as_bytes()));

    Ok(format!(
        "{}{}?{}&X-Amz-Signature={}",
        base, clean_path, canonical_query, signature,
    ))
}

fn build_canonical_query(credential: &str, timestamp: &str, user_query: Option<&str>) -> String {
    let mut params: Vec<(&str, String)> = vec![
        ("X-Amz-Algorithm", "AWS4-HMAC-SHA256".to_string()),
        (
            "X-Amz-Credential",
            urlencoding::encode(credential).to_string(),
        ),
        ("X-Amz-Date", timestamp.to_string()),
        ("X-Amz-Expires", PRESIGNED_TTL_SECS.to_string()),
        ("X-Amz-SignedHeaders", "host".to_string()),
    ];

    if let Some(query) = user_query {
        for pair in query.split('&') {
            if pair.is_empty() {
                continue;
            }
            if let Some((key, value)) = pair.split_once('=') {
                params.push((key, urlencoding::encode(value).to_string()));
            } else {
                params.push((pair, String::new()));
            }
        }
    }

    params.sort_by(|a, b| a.0.cmp(b.0));
    params
        .iter()
        .map(|(k, v)| {
            if v.is_empty() {
                (*k).to_string()
            } else {
                format!("{k}={v}")
            }
        })
        .collect::<Vec<_>>()
        .join("&")
}

/// Generate a GET presigned URL for CDN path.
pub fn cdn_url(path: &str) -> Result<String, String> {
    let config = load_cdn_config()?;
    let (clean_path, user_query) = match path.split_once('?') {
        Some((p, q)) => (p, Some(q)),
        None => (path, None),
    };
    build_presigned_url("GET", clean_path, user_query, &config)
}

fn put_header(
    name: &str,
    value: &str,
) -> Result<(reqwest::header::HeaderName, reqwest::header::HeaderValue), String> {
    let n = name
        .parse()
        .map_err(|e| format!("invalid header '{name}': {e}"))?;
    let v = value
        .parse()
        .map_err(|e| format!("invalid value for '{name}': {e}"))?;
    Ok((n, v))
}

pub fn sign_put_request(
    path: &str,
    content_length: usize,
) -> Result<(String, reqwest::header::HeaderMap), String> {
    let config = load_cdn_config()?;
    let base = config.base_url.trim_end_matches('/');
    let (host, base_path) = split_base_url(base);

    let clean_path = match path.split_once('?') {
        Some((p, _)) => p,
        None => path,
    };
    let canonical_path = format!("/{}{}", base_path, clean_path);

    let now = Utc::now();
    let date = now.format("%Y%m%d").to_string();
    let timestamp = now.format("%Y%m%dT%H%M%SZ").to_string();

    let canonical_request = format!(
        "PUT\n{}\n\nhost:{}\nx-amz-content-sha256:UNSIGNED-PAYLOAD\nx-amz-date:{}\n\nhost;x-amz-content-sha256;x-amz-date\nUNSIGNED-PAYLOAD",
        canonical_path, host, timestamp,
    );

    let request_hash = to_hex(&Sha256::digest(canonical_request.as_bytes()));
    let string_to_sign = format!(
        "AWS4-HMAC-SHA256\n{}\n{}/{}/s3/aws4_request\n{}",
        timestamp, date, config.region, request_hash,
    );

    let signing_key = derive_signing_key(&date, &config.region, &config.secret_key);
    let signature = to_hex(&hmac_sha256(&signing_key, string_to_sign.as_bytes()));

    let authorization = format!(
        "AWS4-HMAC-SHA256 Credential={}/{}/{}/s3/aws4_request, SignedHeaders=host;x-amz-content-sha256;x-amz-date, Signature={}",
        config.access_key, date, config.region, signature,
    );

    let url = format!("{base}{clean_path}");

    let cl = content_length.to_string();
    let mut headers = reqwest::header::HeaderMap::new();
    for (name, value) in [
        ("host", host),
        ("x-amz-content-sha256", "UNSIGNED-PAYLOAD"),
        ("x-amz-date", &timestamp),
        ("authorization", &authorization),
        ("content-length", &cl),
    ] {
        let (n, v) = put_header(name, value)?;
        headers.insert(n, v);
    }

    Ok((url, headers))
}
