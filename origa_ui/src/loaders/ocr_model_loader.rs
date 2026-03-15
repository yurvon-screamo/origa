use js_sys::{Reflect, Uint8Array};
use origa::domain::OrigaError;
use origa::ocr::{ModelConfig, ModelFiles};
use std::cell::RefCell;
use std::rc::Rc;
use tracing::{debug, info};
use wasm_bindgen::{JsCast, JsValue};
use wasm_bindgen_futures::JsFuture;
use web_sys::{
    Cache, ReadableStream, ReadableStreamDefaultReader, Request, RequestInit, RequestMode,
    Response, Window,
};

/// Approximate size of Deim model in bytes (50 MB)
pub const _DEIM_SIZE: u64 = 52_428_800;
/// Approximate size of Parseq models combined in bytes (100 MB)
pub const _PARSEQ_SIZE: u64 = 104_857_600;
/// Number of model files required for NDLOCR-Lite:
/// deim.onnx, parseq30.onnx, parseq50.onnx, parseq100.onnx, vocab.txt
const MODEL_FILE_COUNT: usize = 5;

/// Progress callback type: (filename, loaded_bytes, total_bytes)
pub type ProgressCallback = Rc<dyn Fn(&str, u64, u64)>;

fn js_err(msg: impl AsRef<str>, e: &JsValue) -> OrigaError {
    OrigaError::OcrError {
        reason: format!("{}: {:?}", msg.as_ref(), e),
    }
}

fn sanitize_cache_name(name: &str) -> String {
    const MAX_CACHE_NAME_LEN: usize = 64;

    let sanitized: String = name
        .chars()
        .take(MAX_CACHE_NAME_LEN)
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '-' || c == '_' {
                c
            } else {
                '_'
            }
        })
        .collect();

    let sanitized = sanitized.replace("..", "_");

    if sanitized.is_empty() {
        "default_cache".to_string()
    } else {
        sanitized
    }
}

pub struct ModelLoader {
    config: ModelConfig,
    on_progress: Option<ProgressCallback>,
}

impl ModelLoader {
    pub fn new(config: ModelConfig) -> Self {
        Self {
            config,
            on_progress: None,
        }
    }

    pub fn with_progress_callback(mut self, callback: ProgressCallback) -> Self {
        self.on_progress = Some(callback);
        self
    }

    pub async fn load_or_download_model(&self) -> Result<ModelFiles, OrigaError> {
        info!(
            "Loading NDLOCR-Lite models from {}",
            self.config.ndlocr_base_url
        );

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
        if loaded.len() != MODEL_FILE_COUNT {
            return Err(OrigaError::OcrError {
                reason: format!(
                    "Expected {} model files, got {}",
                    MODEL_FILE_COUNT,
                    loaded.len()
                ),
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

        let total_bytes = self.get_content_length(&response);

        let stream = response.body().ok_or_else(|| OrigaError::OcrError {
            reason: "Response body is not a stream".to_string(),
        })?;

        let reader = self.get_stream_reader(&stream)?;

        let data = self
            .read_stream_with_progress(&reader, filename, total_bytes)
            .await?;

        debug!("Downloaded {} bytes for {}", data.len(), filename);

        // Clone required: Response::new_with_opt_u8_array_and_init takes &mut [u8]
        // and may modify/consume it during cache storage. We must preserve original data.
        let mut data_for_cache = data.clone();
        let cache_request = Request::new_with_str(filename)
            .map_err(|e| js_err("Failed to create cache request", &e))?;

        let cache_response_init = web_sys::ResponseInit::new();
        let cache_response = Response::new_with_opt_u8_array_and_init(
            Some(&mut data_for_cache[..]),
            &cache_response_init,
        )
        .map_err(|e| js_err("Failed to create cache response", &e))?;

        debug!("Caching {}", filename);
        JsFuture::from(cache.put_with_request(&cache_request, &cache_response))
            .await
            .map_err(|e| js_err(format!("Failed to cache {}", filename), &e))?;

        Ok(data)
    }

    fn get_content_length(&self, response: &Response) -> Option<u64> {
        response
            .headers()
            .get("content-length")
            .ok()
            .flatten()
            .and_then(|s| s.parse::<u64>().ok())
    }

    fn get_stream_reader(
        &self,
        stream: &ReadableStream,
    ) -> Result<ReadableStreamDefaultReader, OrigaError> {
        stream
            .get_reader()
            .dyn_into::<ReadableStreamDefaultReader>()
            .map_err(|e| js_err("Failed to cast to ReadableStreamDefaultReader", &e))
    }

    async fn read_stream_with_progress(
        &self,
        reader: &ReadableStreamDefaultReader,
        filename: &str,
        total_bytes: Option<u64>,
    ) -> Result<Vec<u8>, OrigaError> {
        let chunks: Rc<RefCell<Vec<Vec<u8>>>> = Rc::new(RefCell::new(Vec::new()));
        let loaded: Rc<RefCell<u64>> = Rc::new(RefCell::new(0));

        loop {
            let read_result = JsFuture::from(reader.read())
                .await
                .map_err(|e| js_err("Failed to read from stream", &e))?;

            let done = Reflect::get(&read_result, &JsValue::from_str("done"))
                .map_err(|e| js_err("Failed to get done flag", &e))?
                .as_bool()
                .unwrap_or(true);

            if done {
                break;
            }

            let value = Reflect::get(&read_result, &JsValue::from_str("value"))
                .map_err(|e| js_err("Failed to get chunk value", &e))?;

            let chunk = Uint8Array::new(&value);
            let chunk_len = chunk.length() as u64;
            let chunk_vec = chunk.to_vec();

            chunks.borrow_mut().push(chunk_vec);

            let mut loaded_guard = loaded.borrow_mut();
            *loaded_guard += chunk_len;

            if let Some(ref callback) = self.on_progress {
                callback(filename, *loaded_guard, total_bytes.unwrap_or(0));
            }
        }

        let chunks_guard = chunks.borrow();
        let total_size: usize = chunks_guard.iter().map(|c| c.len()).sum();
        let mut result = Vec::with_capacity(total_size);
        for chunk in chunks_guard.iter() {
            result.extend_from_slice(chunk);
        }

        Ok(result)
    }
}
