use std::sync::OnceLock;

pub struct Urls {
    pub base: &'static str,
    pub dictionary: &'static str,
    pub ndlocr_base: &'static str,
}

static URLS: OnceLock<Urls> = OnceLock::new();

pub fn urls() -> &'static Urls {
    URLS.get_or_init(|| {
        let base = env!("PUBLIC_BASE_URL");
        Urls {
            base,
            dictionary: "/public/dictionaries/unidic/cache/dictionary-data",
            ndlocr_base: "/public/ndlocr",
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

pub fn ndlocr_base_url() -> String {
    public_url(urls().ndlocr_base)
}
