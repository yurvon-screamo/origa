# Медиа-ассеты Origa

Карта зон репозитория: где что лежит, кто потребляет, правила для новых
файлов. Карта машинно проверяется тестом `scripts/tests/test_media_map.py` —
зона без потребителя или путь, которого нет, валят сборку.

Два реестра делят медиа между собой:

| Реестр | Что хранит | Кто пишет |
| --- | --- | --- |
| `docs/ui-screenshots/shots/*.json` | скрины интерфейса (ADR-067) | `end2end/shots` (`npm run shots -- capture --upload`) |
| `docs/media/store-assets.json` | магазинные и маркетинговые ассеты (ADR-068) | `scripts/publish_media.py` |

## Зоны репозитория

| Путь | Содержимое | В git | Потребитель |
| --- | --- | --- | --- |
| `origa_landing/public/images/` | картинки лендинга (26 файлов) | да | `origa_landing/src`, корневые README; тест `origa_landing/tests/assets.rs` |
| `origa_landing/public/og-image.png`, `origa_landing/public/favicon.png`, `origa_landing/public/favicon.ico`, `origa_landing/public/apple-touch-icon.png` | og:image и иконки сайта | да | `origa_landing/src/components/seo.rs`, `origa_landing/src/app.rs` |
| `origa_landing/public/badges/` | бейджи сторов (App Store, RuStore), SVG | да | корневые `README.md`; `origa_landing/src/pages/download.rs` |
| `origa_landing/public/jlpt/` | PDF-брошюры JLPT, 20 файлов | да | страницы `/download`; манифест `origa_landing/public/jlpt/MANIFEST.json` |
| `origa_landing/public/fonts/landing/` | шрифты лендинга | да | `origa_landing/style/landing.css` |
| `origa_ui/public/logo*.png` | лого приложения | да | `origa_ui/src/ui_components/logo.rs`, `origa_ui/index.html` |
| `origa_ui/public/external_icons/` | логотипы сторонних приложений (Anki, Duolingo, Irodori, Migii, Minna no Nihongo) | да | `origa_ui/src/pages/onboarding/` — шаг импорта; вне этого контекста не использовать |
| `tauri/icons/`, `tauri/gen/**` | иконки приложений — генерённые | да | `tauri/tauri.conf.json`; руками не править |
| `tauri/msix/Assets/` | иконки MSIX-пакета | да | `tauri/scripts/build-msix.ps1` |
| `origa/src/ocr/ocr_example.jpg` | фикстура OCR-теста | да | `origa/src/ocr/tests.rs` |
| `origa_ui/src/ui_components/kanji_drawing/test_kanji_med_spaced.svg` | фикстура парсера SVG | да | `origa_ui/src/ui_components/kanji_drawing/svg_parser.rs` |
| `end2end/fixtures/standard_sample1.wav` | фикстура e2e и STT-теста | да | `end2end/bdd/steps/words.steps.ts`, `origa/src/stt/tests.rs` |
| `docs/ui-screenshots/` | реестр скринов интерфейса: метаданные + ссылки на CDN | да (только метаданные) | `end2end/shots/`; маркетинг и дизайн — источник актуальных картинок |
| `docs/media/store-assets.json` | реестр магазинных ассетов: ключи, хеши, происхождение | да | `scripts/publish_media.py` |
| `store-assets/` | локальный staging для переснятых ассетов | **нет** | `scripts/publish_media.py --source store-assets/...` |
| CDN `media/**` | сами магазинные ассеты (41 файл) | **нет** | листинги App Store (EN, RU, iPad), Play Console, соцсети; скачивается по ссылке из реестра |
| `cdn/` | контент для CDN-деплоя: словари, грамматика, фразы, модели | **нет** | `scripts/deploy_cdn.py` |

CDN-префиксы магазинных ассетов (ADR-068): `media/app-store-iphone-en/`,
`media/app-store-iphone-ru/`, `media/app-store-ipad/`,
`media/store-creative-en/`, `media/store-creative-ru/`,
`media/play-feature-graphic/`, `media/social-poster/`,
`media/marketing-visual/`.

## Правила

1. Скрин интерфейса снимает `end2end/shots` и льёт в CDN, не в git (ADR-067).
2. Новый магазинный ассет кладётся в `store-assets/` и публикуется скриптом
   `scripts/publish_media.py` в реестр (ADR-068).
3. Новая картинка лендинга добавляется в allowlist
   `origa_landing/tests/assets.rs` — иначе тест её отвергнет как сироту.
4. Картинки лендинга и магазинные ассеты между зонами не дублируются.
5. `tauri/icons*`, `tauri/gen/**`, `tauri/msix/Assets/` — генерённые. Ручная
   правка теряется при следующей регенерации.
6. `origa_ui/public/external_icons/` — товарные знаки сторонних продуктов.
   Вне контекста шага импорта не использовать.

## Provenance

- 2026-10: `.media/` (91 файл, 90 MiB), `docs/screenshots/` (30, 11.6 MiB) и
  `docs/marketing/assets/` (4, 2.4 MiB) удалены; магазинные ассеты
  опубликованы в CDN и описаны в `docs/media/store-assets.json` (ADR-068).
  Источник восстановления — git история.
- 2026-09: из корня `.media/` удалены 10 md5-дублей лендинг-картинок
  (канон — `origa_landing/public/images/`); удалён `.media/content.png`;
  удалён `dns-page.png` из корня.
