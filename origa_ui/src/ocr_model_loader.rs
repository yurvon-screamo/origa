use js_sys::Uint8Array;
use origa::domain::OrigaError;
use origa::ocr::{ModelConfig, ModelFiles};
use tracing::{debug, info};
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::JsFuture;
use web_sys::{Cache, Request, RequestInit, RequestMode, Response, Window};

pub struct ModelLoader {
    config: ModelConfig,
}

impl ModelLoader {
    pub fn new(config: ModelConfig) -> Self {
        Self { config }
    }

    pub async fn load_or_download_model(&self) -> Result<ModelFiles, OrigaError> {
        info!("Loading OCR model: {}", self.config.model_name);
        let window = web_sys::window().ok_or_else(|| OrigaError::OcrError {
            reason: "No window object available".to_string(),
        })?;

        let cache = self.get_or_create_cache(&window).await?;

        if self.is_model_cached(&cache).await? {
            info!(
                "Model {} found in cache, loading...",
                self.config.model_name
            );
            self.load_from_cache(&cache).await
        } else {
            info!(
                "Model {} not found in cache, downloading...",
                self.config.model_name
            );
            self.download_and_cache_model(&cache).await
        }
    }

    async fn get_or_create_cache(&self, window: &Window) -> Result<Cache, OrigaError> {
        let cache_storage = window.caches().map_err(|e| OrigaError::OcrError {
            reason: format!("Failed to get cache storage: {:?}", e),
        })?;

        let safe_model_name: String = self
            .config
            .model_name
            .chars()
            .map(|c| {
                if c.is_alphanumeric() || c == '-' {
                    c
                } else {
                    '_'
                }
            })
            .collect();
        let cache_name = format!("manga-ocr-model-{}", safe_model_name);

        let cache = JsFuture::from(cache_storage.open(&cache_name))
            .await
            .map_err(|e| OrigaError::OcrError {
                reason: format!("Failed to open cache: {:?}", e),
            })?;

        cache.dyn_into::<Cache>().map_err(|e| OrigaError::OcrError {
            reason: format!("Failed to cast to Cache: {:?}", e),
        })
    }

    async fn is_model_cached(&self, cache: &Cache) -> Result<bool, OrigaError> {
        for file in ModelConfig::file_names() {
            debug!("Checking if {} is cached", file);
            let request = Request::new_with_str(file).map_err(|e| OrigaError::OcrError {
                reason: format!("Failed to create request: {:?}", e),
            })?;

            let has_response = JsFuture::from(cache.match_with_request(&request))
                .await
                .map_err(|e| OrigaError::OcrError {
                    reason: format!("Failed to check cache: {:?}", e),
                })?
                .as_bool()
                .unwrap_or(false);

            if !has_response {
                return Ok(false);
            }
        }
        Ok(true)
    }

    async fn load_from_cache(&self, cache: &Cache) -> Result<ModelFiles, OrigaError> {
        debug!("Loading files from cache");
        let encoder = self
            .load_file_from_cache(cache, "encoder_model.onnx")
            .await?;
        let decoder = self
            .load_file_from_cache(cache, "decoder_model.onnx")
            .await?;
        let tokenizer = self.load_file_from_cache(cache, "tokenizer.json").await?;

        info!("Model files loaded successfully from cache");
        Ok(ModelFiles {
            encoder,
            decoder,
            tokenizer,
        })
    }

    async fn load_file_from_cache(
        &self,
        cache: &Cache,
        filename: &str,
    ) -> Result<Vec<u8>, OrigaError> {
        let request = Request::new_with_str(filename).map_err(|e| OrigaError::OcrError {
            reason: format!("Failed to create request: {:?}", e),
        })?;

        let response_js = JsFuture::from(cache.match_with_request(&request))
            .await
            .map_err(|e| OrigaError::OcrError {
                reason: format!("Failed to get from cache: {:?}", e),
            })?;

        if response_js.is_null() || response_js.is_undefined() {
            return Err(OrigaError::OcrError {
                reason: format!("File {} not in cache", filename),
            });
        }

        let response = response_js
            .dyn_into::<Response>()
            .map_err(|e| OrigaError::OcrError {
                reason: format!("Failed to cast to Response: {:?}", e),
            })?;

        let array_buffer =
            JsFuture::from(response.array_buffer().map_err(|e| OrigaError::OcrError {
                reason: format!("Failed to get array buffer: {:?}", e),
            })?)
            .await
            .map_err(|e| OrigaError::OcrError {
                reason: format!("Failed to read array buffer: {:?}", e),
            })?;

        let uint8_array = Uint8Array::new(&array_buffer);
        Ok(uint8_array.to_vec())
    }

    async fn download_and_cache_model(&self, cache: &Cache) -> Result<ModelFiles, OrigaError> {
        for filename in ModelConfig::file_names() {
            self.download_and_cache_file(cache, filename).await?;
        }

        self.load_from_cache(cache).await
    }

    async fn download_and_cache_file(
        &self,
        cache: &Cache,
        filename: &str,
    ) -> Result<(), OrigaError> {
        let url = self.config.model_file_url(filename);
        info!("Downloading {} from {}", filename, url);

        let opts = RequestInit::new();
        opts.set_method("GET");
        opts.set_mode(RequestMode::Cors);

        let request =
            Request::new_with_str_and_init(&url, &opts).map_err(|e| OrigaError::OcrError {
                reason: format!("Failed to create request: {:?}", e),
            })?;

        let window = web_sys::window().ok_or_else(|| OrigaError::OcrError {
            reason: "No window object available".to_string(),
        })?;

        let response = JsFuture::from(window.fetch_with_request(&request))
            .await
            .map_err(|e| OrigaError::OcrError {
                reason: format!("Failed to fetch {}: {:?}", filename, e),
            })?
            .dyn_into::<Response>()
            .map_err(|e| OrigaError::OcrError {
                reason: format!("Failed to cast to Response: {:?}", e),
            })?;

        if !response.ok() {
            return Err(OrigaError::OcrError {
                reason: format!(
                    "Failed to download {}: HTTP {}",
                    filename,
                    response.status()
                ),
            });
        }

        let array_buffer =
            JsFuture::from(response.array_buffer().map_err(|e| OrigaError::OcrError {
                reason: format!("Failed to get array buffer: {:?}", e),
            })?)
            .await
            .map_err(|e| OrigaError::OcrError {
                reason: format!("Failed to read response body: {:?}", e),
            })?;

        let mut data = Uint8Array::new(&array_buffer).to_vec();
        debug!("Downloaded {} bytes for {}", data.len(), filename);

        let cache_request = Request::new_with_str(filename).map_err(|e| OrigaError::OcrError {
            reason: format!("Failed to create cache request: {:?}", e),
        })?;

        let cache_response_init = web_sys::ResponseInit::new();
        let cache_response =
            Response::new_with_opt_u8_array_and_init(Some(&mut data[..]), &cache_response_init)
                .map_err(|e| OrigaError::OcrError {
                    reason: format!("Failed to create cache response: {:?}", e),
                })?;

        debug!("Caching {}", filename);
        JsFuture::from(cache.put_with_request(&cache_request, &cache_response))
            .await
            .map_err(|e| OrigaError::OcrError {
                reason: format!("Failed to cache {}: {:?}", filename, e),
            })?;

        Ok(())
    }
}
