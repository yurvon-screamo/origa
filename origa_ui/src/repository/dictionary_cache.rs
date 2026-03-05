use idb::{Database, DatabaseEvent, Factory, ObjectStoreParams, TransactionMode};
use origa::domain::{DictionaryData, OrigaError};
use wasm_bindgen::JsValue;
use web_sys::console;

const DB_NAME: &str = "origa_dictionary";
const DB_VERSION: u32 = 1;
const STORE_NAME: &str = "dictionary_cache";
const CACHE_KEY: &str = "unidic_data";
const VERSION_KEY: &str = "version";

pub const DICTIONARY_VERSION: &str = env!("CARGO_PKG_VERSION");

async fn open_database() -> Result<Database, OrigaError> {
    let factory = Factory::new().map_err(|e| {
        let reason = format!("Failed to create IndexedDB factory: {:?}", e);
        console::error_1(&reason.clone().into());
        OrigaError::RepositoryError { reason }
    })?;

    let mut open_request = factory.open(DB_NAME, Some(DB_VERSION)).map_err(|e| {
        let reason = format!("Failed to open IndexedDB: {:?}", e);
        console::error_1(&reason.clone().into());
        OrigaError::RepositoryError { reason }
    })?;

    open_request.on_upgrade_needed(|event| {
        let database = match event.database() {
            Ok(db) => db,
            Err(e) => {
                console::error_1(&format!("Failed to get database: {:?}", e).into());
                return;
            }
        };

        if database.store_names().iter().any(|n| n == STORE_NAME) {
            return;
        }

        let store_params = ObjectStoreParams::new();

        match database.create_object_store(STORE_NAME, store_params) {
            Ok(_) => console::info_1(&"Object store 'dictionary_cache' created".into()),
            Err(e) => console::error_1(&format!("Failed to create object store: {:?}", e).into()),
        }
    });

    open_request.await.map_err(|e| {
        let reason = format!("Failed to initialize IndexedDB: {:?}", e);
        console::error_1(&reason.clone().into());
        OrigaError::RepositoryError { reason }
    })
}

pub async fn get_cached_dictionary() -> Result<Option<DictionaryData>, OrigaError> {
    let db = open_database().await?;

    let transaction = db
        .transaction(&[STORE_NAME], TransactionMode::ReadOnly)
        .map_err(|e| {
            let reason = format!("Failed to create transaction: {:?}", e);
            console::error_1(&reason.clone().into());
            OrigaError::RepositoryError { reason }
        })?;

    let store = transaction.object_store(STORE_NAME).map_err(|e| {
        let reason = format!("Failed to get object store: {:?}", e);
        console::error_1(&reason.clone().into());
        OrigaError::RepositoryError { reason }
    })?;

    let version_key = JsValue::from_str(VERSION_KEY);
    let version_request = store.get(version_key).map_err(|e| {
        let reason = format!("Failed to create get version request: {:?}", e);
        console::error_1(&reason.clone().into());
        OrigaError::RepositoryError { reason }
    })?;

    let cached_version: Option<JsValue> = version_request.await.map_err(|e| {
        let reason = format!("Failed to get version: {:?}", e);
        console::error_1(&reason.clone().into());
        OrigaError::RepositoryError { reason }
    })?;

    let version_matches = cached_version
        .and_then(|v| v.as_string())
        .map(|v| v == DICTIONARY_VERSION)
        .unwrap_or(false);

    if !version_matches {
        console::info_1(&"Dictionary version mismatch, will reload".into());
        return Ok(None);
    }

    let data_key = JsValue::from_str(CACHE_KEY);
    let data_request = store.get(data_key).map_err(|e| {
        let reason = format!("Failed to create get data request: {:?}", e);
        console::error_1(&reason.clone().into());
        OrigaError::RepositoryError { reason }
    })?;

    let value: Option<JsValue> = data_request.await.map_err(|e| {
        let reason = format!("Failed to get dictionary data: {:?}", e);
        console::error_1(&reason.clone().into());
        OrigaError::RepositoryError { reason }
    })?;

    match value {
        Some(v) => {
            let data: DictionaryData = serde_wasm_bindgen::from_value(v).map_err(|e| {
                let reason = format!("Failed to deserialize dictionary data: {:?}", e);
                console::error_1(&reason.clone().into());
                OrigaError::RepositoryError { reason }
            })?;
            console::info_1(&"Dictionary loaded from cache".into());
            Ok(Some(data))
        }
        None => {
            console::info_1(&"No cached dictionary found".into());
            Ok(None)
        }
    }
}

pub async fn save_dictionary_to_cache(data: &DictionaryData) -> Result<(), OrigaError> {
    let db = open_database().await?;

    let transaction = db
        .transaction(&[STORE_NAME], TransactionMode::ReadWrite)
        .map_err(|e| {
            let reason = format!("Failed to create transaction: {:?}", e);
            console::error_1(&reason.clone().into());
            OrigaError::RepositoryError { reason }
        })?;

    let store = transaction.object_store(STORE_NAME).map_err(|e| {
        let reason = format!("Failed to get object store: {:?}", e);
        console::error_1(&reason.clone().into());
        OrigaError::RepositoryError { reason }
    })?;

    let version_value = JsValue::from_str(DICTIONARY_VERSION);
    let version_request = store.put(&version_value, Some(&JsValue::from_str(VERSION_KEY))).map_err(|e| {
        let reason = format!("Failed to create put version request: {:?}", e);
        console::error_1(&reason.clone().into());
        OrigaError::RepositoryError { reason }
    })?;
    version_request.await.map_err(|e| {
        let reason = format!("Failed to save version: {:?}", e);
        console::error_1(&reason.clone().into());
        OrigaError::RepositoryError { reason }
    })?;

    let data_value = serde_wasm_bindgen::to_value(data).map_err(|e| {
        let reason = format!("Failed to serialize dictionary data: {:?}", e);
        console::error_1(&reason.clone().into());
        OrigaError::RepositoryError { reason }
    })?;

    let data_request = store.put(&data_value, Some(&JsValue::from_str(CACHE_KEY))).map_err(|e| {
        let reason = format!("Failed to create put data request: {:?}", e);
        console::error_1(&reason.clone().into());
        OrigaError::RepositoryError { reason }
    })?;

    data_request.await.map_err(|e| {
        let reason = format!("Failed to save dictionary data: {:?}", e);
        console::error_1(&reason.clone().into());
        OrigaError::RepositoryError { reason }
    })?;

    console::info_1(&"Dictionary saved to cache".into());
    Ok(())
}
