use idb::{Database, DatabaseEvent, Factory, ObjectStoreParams, TransactionMode};
use origa::{
    domain::{OrigaError, User},
    traits::UserRepository,
};
use ulid::Ulid;
use wasm_bindgen::JsValue;

use crate::loaders::recalculate_user_jlpt_progress;

const DB_NAME: &str = "origa";
const DB_VERSION: u32 = 1;
const STORE_NAME: &str = "users";

fn user_key(user_id: Ulid) -> String {
    format!("user:{}", user_id)
}

async fn open_database() -> Result<Database, OrigaError> {
    let factory = Factory::new().map_err(|e| {
        let reason = format!("Failed to create IndexedDB factory: {:?}", e);
        tracing::error!("{}", reason);
        OrigaError::RepositoryError { reason }
    })?;

    let mut open_request = factory.open(DB_NAME, Some(DB_VERSION)).map_err(|e| {
        let reason = format!("Failed to open IndexedDB: {:?}", e);
        tracing::error!("{}", reason);
        OrigaError::RepositoryError { reason }
    })?;

    open_request.on_upgrade_needed(|event| {
        let database = match event.database() {
            Ok(db) => db,
            Err(e) => {
                tracing::error!("Failed to get database: {:?}", e);
                return;
            },
        };

        if database.store_names().iter().any(|n| n == STORE_NAME) {
            return;
        }

        let store_params = ObjectStoreParams::new();

        match database.create_object_store(STORE_NAME, store_params) {
            Ok(_) => tracing::info!("Object store 'users' created"),
            Err(e) => tracing::error!("Failed to create object store: {:?}", e),
        }
    });

    open_request.await.map_err(|e| {
        let reason = format!("Failed to initialize IndexedDB: {:?}", e);
        tracing::error!("{}", reason);
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
                tracing::error!("{}", reason);
                OrigaError::RepositoryError { reason }
            })?;

        let store = transaction.object_store(STORE_NAME).map_err(|e| {
            let reason = format!("Failed to get object store: {:?}", e);
            tracing::error!("{}", reason);
            OrigaError::RepositoryError { reason }
        })?;

        let request = store.get_all(None, None).map_err(|e| {
            let reason = format!("Failed to create get_all request: {:?}", e);
            tracing::error!("{}", reason);
            OrigaError::RepositoryError { reason }
        })?;

        let all_values: Vec<JsValue> = request.await.map_err(|e| {
            let reason = format!("Failed to get all users: {:?}", e);
            tracing::error!("{}", reason);
            OrigaError::RepositoryError { reason }
        })?;

        let mut users = vec![];
        for value in all_values {
            let user: User = serde_wasm_bindgen::from_value(value).map_err(|e| {
                let reason = format!("Failed to deserialize user: {:?}", e);
                tracing::error!("{}", reason);
                OrigaError::RepositoryError { reason }
            })?;
            users.push(user);
        }

        Ok(users)
    }
}

impl UserRepository for FileSystemUserRepository {
    async fn get_current_user(&self) -> Result<Option<User>, OrigaError> {
        let users = self.list_users().await?;
        Ok(users.first().cloned())
    }

    async fn save(&self, user: &User) -> Result<(), OrigaError> {
        let mut user_clone = user.clone();
        recalculate_user_jlpt_progress(&mut user_clone);

        let db = open_database().await?;

        let transaction = db
            .transaction(&[STORE_NAME], TransactionMode::ReadWrite)
            .map_err(|e| {
                let reason = format!("Failed to create transaction: {:?}", e);
                tracing::error!("{}", reason);
                OrigaError::RepositoryError { reason }
            })?;

        let store = transaction.object_store(STORE_NAME).map_err(|e| {
            let reason = format!("Failed to get object store: {:?}", e);
            tracing::error!("{}", reason);
            OrigaError::RepositoryError { reason }
        })?;

        let key = user_key(user_clone.id());
        let value = serde_wasm_bindgen::to_value(&user_clone).map_err(|e| {
            let reason = format!("Failed to serialize user: {:?}", e);
            tracing::error!("{}", reason);
            OrigaError::RepositoryError { reason }
        })?;

        let request = store
            .put(&value, Some(&JsValue::from_str(&key)))
            .map_err(|e| {
                let reason = format!("Failed to create put request: {:?}", e);
                tracing::error!("{}", reason);
                OrigaError::RepositoryError { reason }
            })?;

        request.await.map_err(|e| {
            let reason = format!("Failed to save user: {:?}", e);
            tracing::error!("{}", reason);
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
                tracing::error!("{}", reason);
                OrigaError::RepositoryError { reason }
            })?;

        let store = transaction.object_store(STORE_NAME).map_err(|e| {
            let reason = format!("Failed to get object store: {:?}", e);
            tracing::error!("{}", reason);
            OrigaError::RepositoryError { reason }
        })?;

        let key = JsValue::from_str(&user_key(user_id));

        let request = store.delete(key).map_err(|e| {
            let reason = format!("Failed to create delete request: {:?}", e);
            tracing::error!("{}", reason);
            OrigaError::RepositoryError { reason }
        })?;

        request.await.map_err(|e| {
            let reason = format!("Failed to delete user: {:?}", e);
            tracing::error!("{}", reason);
            OrigaError::RepositoryError { reason }
        })?;

        Ok(())
    }
}
