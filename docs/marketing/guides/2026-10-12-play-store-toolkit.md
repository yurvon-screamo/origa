# Гайд: ASO-срез Google Play — выдача, плотность, конкуренты

> **Автор:** @origa_marketing_en_bot · 2026-10-12 · По задаче босса («научимся искать в Google Play»).
> Спарка к `2026-10-12-aso-appstore-toolkit.md` (App Store). Origa в закрытом тесте — публичных позиций нет, меряем поле.

Формат: инструмент → как получить данные → что смотреть → куда сохранить.

---

## 0. Два уровня

| Вопрос | Инструмент | Ключ |
|---|---|---|
| Кто ранжируется по сидам, плотность, заголовки конкурентов | **публичная выдача play.google.com** (HTML-парс, без ключа) | не нужен |
| Отзывы нашего приложения, статистика установок | **Play Developer API** (service account) | ключ (владелец: Play Console → Users and Permissions → линковка SA) |

**Кавет про закрытый тест:** приложение в closed testing невидимо в публичном поиске Play — «наши позиции» не существуют до выхода из теста. Метод меряет поле заранее; после релиза те же скрипты меряют уже наши позиции.

## 1. Парсер выдачи (без ключа)

`https://play.google.com/store/search?q=<term>&c=apps&gl=US&hl=en` отдаёт первую страницу выдачи (~30 приложений) одним HTML. Приложения размечены `<a class="Si6A0c" href="/store/apps/details?id=<pkg>">`, рядом спанами: название, разработчик, рейтинг.

Парсер (рабочий, python stdlib):

```python
import re, urllib.parse, urllib.request

UA = "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 Chrome/120.0 Safari/537.36"

def play_search(term, gl="US", hl="en"):
    url = "https://play.google.com/store/search?" + urllib.parse.urlencode(
        {"q": term, "c": "apps", "gl": gl, "hl": hl})
    req = urllib.request.Request(url, headers={"User-Agent": UA})
    html = urllib.request.urlopen(req, timeout=30).read().decode("utf-8", "ignore")
    out, seen = [], set()
    for b in re.split(r'<a class="Si6A0c', html)[1:]:
        m = re.search(r'href="/store/apps/details\?id=([\w.]+)"', b)
        if not m or m.group(1) in seen:
            continue
        seen.add(m.group(1))
        spans = re.findall(r'<span[^>]*>([^<]{2,80})</span>', b[:3000])
        rating = next((s for s in spans if re.match(r'^\d\.\d$', s)), "")
        out.append({"id": m.group(1), "title": spans[0] if spans else "",
                    "dev": spans[1] if len(spans) > 1 else "", "rating": rating})
    return out
```

Гигиена: `sleep` 3–4 с между сидами; класс разметки (`Si6A0c`) — текущий HTML Play, меняется без предупреждения — при нулевой отдаче проверять страницу глазами; глубина — только первая страница (~30 apps), для глубже — пагинация js, не нужна для топ-задач.

## 2. Срез US 2026-10-12 (gl=US, hl=en; Origa в закрытом тесте — позиций нет)

Полный топ-10 по каждому сиду: `marketing-en` репо, `en/research/2026-10-12-play-us-slice-raw.txt`; скрипт — `en/tools/play_us_slice.py`. Сводка плотности и лидеров:

| Сид | Приложений в выдаче | Лидер (рейтинг) | Характер поля |
| --- | --- | --- | --- |
| learn japanese | 30 | Wagotabi (4.9) | головной, renshuu/Todaii/HeyJapan в топе |
| jlpt | 30 | Migii JLPT (4.5) | узкоспециализированный, много мелких |
| japanese flashcards | 30 | Japanese Vocabulary N5-N1 (4.4) | AnkiDroid №2 — честный сосед |
| kanji | 30 | Kanji Study 漢字学習 (4.9) | зрелое поле, 4.5–4.9 норма |
| **japanese ocr** | 30 | Kanji Scanner (—) | **самое слабое поле**: generic OCR-сканеры (Text Fairy, Image to Text ×4) вперемешку с 3 специализированными JP-OCR |
| hiragana | 30 | NihonGo (4.8) | много «retired»/мёртвых приложений в топе |
| learn japanese offline | 30 | Learn Japanese offline (4.0) | низкорейтинговые лидеры (4.0–4.5) — поле слабое |
| anki alternative | 10 | AnkiDroid (4.8) | выдача узкая: Anki-клоны, не JP |
| japanese reading | 29 | Japanese Reading (4.1) | нишевые ридеры, рейтинги малые |
| study japanese | 30 | renshuu (4.9) | дублирует головной |
| japanese dictionary | 30 | Takoboto (4.7) | словари, устойчивое поле |

## 3. Что смотреть (выводы среза)

1. **OCR-поле в Play — самое слабое из всех сидов**: только 3 специализированных JP-OCR (Kanji Scanner, Mieru, KanjiSnap), остальное — generic-сканеры текста с рейтингом 3.7–4.5. Тот же вывод, что по App Store US: capture-to-SRS с on-device OCR — свободная позиция.
2. **«learn japanese offline»** — лидеры с рейтингом 4.0–4.5 при живом спросе: слабое поле, наш офлайн-first ложится напрямую.
3. **Anki-мост**: AnkiDroid — №2 в «japanese flashcards» и №1 в «anki alternative»; наш Anki-импорт — прямая релевантность этому кластеру.
4. **JLPT-кластер**: Migii №1 (и в Play, и в App Store VN) — тот же конкурент, что в iOS-срезе.
5. Кластеры «study japanese»/«learn japanese» дублируют друг друга — один набор контендеров, сиды схлопывать в один.

## 4. Куда сохранить

Срезы → `marketing/<locale>/research/` + формальные — `docs/marketing/research/` (конвенция ASO-серии); скрипт — в гайде выше, живая копия — `marketing/en/tools/`.

## 5. Ритм

1. Поле мерить раз в месяц до выхода из закрытого теста (дёшево, без ключей).
2. После публичного релиза в Play — внеочередной срез позиций Origa по тем же сидам.
3. Play Developer API (после линковки SA владельцем) — отзывы и установки; показов по источникам в Play API нет — только консоль/CSV.
