# Настройка Firebase User Repository

Этот документ описывает альтернативную реализацию `UserRepository` через Google Firebase Firestore.

## 🚀 Быстрый старт

### 1. Настройка Google Cloud проекта

```bash
# Создать новый проект (или использовать существующий)
gcloud projects create your-project-id

# Установить проект как текущий
gcloud config set project your-project-id

# Включить Firestore API
gcloud services enable firestore.googleapis.com
```

### 2. Создание базы данных Firestore

```bash
# Создать Firestore базу данных в режиме Native
gcloud firestore databases create --region=us-central1
```

### 3. Настройка аутентификации

#### Вариант A: Через сервисный аккаунт (рекомендуется для продакшена)

```bash
# Создать сервисный аккаунт
gcloud iam service-accounts create origa-firebase-sa \
    --description="Service account for Origa Firebase integration" \
    --display-name="Origa Firebase SA"

# Назначить роли
gcloud projects add-iam-policy-binding your-project-id \
    --member="serviceAccount:origa-firebase-sa@your-project-id.iam.gserviceaccount.com" \
    --role="roles/datastore.user"

# Создать и скачать ключи
gcloud iam service-accounts keys create firebase-key.json \
    --iam-account=origa-firebase-sa@your-project-id.iam.gserviceaccount.com

# Активировать сервисный аккаунт
gcloud auth activate-service-account --key-file=firebase-key.json

# Получить access token
gcloud auth application-default print-access-token
```

#### Вариант B: Через личный аккаунт (для разработки)

```bash
# Авторизоваться
gcloud auth login

# Получить access token
gcloud auth application-default print-access-token
```

### 4. Использование в коде

```rust
use origa::infrastructure::FirebaseUserRepository;
use origa::application::UserRepository;
use origa::domain::{User, JapaneseLevel, NativeLanguage};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let repository = FirebaseUserRepository::new(
        "your-project-id".to_string(),
        None, // Использует "(default)" базу данных
        "your-access-token".to_string(),
    )
    .await?
    .with_collection_name("users".to_string());

    // Создать пользователя
    let user = User::new(
        "test_user".to_string(),
        JapaneseLevel::Beginner,
        NativeLanguage::English,
    );

    // Сохранить
    repository.save(&user).await?;

    // Найти по ID
    let found_user = repository.find_by_id(user.id()).await?;

    // Получить всех пользователей
    let all_users = repository.list().await?;

    // Удалить
    repository.delete(user.id()).await?;

    Ok(())
}
```

## 🔧 Конфигурация

### Переменные окружения

```bash
# Обязательные
export FIREBASE_PROJECT_ID="your-project-id"
export FIREBASE_ACCESS_TOKEN="your-access-token"

# Необязательные
export FIREBASE_DATABASE_ID="(default)"  # По умолчанию
export FIREBASE_COLLECTION_NAME="users"  # По умолчанию
```

## 🔐 Безопасность

### Правила безопасности Firestore

```javascript
rules_version = '2';
service cloud.firestore {
  match /databases/{database}/documents {
    // Правила для коллекции пользователей
    match /users/{userId} {
      // Разрешить чтение и запись только для аутентифицированных пользователей
      allow read, write: if request.auth != null;
    }
    
    // Более строгие правила - пользователь может работать только со своими данными
    match /users/{userId} {
      allow read, write: if request.auth != null && request.auth.uid == userId;
    }
  }
}
```

### Применение правил безопасности

```bash
# Создать файл firestore.rules и применить
gcloud firestore rules deploy firestore.rules
```

## 🚨 Обработка ошибок

Firebase репозиторий может генерировать следующие типы ошибок:

- **HTTP ошибки**: 401 (неавторизован), 403 (запрещено), 404 (не найдено)
- **Сериализация**: Ошибки при преобразовании данных в JSON и обратно
- **Сетевые ошибки**: Таймауты, недоступность сервиса

Пример обработки:

```rust
match repository.save(&user).await {
    Ok(_) => println!("Пользователь сохранён"),
    Err(OrigaError::RepositoryError { reason }) if reason.contains("401") => {
        println!("Ошибка аутентификации: {}", reason);
        // Обновить access token
    }
    Err(e) => println!("Другая ошибка: {}", e),
}
```

## 🧪 Тестирование

### Запуск примера

```bash
# Установить переменные окружения
export FIREBASE_PROJECT_ID="your-project-id"
export FIREBASE_ACCESS_TOKEN="$(gcloud auth application-default print-access-token)"

# Запустить пример
cargo run --example firebase_example --features="examples"
```

### Юнит-тесты

```bash
# Запустить тесты (не требуют реального подключения к Firebase)
cargo test firebase_user_repository

# Интеграционные тесты (требуют настроенный Firebase проект)
FIREBASE_PROJECT_ID="test-project" \
FIREBASE_ACCESS_TOKEN="token" \
cargo test --test firebase_integration
```

## 📈 Производительность и ограничения

### Лимиты Firebase

- **Чтение**: 50,000 операций в день (бесплатный план)
- **Запись**: 20,000 операций в день (бесплатный план)
- **Хранение**: 1 ГБ (бесплатный план)

### Оптимизация

1. **Батчевые операции**: Группируйте операции для уменьшения количества запросов
2. **Кеширование**: Кешируйте часто запрашиваемые данные локально
3. **Индексы**: Создавайте составные индексы для сложных запросов

## 🔄 Миграция с FileSystemUserRepository

```rust
use origa::infrastructure::{FileSystemUserRepository, FirebaseUserRepository};

async fn migrate_to_firebase() -> Result<(), Box<dyn std::error::Error>> {
    // Старый репозиторий
    let fs_repo = FileSystemUserRepository::new("./data".into()).await?;
    
    // Новый репозиторий
    let firebase_repo = FirebaseUserRepository::new(
        "project-id".to_string(),
        None,
        "token".to_string(),
    ).await?;
    
    // Перенести всех пользователей
    let users = fs_repo.list().await?;
    for user in users {
        firebase_repo.save(&user).await?;
        println!("Migrated user: {}", user.username());
    }
    
    Ok(())
}
```
