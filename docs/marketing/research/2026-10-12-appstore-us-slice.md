# App Store US-срез: позиции Origa, витрина США

> **Автор:** @origa_marketing_en_bot · 2026-10-12 · Метод: iTunes Search API без ключа (`docs/marketing/guides/2026-10-12-aso-appstore-toolkit.md` §1), limit 200, sleep 2–3 с.
> Приложение: «Origa - JLPT Learn Japanese», trackId 6795012532. На дату среза: **0 оценок** (userRatingCount 0).

## Позиции Origa (US, топ-200)

| Запрос | Позиция | Выдача |
| --- | --- | --- |
| origa (бренд) | **3** / 117 | топ заглушен origami-приложениями (у лидера 2 031 оценка) |
| japanese | – | 193 |
| learn japanese | – | 189 |
| jlpt | – | 185 |
| japanese flashcards | – | 175 |
| kanji | – | 188 |
| japanese ocr | – | 187 |
| hiragana | – | 169 |
| learn japanese offline | – | 186 |
| anki alternative | – | 176 |
| japanese dictionary | – | 179 |
| japanese reading | – | 175 |
| study japanese | – | 189 |

13 из 13 контентных сидов — вне топ-200; единственная видимость — брендовая, и та с шумом от «origami».

## Конкурентная плотность топов (имя / разработчик / оценки)

- `japanese dictionary`: Shirabe Jisho (8 811), Japanese (3 595), Takoboto (56)
- `japanese reading`: Yomu Yomu (187), Satori Reader (218), Shinobi (838) — класс мелких нишевых, не гигантов
- `study japanese`: Japanese!! (22 421), renshuu (1 426), Todaii (1 551)
- Головной `learn japanese` (из гайда §1): MARU (18 341), Japanese!! (22 421), Teuida (58 811), LingoDeer (27 993), Busuu (101 187), Duolingo (5 501 388)

Ниша «reading/dictionary» на порядок слабее по оценкам, чем головные термины — вход через long-tail реализуем и без массового рейтинга, при условии его появления.

## Чтение

1. **0 оценок — блокер №1.** Apple сильно взвешивает velocity/объём рейтинга; без стартового пула метаданные не работают (совпадает с выводами KO #724 и VN #725 срезов). Первый рычаг — встроенный rating prompt по таймингу после успешной сессии (продуктовая задача).
2. **Брендовая выдача занята «origami»** — это же имя из KR-среза (ранг 3) и VN (ранг 45). Дёшево чинится подзаголовком: брендовый термин «Origa» стоит подкрепить в подзаголовке («Origa – Learn Japanese, JLPT»), чтобы точное имя+интенция ранжировались первой строкой.
3. **Long-tail «japanese reading»/«dictionary»** — кандидаты №1 для keyword-поля, когда база оценок появится: ниша живая (Shirabe 8.8k, Satori 218), а наш capture-to-SRS + OCR — прямая дифференциация против словарей без тренировки.
4. Головные термины (learn japanese, japanese) — не тратить попытки: там Duolingo/Busuu-класс.

## Действия

- Продукт: rating prompt (SKStoreReviewController после успешной сессии) — парковано lead'ом.
- ASC-метаданные (владелец): подзаголовок с брендом; keyword-поле после появления оценок — под «reading, dictionary, jlpt».
- Повтор среза: раз в месяц + после первых 20–50 оценок (гайд §4).
