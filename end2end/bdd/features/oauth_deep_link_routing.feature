# language: ru
Функция: Маршрутизация deep-link в oauth-листенере
  Баг #1482: oauth-листенер прогонял через process_oauth_url любой origa:// URL,
  и чужие deep-link (selftest-проба android-smoke, шорткат «добавить слово»)
  ронялись в error-ветку → error-строка в консоли (tracing-wasm шлёт её через
  console.log) + ложный алерт «unsupported callback»
  на /login. Фикс: предикат is_auth_callback_url в трёх путях входа — oauth
  слушает только auth-callback URL (origa://auth/callback*).

  Приложение запускается в моке window.__TAURI__ (транспортная граница
  core/tauri.rs): юзер не-auth, открыт /login — состояние бага #1482.

  Сценарий: Чужой deep-link событие молчит на странице входа
    Допустим приложение запущено в моке Tauri без сессии
    Когда приходит deep-link "origa://selftest-ocr"
    Тогда ошибки oauth на странице входа нет

  Сценарий: Чужой deep-link при старте приложения молчит на странице входа
    Допустим приложение запущено в моке Tauri без сессии с текущим deep-link "origa://add?source=camera"
    Тогда ошибки oauth на странице входа нет

  Сценарий: Настоящий auth-callback запускает OAuth-обмен
    Допустим приложение запущено в моке Tauri без сессии
    Когда приходит deep-link "origa://auth/callback?code=e2e-stale-code"
    Тогда запрос обмена кода уходит в TrailBase
