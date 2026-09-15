# ADR-056: iOS manual signing вместо минта сертификатов на каждом CI-прогоне

- Статус: Принято
- Дата: 2026-09-15
- Контекст: инциденты 0.7.19 — ITMS-90721 «Certificate Revoked» на iOS-билдах
  в App Store Connect + регулярные письма «null null has revoked your
  certificate (Distribution)»

## Контекст

После PR #505 (нативный SIWA) iOS-джоба в `_build-tauri-apple.yml` работала по
схеме «минт на каждый прогон»: скрипт `mint_ios_identity.py` создавал через
App Store Connect API свежий Apple Distribution сертификат + IOS_APP_STORE
профиль, подписывал ими пере-экспортированный .ipa, а шаг «Cleanup minted iOS
identity» **отзывал сертификат** (`DELETE /v1/certificates/{id}`) сразу после
re-sign — **до** загрузки в ASC. Слоты сертификатов ограничены, поэтому чистка
казалась обязательной.

Последствия:

- **ITMS-90721**: altool грузит .ipa, подписанный уже отозванным сертификатом;
  ASC валидирует билд асинхронно и в зависимости от гонки распространения
  ревокации отклоняет его с «Certificate Revoked». Apple прямо предупреждает:
  ревокация сертификата валит все in-flight билды, им подписанные.
- **Письма «null null has revoked your certificate»**: каждая программная
  ревокация через ASC API-ключ шлёт владельцу аккаунта письмо; actor без
  человеческого имени рендерится шаблоном как «null null».
- **Гонка параллельных прогонов**: `purge_previous_ci_identities()` удаляла все
  `origa-ci-*` профили и их сертификаты — retry релиза или подряд идущие rc-теги
  отзывали сертификат соседнего прогона в полёте.

Корневая причина всего цикла — auto-provisioning: с видимым `APPLE_API_*`
кредами tauri-cli вообще не подписывает архив
(`skip_signing = credentials.is_some()`, `mobile/ios/build.rs`) и доверяет
подпись export-time `-allowProvisioningUpdates`, который минтит сертификаты.
Профиль, выданный auto-provisioning, не содержал capability Sign in with Apple
(ASC кэширует профили) — отсюда и возник re-sign-обход в #505.

## Решение

Официальный **Tauri Manual Signing**
(<https://v2.tauri.app/distribute/sign/ios/>) — статичные сертификат и профиль
в секретах, нулевой churn на ASC API:

- `IOS_CERTIFICATE` (base64 P12 «Apple Distribution») +
  `IOS_CERTIFICATE_PASSWORD` + `IOS_MOBILE_PROVISION` (base64
  .mobileprovision) подаются в env шага `cargo tauri ios build`.
- tauri-cli сам создаёт временный keychain из P12, устанавливает профиль в
  `~/Library/Developer/MobileDevice/`, ставит `CODE_SIGN_STYLE=Manual` +
  identity + profile UUID в pbxproj и ExportOptions.plist.
- Без `APPLE_API_*` в env сборки `auth_credentials_from_env()` возвращает
  `None` → архив подписывается сразу с identity + `CODE_SIGN_ENTITLEMENTS`
  (entitlements-файл с applesignin уже подключён в project.yml) — исходная
  проблема #505 закрывается штатным путём.
- Репозиторные секреты: `APPLE_IOS_CERTIFICATE`,
  `APPLE_IOS_CERTIFICATE_PASSWORD`, `APPLE_IOS_MOBILE_PROVISION`.

### Инвариант

`APPLE_API_*` не должны попадать в env шага build iOS-джобы — ни через
step-env, ни через `$GITHUB_ENV`-экспорт (шаг «Set API key env» удалён).
Креды остаются только там, где нужны: `decode-apple-key` пишет
`./private_keys/AuthKey_<ID>.p8` для `xcrun altool` (конвенция поиска),
upload-шаг использует `--apiKey/--apiIssuer`. macOS-джоба не затронута:
`cargo tauri build` (desktop) не читает эти переменные.

### Удалено

- Шаги «Re-sign .ipa with SIWA entitlements» и «Cleanup minted iOS identity».
- `mint_ios_identity.py` — весь его сценарий (минт/ревок) умер.
- `manage_ios_profiles.py` — flush кэша ASC-профилей нужен был только
  auto-provisioning; при manual signing профиль фиксирован секретом.
- Fail-closed гейт «Verify iOS entitlements (SIWA)» **сохранён** перед upload:
  если manual signing когда-нибудь не применит entitlements, CI упадёт до
  загрузки, а не отдаст сломанный билд в ASC.

## Альтернативы

- **Оставить минт, отложить ревокацию до конца обработки билда в ASC**
  (поллинг статуса): письма продолжают приходить, +15–30 мин macOS-раннера на
  релиз, гонка параллельных purge остаётся. Отвергнуто.
- **fastlane match** (индустриальный стандарт синка cert+profile): тащит
  Ruby-тулинг в Rust-проект ради задачи, которую tauri-cli решает нативно тремя
  env-переменными. Отвергнуто как overkill для соло-проекта.

## Последствия

- Ноль ревокаций сертификатов из CI → нет 90721 и писем «null null».
- Годовая ротация секрета cert+profile (см. runbook) — та же дисциплина, что у
  существующих `APPLE_MAC_APP_CERT_P12` macOS-секретов.
- При смене capabilities App ID профиль надо перегенерировать в портале и
  обновить секрет (manual signing не обновляет его автоматически — осознанный
  trade-off).
- **Runbook ротации** (раз в год, до истечения срока действия сертификата):
  1. Портал → Certificates: создать новый Apple Distribution (CSR/P12), не
     отзывая старый до завершения перехода.
  2. Портал → Profiles: отредактировать App Store профиль net.uwuwu.origa,
     привязав новый сертификат, скачать.
  3. Обновить три секрета в GitHub.
  4. Прогнать `gh workflow run tauri.yml -f force_apple=true --ref master`.
  5. Только после зелёного прогона отозвать старый сертификат. Одно письмо
     «has revoked your certificate» на этом шаге — **ожидаемо**, это плановая
     ротация, а не инцидент.
- **Генерация cert/P12** (работает с любого Linux, ключ не покидает машину):
  ```bash
  openssl genrsa -out ci.key 2048
  openssl req -new -key ci.key -out ci.csr -subj "/CN=Origa CI/O=Origa/C=US"
  # портал: Certificates → Apple Distribution → upload ci.csr → download .cer
  openssl x509 -inform der -in distribution.cer -out ci.pem
  # ВАЖНО: -legacy обязателен — OpenSSL 3.x по умолчанию подписывает PKCS#12
  # MAC'ом SHA-256, который macOS `security import` отвергает с обманчивым
  # «MAC verification failed (wrong password?)». -legacy даёт SHA-1 MAC +
  # RC2/3DES — формат, который принимает keychain.
  openssl pkcs12 -export -legacy -inkey ci.key -in ci.pem -out ci.p12 \
    -passout file:p12.password.txt
  base64 -w0 ci.p12   # → APPLE_IOS_CERTIFICATE
  ```
  Рабочие материалы ротации живут на машине владельца в
  `~/apple-certs-adr056/` (ключ 600, пароль P12 в `p12.password.txt`).
- **Первичная настройка** (однократно): до неё CI-прогон iOS-джобы падает на
  префлайте «manual signing secrets present» — это fail-loud по дизайну;
  секреты должны существовать до мерджа этого ADR в master.
- Откат — `git revert` одного коммита; состояние портала не меняется.

## Валидация

Прогон `gh workflow run tauri.yml -f force_apple=true --ref <branch>`:
(a) гейт «Verify iOS entitlements» зелёный; (b) upload либо успешен, либо
отклонён как duplicate version (артефакт .ipa + зелёный гейт = фикс
подтверждён; duplicate — не регресс); (c) после прогона нет письма о ревокации;
(d) билд в ASC проходит processing без 90721 (асинхронно, проверяется позже).
