use idb::{Database, DatabaseEvent, Factory, ObjectStoreParams, TransactionMode};
use origa::{
    application::UserRepository,
    domain::{OrigaError, User},
};
use serde_wasm_bindgen;
use ulid::Ulid;
use wasm_bindgen::JsValue;
use web_sys::console;

const DB_NAME: &str = "origa";
const DB_VERSION: u32 = 1;
const STORE_NAME: &str = "users";

fn user_key(user_id: Ulid) -> String {
    format!("user:{}", user_id)
}

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
            Ok(_) => console::info_1(&"Object store 'users' created".into()),
            Err(e) => console::error_1(&format!("Failed to create object store: {:?}", e).into()),
        }
    });

    open_request.await.map_err(|e| {
        let reason = format!("Failed to initialize IndexedDB: {:?}", e);
        console::error_1(&reason.clone().into());
        OrigaError::RepositoryError { reason }
    })
}

#[derive(Clone)]
pub struct FileSystemUserRepository {}

impl FileSystemUserRepository {
    pub fn new() -> Self {
        Self {}
    }

    async fn list_users(&self) -> Result<Vec<User>, OrigaError> {
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

        let request = store.get_all(None, None).map_err(|e| {
            let reason = format!("Failed to create get_all request: {:?}", e);
            console::error_1(&reason.clone().into());
            OrigaError::RepositoryError { reason }
        })?;

        let all_values: Vec<JsValue> = request.await.map_err(|e| {
            let reason = format!("Failed to get all users: {:?}", e);
            console::error_1(&reason.clone().into());
            OrigaError::RepositoryError { reason }
        })?;

        let mut users = vec![];
        for value in all_values {
            let user: User = serde_wasm_bindgen::from_value(value).map_err(|e| {
                let reason = format!("Failed to deserialize user: {:?}", e);
                console::error_1(&reason.clone().into());
                OrigaError::RepositoryError { reason }
            })?;
            users.push(user);
        }

        Ok(users)
    }
}

impl UserRepository for FileSystemUserRepository {
    async fn find_by_id(&self, user_id: Ulid) -> Result<Option<User>, OrigaError> {
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

        let key = JsValue::from_str(&user_key(user_id));
        let request = store.get(key).map_err(|e| {
            let reason = format!("Failed to create get request: {:?}", e);
            console::error_1(&reason.clone().into());
            OrigaError::RepositoryError { reason }
        })?;

        let value: Option<JsValue> = request.await.map_err(|e| {
            let reason = format!("Failed to get user: {:?}", e);
            console::error_1(&reason.clone().into());
            OrigaError::RepositoryError { reason }
        })?;

        match value {
            Some(v) => {
                let user: User = serde_wasm_bindgen::from_value(v).map_err(|e| {
                    let reason = format!("Failed to deserialize user: {:?}", e);
                    console::error_1(&reason.clone().into());
                    OrigaError::RepositoryError { reason }
                })?;
                Ok(Some(user))
            }
            None => {
                console::warn_1(&format!("User not found: {}", user_id).into());
                Ok(None)
            }
        }
    }

    async fn find_by_email(&self, email: &str) -> Result<Option<User>, OrigaError> {
        let users = self.list_users().await?;
        Ok(users.into_iter().find(|x| x.email() == email))
    }

    async fn find_by_telegram_id(&self, telegram_id: &u64) -> Result<Option<User>, OrigaError> {
        let users = self.list_users().await?;
        Ok(users
            .into_iter()
            .find(|x| x.telegram_user_id() == Some(telegram_id)))
    }

    async fn save(&self, user: &User) -> Result<(), OrigaError> {
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

        let key = user_key(user.id());
        let value = serde_wasm_bindgen::to_value(user).map_err(|e| {
            let reason = format!("Failed to serialize user: {:?}", e);
            console::error_1(&reason.clone().into());
            OrigaError::RepositoryError { reason }
        })?;

        let request = store
            .put(&value, Some(&JsValue::from_str(&key)))
            .map_err(|e| {
                let reason = format!("Failed to create put request: {:?}", e);
                console::error_1(&reason.clone().into());
                OrigaError::RepositoryError { reason }
            })?;

        request.await.map_err(|e| {
            let reason = format!("Failed to save user: {:?}", e);
            console::error_1(&reason.clone().into());
            OrigaError::RepositoryError { reason }
        })?;

        Ok(())
    }

    async fn delete(&self, user_id: Ulid) -> Result<(), OrigaError> {
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

        let key = JsValue::from_str(&user_key(user_id));

        let request = store.delete(key).map_err(|e| {
            let reason = format!("Failed to create delete request: {:?}", e);
            console::error_1(&reason.clone().into());
            OrigaError::RepositoryError { reason }
        })?;

        request.await.map_err(|e| {
            let reason = format!("Failed to delete user: {:?}", e);
            console::error_1(&reason.clone().into());
            OrigaError::RepositoryError { reason }
        })?;

        Ok(())
    }
}
