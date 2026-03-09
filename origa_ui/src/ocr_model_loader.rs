use js_sys::Uint8Array;
use origa::domain::OrigaError;
use origa::ocr::{ModelConfig, ModelFiles};
use tracing::{debug, info};
use wasm_bindgen::{JsCast, JsValue};
use wasm_bindgen_futures::JsFuture;
use web_sys::{Cache, Request, RequestInit, RequestMode, Response, Window};

fn js_err(msg: impl AsRef<str>, e: &JsValue) -> OrigaError {
    OrigaError::OcrError {
        reason: format!("{}: {:?}", msg.as_ref(), e),
    }
}

fn sanitize_cache_name(name: &str) -> String {
    name.chars()
        .map(|c| if c.is_alphanumeric() || c == '-' || c == '_' { c } else { '_' })
        .collect()
}

pub struct ModelLoader {
    config: ModelConfig,
}

impl ModelLoader {
    pub fn new(config: ModelConfig) -> Self {
        Self { config }
    }

    pub async fn load_or_download_model(&self) -> Result<ModelFiles, OrigaError> {
        info!("Loading NDLOCR-Lite models from {}", self.config.ndlocr_base_url);

        let window = web_sys::window().ok_or_else(|| OrigaError::OcrError {
            reason: "No window object available".to_string(),
        })?;

        let cache_name = sanitize_cache_name(&self.config.ndlocr_cache_dir);
        let cache = self.get_cache(&window, &cache_name).await?;

        let filenames = ModelConfig::ndlocr_file_names();

        if self.ensure_files_cached(&cache, filenames).await? {
            info!("NDLOCR-Lite models found in cache, loading...");
            let loaded = self.load_files_from_cache(&cache, filenames).await?;
            self.build_model_files(loaded)
        } else {
            info!("NDLOCR-Lite models not found in cache, downloading...");
            let files: Vec<(&str, String)> = filenames
                .iter()
                .map(|&f| (f, self.config.model_url(f)))
                .collect();
            let loaded = self.download_and_cache_model(&cache, &files).await?;
            self.build_model_files(loaded)
        }
    }

    fn build_model_files(&self, mut loaded: Vec<Vec<u8>>) -> Result<ModelFiles, OrigaError> {
        if loaded.len() != 5 {
            return Err(OrigaError::OcrError {
                reason: format!("Expected 5 model files, got {}", loaded.len()),
            });
        }
        Ok(ModelFiles {
            deim: loaded.remove(0),
            parseq30: loaded.remove(0),
            parseq50: loaded.remove(0),
            parseq100: loaded.remove(0),
            vocab: loaded.remove(0),
        })
    }

    async fn get_cache(&self, window: &Window, cache_name: &str) -> Result<Cache, OrigaError> {
        let cache_storage = window
            .caches()
            .map_err(|e| js_err("Failed to get cache storage", &e))?;

        let cache = JsFuture::from(cache_storage.open(cache_name))
            .await
            .map_err(|e| js_err("Failed to open cache", &e))?;

        cache
            .dyn_into::<Cache>()
            .map_err(|e| js_err("Failed to cast to Cache", &e))
    }

    async fn ensure_files_cached(
        &self,
        cache: &Cache,
        filenames: &[&str],
    ) -> Result<bool, OrigaError> {
        for filename in filenames {
            if !self.is_file_cached(cache, filename).await? {
                return Ok(false);
            }
        }
        Ok(true)
    }

    async fn load_files_from_cache(
        &self,
        cache: &Cache,
        filenames: &[&str],
    ) -> Result<Vec<Vec<u8>>, OrigaError> {
        let mut result = Vec::with_capacity(filenames.len());
        for filename in filenames {
            let data = self.load_file_from_cache(cache, filename).await?;
            result.push(data);
        }
        Ok(result)
    }

    async fn download_and_cache_model(
        &self,
        cache: &Cache,
        files: &[(&str, String)],
    ) -> Result<Vec<Vec<u8>>, OrigaError> {
        for (filename, url) in files {
            self.download_and_cache_file(cache, filename, url).await?;
        }
        let filenames: Vec<&str> = files.iter().map(|(f, _)| *f).collect();
        self.load_files_from_cache(cache, &filenames).await
    }

    async fn is_file_cached(&self, cache: &Cache, filename: &str) -> Result<bool, OrigaError> {
        debug!("Checking if {} is cached", filename);
        let request =
            Request::new_with_str(filename).map_err(|e| js_err("Failed to create request", &e))?;

        let has_response = JsFuture::from(cache.match_with_request(&request))
            .await
            .map_err(|e| js_err("Failed to check cache", &e))?;

        Ok(!has_response.is_null() && !has_response.is_undefined())
    }

    async fn load_file_from_cache(
        &self,
        cache: &Cache,
        filename: &str,
    ) -> Result<Vec<u8>, OrigaError> {
        let request =
            Request::new_with_str(filename).map_err(|e| js_err("Failed to create request", &e))?;

        let response_js = JsFuture::from(cache.match_with_request(&request))
            .await
            .map_err(|e| js_err("Failed to get from cache", &e))?;

        if response_js.is_null() || response_js.is_undefined() {
            return Err(OrigaError::OcrError {
                reason: format!("File {} not in cache", filename),
            });
        }

        let response = response_js
            .dyn_into::<Response>()
            .map_err(|e| js_err("Failed to cast to Response", &e))?;

        let array_buffer = JsFuture::from(
            response
                .array_buffer()
                .map_err(|e| js_err("Failed to get array buffer", &e))?,
        )
        .await
        .map_err(|e| js_err("Failed to read array buffer", &e))?;

        let uint8_array = Uint8Array::new(&array_buffer);
        Ok(uint8_array.to_vec())
    }

    async fn download_and_cache_file(
        &self,
        cache: &Cache,
        filename: &str,
        url: &str,
    ) -> Result<Vec<u8>, OrigaError> {
        info!("Downloading {} from {}", filename, url);

        let opts = RequestInit::new();
        opts.set_method("GET");
        opts.set_mode(RequestMode::Cors);

        let request = Request::new_with_str_and_init(url, &opts)
            .map_err(|e| js_err("Failed to create request", &e))?;

        let window = web_sys::window().ok_or_else(|| OrigaError::OcrError {
            reason: "No window object available".to_string(),
        })?;

        let response = JsFuture::from(window.fetch_with_request(&request))
            .await
            .map_err(|e| js_err(format!("Failed to fetch {}", filename), &e))?
            .dyn_into::<Response>()
            .map_err(|e| js_err("Failed to cast to Response", &e))?;

        if !response.ok() {
            return Err(OrigaError::OcrError {
                reason: format!(
                    "Failed to download {}: HTTP {}",
                    filename,
                    response.status()
                ),
            });
        }

        let array_buffer = JsFuture::from(
            response
                .array_buffer()
                .map_err(|e| js_err("Failed to get array buffer", &e))?,
        )
        .await
        .map_err(|e| js_err("Failed to read response body", &e))?;

        let mut data = Uint8Array::new(&array_buffer).to_vec();
        debug!("Downloaded {} bytes for {}", data.len(), filename);

        let cache_request = Request::new_with_str(filename)
            .map_err(|e| js_err("Failed to create cache request", &e))?;

        let cache_response_init = web_sys::ResponseInit::new();
        let cache_response =
            Response::new_with_opt_u8_array_and_init(Some(&mut data[..]), &cache_response_init)
                .map_err(|e| js_err("Failed to create cache response", &e))?;

        debug!("Caching {}", filename);
        JsFuture::from(cache.put_with_request(&cache_request, &cache_response))
            .await
            .map_err(|e| js_err(format!("Failed to cache {}", filename), &e))?;

        Ok(data)
    }
}
