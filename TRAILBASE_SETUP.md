# TrailBase Setup Guide

## Обязательные шаги перед запуском

### 1. Создание таблицы в базе данных

1. Откройте TrailBase Admin Panel: `https://trailbase.uwuwu.net/_/admin/`
2. Перейдите в SQL Editor: `/_/admin/editor`
3. Выполните SQL из файла `trailbase_schema.sql`:

```sql
CREATE TABLE user (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    auth_user_id TEXT UNIQUE NOT NULL,
    username TEXT NOT NULL,
    email TEXT NOT NULL,
    native_language INTEGER NOT NULL DEFAULT 0,
    jlpt_progress TEXT CHECK(json_valid(jlpt_progress)),
    current_japanese_level INTEGER,
    duolingo_jwt_token TEXT,
    telegram_user_id INTEGER,
    reminders_enabled INTEGER NOT NULL DEFAULT 0,
    knowledge_set TEXT NOT NULL DEFAULT 'Default',
    updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
) STRICT;

CREATE INDEX idx_user_auth_user_id ON user(auth_user_id);
CREATE INDEX idx_user_email ON user(email);
```

### 2. Настройка OAuth провайдеров

1. В Admin Panel перейдите в раздел **Auth** → **Providers**
2. Настройте провайдеры:

#### Google OAuth
- **Provider Name**: `google`
- **Client ID**: Ваш Google OAuth Client ID
- **Client Secret**: Ваш Google OAuth Client Secret
- **Redirect URI**: 
  - Tauri: `origa://auth/callback`
  - Web: `https://yourdomain.com/login`
- **Scopes**: `email profile`

#### Yandex OAuth (Keycloak)
- **Provider Name**: `keycloak`
- **Client ID**: Ваш Yandex/Keycloak Client ID
- **Client Secret**: Ваш Yandex/Keycloak Client Secret
- **Redirect URI**: 
  - Tauri: `origa://auth/callback`
  - Web: `https://yourdomain.com/login`
- **Scopes**: `email profile`

### 3. Настройка Record API Permissions

1. В Admin Panel перейдите в **Records** → выберите таблицу `user`
2. Настройте permissions:
   - **Read**: Authenticated users могут читать свои записи
   - **Create**: Authenticated users могут создавать записи
   - **Update**: Authenticated users могут обновлять свои записи
   - **Delete**: Authenticated users могут удалять свои записи

### 4. Проверка конфигурации

Убедитесь что в `origa_ui/src/repository/trailbase_client.rs:7` указан правильный URL:
```rust
const TRAILBASE_URL: &str = "https://trailbase.uwuwu.net";
```

## Реализованные исправления

### ✅ JWT Claims декодирование
- Email и auth_user_id (sub) извлекаются из JWT токена
- Если email отсутствует в токене - возвращается ошибка (без fallback)
- Реализовано в `trailbase_client.rs:decode_jwt_claims()`

### ✅ Record API ID handling
- После создания записи сохраняется числовой `record_id` в сессию
- Для update/delete используется числовой `id`, а не email/auth_user_id
- Query filter `email=eq.{email}` для поиска записей

### ✅ Убран fallback email
- Если email отсутствует в JWT - пользователь получает ошибку
- Нет риска конфликтов между пользователями

## Тестирование

1. Откройте приложение
2. Нажмите "Войти через Google" или "Войти через Yandex"
3. Пройдите OAuth авторизацию
4. Проверьте что:
   - Пользователь создан в таблице `user`
   - Email и auth_user_id заполнены корректно
   - При повторном входе загружается существующий профиль

## Диагностика проблем

### "Email не найден в токене авторизации"
**Причина**: OAuth провайдер не возвращает email в JWT claims
**Решение**: Проверьте настройки OAuth провайдера - scopes должны включать `email`

### "Record ID missing from database row"
**Причина**: Таблица создана без AUTOINCREMENT для id
**Решение**: Пересоздайте таблицу с правильной схемой

### "Failed to decode JWT"
**Причина**: Некорректный формат токена от TrailBase
**Решение**: Проверьте что TrailBase возвращает стандартный JWT формат
