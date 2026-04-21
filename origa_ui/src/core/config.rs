use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};

use hmac::{Hmac, Mac};
use sha2::{Digest, Sha256};

type HmacSha256 = Hmac<Sha256>;

const PRESIGNED_TTL_SECS: u32 = 604800;

pub struct Urls {
    pub base: &'static str,
    pub dictionary: &'static str,
    pub ndlocr_base: &'static str,
    #[cfg(target_arch = "wasm32")]
    pub whisper_base: &'static str,
}

static URLS: OnceLock<Urls> = OnceLock::new();

pub fn urls() -> &'static Urls {
    URLS.get_or_init(|| {
        let base = env!("PUBLIC_BASE_URL");
        Urls {
            base,
            dictionary: "/public/dictionaries/unidic/cache/dictionary-data",
            ndlocr_base: "/public/ndlocr",
            #[cfg(target_arch = "wasm32")]
            whisper_base: "https://huggingface.co/onnx-community/whisper-tiny/resolve/main",
        }
    })
}

pub fn public_url(path: &str) -> String {
    if !path.starts_with('/') {
        tracing::warn!(
            "public_url called with relative path '{}', expected absolute path starting with '/'",
            path
        );
    }

    let base = urls().base;
    if base.is_empty() {
        path.to_string()
    } else {
        format!("{}{}", base.trim_end_matches('/'), path)
    }
}

static SIGNING_CACHE: OnceLock<Mutex<HashMap<String, String>>> = OnceLock::new();

fn get_current_date() -> String {
    #[cfg(target_arch = "wasm32")]
    {
        let d = js_sys::Date::new_0();
        format!(
            "{:04}{:02}{:02}",
            d.get_full_year(),
            d.get_month() + 1,
            d.get_date()
        )
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        chrono::Utc::now().format("%Y%m%d").to_string()
    }
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
    let (host, path) = without_scheme
        .split_once('/')
        .unwrap_or((without_scheme, ""));
    (host, path)
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

fn build_canonical_request(canonical_path: &str, query: &str, host: &str) -> String {
    format!(
        "GET\n{}\n{}\nhost:{}\n\nhost\nUNSIGNED-PAYLOAD",
        canonical_path, query, host,
    )
}

fn derive_signing_key(date: &str, region: &str, secret: &str) -> Vec<u8> {
    let k_date = hmac_sha256(format!("AWS4{}", secret).as_bytes(), date.as_bytes());
    let k_region = hmac_sha256(&k_date, region.as_bytes());
    let k_service = hmac_sha256(&k_region, b"s3");
    hmac_sha256(&k_service, b"aws4_request")
}

struct SigningResult {
    signature: String,
    canonical_query: String,
}

fn compute_signing(clean_path: &str, user_query: Option<&str>) -> SigningResult {
    let date = get_current_date();
    let region = env!("CDN_REGION");
    let secret = env!("CDN_SECRET_KEY");
    let access_key = env!("CDN_ACCESS_KEY");
    let base = env!("CDN_BASE_URL").trim_end_matches('/');
    let (host, base_path) = split_base_url(base);

    let canonical_path = format!("/{}{}", base_path, clean_path);
    let timestamp = format!("{}T000000Z", date);
    let credential = format!("{}/{}/{}/s3/aws4_request", access_key, date, region);
    let canonical_query = build_canonical_query(&credential, &timestamp, user_query);
    let canonical_request = build_canonical_request(&canonical_path, &canonical_query, host);

    let request_hash = to_hex(&Sha256::digest(canonical_request.as_bytes()));
    let string_to_sign = format!(
        "AWS4-HMAC-SHA256\n{}\n{}/{}/s3/aws4_request\n{}",
        timestamp, date, region, request_hash,
    );

    let signing_key = derive_signing_key(&date, region, secret);
    let signature = hmac_sha256(&signing_key, string_to_sign.as_bytes());

    SigningResult {
        signature: to_hex(&signature),
        canonical_query,
    }
}

pub fn cdn_url(path: &str) -> String {
    let base = env!("CDN_BASE_URL").trim_end_matches('/');
    let date = get_current_date();

    let (clean_path, user_query) = match path.split_once('?') {
        Some((p, q)) => (p, Some(q)),
        None => (path, None),
    };

    let cache = SIGNING_CACHE.get_or_init(|| Mutex::new(HashMap::new()));
    let cache_key = format!("{date}:{path}");

    {
        let guard = cache.lock().expect("signing cache lock");
        if let Some(cached) = guard.get(&cache_key) {
            return cached.clone();
        }
    }

    let result = compute_signing(clean_path, user_query);

    let url = format!(
        "{}{}?{}&X-Amz-Signature={}",
        base, clean_path, result.canonical_query, result.signature,
    );

    {
        let mut guard = cache.lock().expect("signing cache lock");
        guard.insert(cache_key, url.clone());
    }

    url
}

pub fn ndlocr_base_url() -> String {
    public_url(urls().ndlocr_base)
}

#[cfg(target_arch = "wasm32")]
pub fn whisper_base_url() -> String {
    urls().whisper_base.to_string()
}
