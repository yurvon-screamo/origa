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

fn sanitize_model_name(name: &str, allow_slash: bool) -> String {
    name.chars()
        .map(|c| {
            let allowed = c.is_alphanumeric() || c == '-' || c == '_' || (allow_slash && c == '/');
            if allowed { c } else { '_' }
        })
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
        info!(
            "Loading models: OCR={} Layout={}",
            self.config.ocr_model_name, self.config.layout_model_name
        );
        let window = web_sys::window().ok_or_else(|| OrigaError::OcrError {
            reason: "No window object available".to_string(),
        })?;

        let ocr_cache = self
            .get_cache(
                &window,
                "manga-ocr-model-",
                &self.config.ocr_model_name,
                false,
            )
            .await?;
        let layout_cache = self
            .get_cache(
                &window,
                "doclayout-yolo-model-",
                &self.config.layout_model_name,
                true,
            )
            .await?;

        let ocr_filenames = ModelConfig::ocr_file_names();
        let ocr_load_names = &ocr_filenames[..3];

        let (encoder, decoder, tokenizer) =
            if self.ensure_files_cached(&ocr_cache, ocr_filenames).await? {
                info!("OCR Model found in cache, loading...");
                let mut loaded = self
                    .load_files_from_cache(&ocr_cache, ocr_load_names)
                    .await?;
                (loaded.remove(0), loaded.remove(0), loaded.remove(0))
            } else {
                info!("OCR Model not found in cache, downloading...");
                let files: Vec<(&str, String)> = ocr_filenames
                    .iter()
                    .map(|&f| (f, self.config.ocr_model_file_url(f)))
                    .collect();
                let mut loaded = self.download_and_cache_model(&ocr_cache, &files).await?;
                (loaded.remove(0), loaded.remove(0), loaded.remove(0))
            };

        let layout_name = self.config.layout_filename.as_str();
        let layout_model = if self
            .ensure_files_cached(&layout_cache, &[layout_name])
            .await?
        {
            info!("Layout model found in cache, loading...");
            self.load_files_from_cache(&layout_cache, &[layout_name])
                .await?
                .into_iter()
                .next()
                .unwrap()
        } else {
            info!("Layout model not found in cache, downloading...");
            let files = [(layout_name, self.config.layout_model_file_url())];
            self.download_and_cache_model(&layout_cache, &files)
                .await?
                .into_iter()
                .next()
                .unwrap()
        };

        info!("All models loaded successfully");
        Ok(ModelFiles {
            encoder,
            decoder,
            tokenizer,
            layout_model,
        })
    }

    async fn get_cache(
        &self,
        window: &Window,
        prefix: &str,
        model_name: &str,
        allow_slash: bool,
    ) -> Result<Cache, OrigaError> {
        let cache_storage = window
            .caches()
            .map_err(|e| js_err("Failed to get cache storage", &e))?;

        let safe_model_name = sanitize_model_name(model_name, allow_slash);
        let cache_name = format!("{}{}", prefix, safe_model_name);

        let cache = JsFuture::from(cache_storage.open(&cache_name))
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
            .map_err(|e| js_err("Failed to check cache", &e))?
            .as_bool()
            .unwrap_or(false);

        Ok(has_response)
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
    ) -> Result<(), OrigaError> {
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

        Ok(())
    }
}
