# OAuth Configuration Guide

## Google OAuth Setup

Google поддерживается Supabase напрямую.

### 1. Настройка Google Cloud Console

1. Перейдите в [Google Cloud Console](https://console.cloud.google.com/)
2. Создайте новый проект или выберите существующий
3. Перейдите в **APIs & Services** → **Credentials**
4. Нажмите **Create Credentials** → **OAuth client ID**
5. Выберите **Web application**
6. Добавьте Authorized redirect URIs:
   ```
   https://<your-project-ref>.supabase.co/auth/v1/callback
   ```
7. Скопируйте **Client ID** и **Client Secret**

### 2. Настройка Supabase

1. Откройте Supabase Dashboard
2. Перейдите в **Authentication** → **Providers**
3. Включите **Google**
4. Вставьте Client ID и Client Secret
5. Сохраните изменения

## Yandex OAuth Setup (via Keycloak Provider)

Yandex настраивается через **Keycloak** провайдер в Supabase (Keycloak поддерживается напрямую).

### Шаг 1: Создание OAuth приложения в Yandex

1. Перейдите в [Yandex OAuth](https://oauth.yandex.ru/client/new)
2. Заполните форму:
   - **Название**: Origa
   - **Платформа**: Веб-сервисы
   - **Redirect URI**: 
     ```
     https://<your-project-ref>.supabase.co/auth/v1/callback
     ```
   - **Доступ к данным**: Выберите:
     - `login:email` - Email адрес
     - `login:info` - Имя, фамилия
3. Нажмите **Создать приложение**
4. Скопируйте:
   - **ID** (Client ID)
   - **Пароль** (Client Secret)

### Шаг 2: Настройка Keycloak провайдера в Supabase

1. Откройте Supabase Dashboard
2. Перейдите в **Authentication** → **Providers**
3. Включите **Keycloak**
4. Заполните поля:
   - **Client ID**: ID из Yandex OAuth
   - **Client Secret**: Пароль из Yandex OAuth
   - **URL**: `https://oauth.yandex.ru` (или ваш Yandex ID endpoint)
5. Сохраните изменения

### Шаг 3: Использование

Код уже настроен на использование Keycloak провайдера для Yandex:

```rust
OAuthProvider::Yandex => "keycloak"
```

При клике на кнопку "Войти через Yandex" будет использоваться Keycloak провайдер.

## Supabase URL Configuration

Не забудьте добавить redirect URLs:

1. Supabase Dashboard → **Authentication** → **URL Configuration**
2. Добавьте в **Redirect URLs**:
   - Для desktop (Tauri):
     ```
     origa://auth/callback
     ```
   - Для веб-версии:
     ```
     http://localhost:1420/login
     https://your-domain.com/login
     ```
3. Сохраните изменения

## Testing OAuth Flow

### Desktop (Windows/Linux/macOS)

1. Соберите и установите приложение:
   ```bash
   cargo tauri build
   ```
2. Установите приложение
3. Откройте браузер и перейдите по:
   ```
   origa://auth/callback#access_token=test
   ```
4. Приложение должно открыться и обработать deep link

### Web Version

Для веб-версии OAuth работает через редирект:

1. Пользователь кликает "Войти через Google/Yandex"
2. Происходит редирект на Supabase OAuth
3. После авторизации пользователь возвращается на `/login#access_token=...`
4. Приложение парсит токены из URL hash и создаёт сессию
5. Редирект на `/home`

Для тестирования локально:
```bash
cd origa_ui && trunk serve
# Откройте http://localhost:1420/login#access_token=test
```

### Development Mode

Для тестирования в dev режиме на Windows/Linux:

```rust
// В lib.rs уже добавлено:
#[cfg(any(windows, target_os = "linux"))]
{
    app.deep_link().register_all()?;
}
```

Это регистрирует deep link scheme для dev сборки.

## Troubleshooting

### Deep link не открывается на Windows
- Убедитесь, что приложение установлено (или запущен `register_all()`)
- Проверьте реестр: `HKEY_CLASSES_ROOT\origa`

### Deep link не открывается на macOS  
- Deep links работают только для установленного приложения в `/Applications`
- В dev режиме не работает

### OAuth callback не обрабатывается
- Проверьте URL в Supabase Dashboard → Authentication → URL Configuration
- Убедитесь, что `origa://auth/callback` (desktop) или `/login` (web) добавлены в список

### Web version: OAuth не работает
- Проверьте, что redirect URL `{origin}/login` добавлен в Supabase
- Откройте DevTools → Network, проверьте запрос к Supabase OAuth
- Проверьте консоль на ошибки при парсинге токенов

### Yandex OAuth возвращает ошибку
- Проверьте правильность Client ID и Client Secret
- Убедитесь, что redirect URI совпадает с настройками Yandex OAuth
