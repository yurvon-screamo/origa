# Origa Translator — Runbook (как всем этим пользоваться)

Живой документ практики: запуск стека, генерация локалей, скан/чистка прод-данных,
деплой, окружение и его грабли. Все рецепты проверены в бою (сентябрь–октябрь 2026).
Компоненты и метрики — в [README.md](../README.md).

## 1. Окружение (venv `/home/yurvon/vllm-xpu`)

Канон (проверен 2026-10-20, сборка «v2» с нуля):

```
torch 2.16.0.dev20261019+xpu   # nightly-xpu index, все компоненты ОДНОЙ датой
torchvision 0.30.0.dev20261019+xpu
torchaudio  2.11.0.dev20261016+xpu
vllm 0.32.0                    # PyPI, ставить --no-deps (deps — отдельным списком без torch*)
triton-xpu 3.9.0               # PyPI; ПЕРЕЗАПИСЫВАЕТ base-triton — ставить ПОСЛЕДНИМ, base не накатывать сверху
vllm-xpu-kernels               # PyPI, нужен 0.32 на XPU
transformers 5.x (5.19–5.20)
oneccl 2022 (не работает с torch 2.16 → патч gloo, см. ниже)
```

Патчи venv (одноразовые, теряются при переустановке пакета):

```bash
# xccl бит на этой паре версий; для single-GPU достаточно gloo:
sed -i 's/    dist_backend: str = "xccl"  # xccl only/    dist_backend: str = "gloo"/' \
  $VENV/lib/python3.12/site-packages/vllm/platforms/xpu.py
sed -i 's/            dist_backend = "xccl"/            dist_backend = "gloo"/' \
  $VENV/lib/python3.12/site-packages/vllm/platforms/__init__.py
```

## 2. Переводчик (vLLM serve)

```bash
vllm serve google/gemma-4-E2B-it-qat-w4a16-ct \
  --enable-lora --lora-modules tr600k=/home/yurvon/origa-translator/model/ep1 \
  --max-lora-rank 16 --port 8091 \
  --gpu-memory-utilization 0.42 --max-model-len 1024 --max-num-seqs 16 \
  --enforce-eager
```

Обязательные флаги/факты XPU:

- **`--enforce-eager` ОБЯЗАТЕЛЕН** на iGPU (B390/Panther Lake): compiled-путь
  (inductor, с графами и без) даёт детерминированный многоязычный бред
  («чь संभावित以来…»). Eager чист. dGPU (Arc Pro) — другое дело (см. vllm#48327).
- **LoRA-имя ≠ served-base-имя** в 0.32: базу НЕ именовать (`--served-model-name`
  не ставить), модель запроса = `tr600k`.
- Два движка (переводчик+судья) — стартовать строго последовательно.
- KV-бюджет: при живом десктопе util 0.42 работает при ≥11GB свободных;
  «No available memory for cache blocks» → снизить max-model-len (фразам хватает 768).
- Скорость: eager ~5.3 фраз/с (перевод), ~7.5 слов/с (глоссы), ~8.8 примеров/с.

Промпт (обученный формат):

```
system: You are a skilled Japanese translator. Translate the sentence accurately and naturally.
user:   Translate the following Japanese sentence into {Chinese (Simplified)|English|…}:\n{JP}
temperature=0, max_tokens=200-250
```

Тонкость zh: инструкция «Chinese (Simplified)» даёт мандарин+упрощёнку.
«Traditional Chinese (Taiwan, 繁體中文)» генеративно НЕстабильна (микс-скрипт в 42%
реплик) — Тайвань делать только OpenCC (раздел 4).

## 3. Судья (vLLM pooling + головы)

```bash
vllm serve intfloat/multilingual-e5-base --served-model-name e5b \
  --runner pooling --port 8090 --gpu-memory-utilization 0.30 \
  --attention-backend TRITON_ATTN \
  --pooler-config '{"pooling_type":"MEAN"}'   # mean обязателен (дефолт LAST)
```

- Головы: `model/judge_heads_v3.pt` (InfoNCE, train_v11). Пул клиентов 32, батчи 64 → ~750 текстов/с.
- Префиксы e5: state → `query: …`, кандидаты → `passage: …`.
- API типизированных вопросов (noul/choice/score): `scripts/judge_api.py`.
- Абсолютные скоры не калиброваны между локалями — только ранжирование
  (Best-of-N) и перцентиль-внутри-локали. Сканы прода: двойное подтверждение
  (перцентиль × алгоритмический флаг), хвост скора ≠ брак.

## 4. Китайские локали (zh-Hans + zh-Hant-TW)

Артефакты в `data/pipeline/`:

| Что | Файл | Объём | Качество |
|---|---|---|---|
| zh-Hans фразы | `zh/zh_translations.jsonl` | 156 228 | 99.98% (после саб-чистки 793) |
| zh-Hant-TW фразы | `tw/tw_translations.jsonl` | 156 228 | ~100% |
| zh-Hans слова | `zh_words/zh_words.jsonl` | 97 452 | 95.6% глосс (пустые = мета-слова) |
| zh-Hant-TW слова | `zh_words/tw_words.jsonl` | 97 452 | 95.6% |
| zh примеры | `zh_examples/ex_zh.jsonl` | 22 413 | 99.63% |
| tw примеры | `zh_examples/tw_examples.jsonl` | 22 413 | 99.63% |
| кандзи desc_zh | `zh_words/kanji_zh.jsonl` | 2 135 | 100% |

**TW = OpenCC, не генерация.** `pip install opencc-python-reimplemented`:

```python
from opencc import OpenCC
cc = OpenCC('s2twp')   # Simplified -> Traditional + тайваньская лексика (软件→軟體, 文件→檔案)
tw = cc.convert(zh_text)
```

156k фраз конвертируются за 20 сек, 0.0000% упрощённых остатков. Генеративный
TW-путь отвергнут (42% микс-скрипт). Правило: детерминированная конвертация >
генерация, когда цель — вариант того же содержания.

Bulk-скрипты (все resume-safe: атомарный чекпоинт, fsync, reconcile):
`translate_zh_bulk.py`, `translate_words_zh_bulk.py`, `translate_examples_zh_bulk.py`.

Детекторы качества вывода — в тех же скриптах (`detect_zh`): кана/традиционность/
эхо (порог 0.8 общих символов — имена-кандзи 白百合 валидны, не эхо).

## 5. Сканы и чистки прод-данных (конвейер)

Проверенный цикл (отработан на фразах 2 066 и словаре 1 693 фиксов в проде):

```
скан судьёй (scan_phrases_v3 / scan_dictionary_v3, кэш эмбеддингов)
→ двойное подтверждение: перцентиль-в-локали × надёжный алго-флаг
→ сабагент-чистка ВСЕХ подозрений (контракт: фиксы в отдельный файл, прод не трогать)
→ сабагент-ревизия каждой записи (скрипт-эвристики пропускают ~15% дефектов — сэмплы мало)
→ судейский регресс-тест (score(new) vs score(old); фонетические заимствования
   дают ложно-негативный скор — читать note-обоснования, не верить слепо)
→ патчи cdn → deploy_cdn → HTTP-verify боевого CDN
```

deploy_cdn: запуск ТОЛЬКО с корня репо (валидатору нужны относительные пути);
после правок phrases/data пересобирать бандлы `bundle_phrases_data.py` ДО деплоя;
dry-run не скачивает remote-манифест (диф неинформативен).

## 6. Грабли XPU (выстраданное)

- **Зомби-EngineCore**: после убийства vllm-родителя выживает дочерний процесс с
  comm-именем `VLLM::EngineCore` — pgrep по «vllm serv» его не видит, порт свободен,
  выглядит как утечка памяти. Диагноз: `fuser -v /dev/dri/renderD128 | grep VLLM`.
  Лечение: `pkill -9 -f "VLLM::EngineCo"` — память возвращается сразу, ребут не нужен.
- **pkill-самоубийства**: literal «vllm serve»/«VLLM» ниже в той же командной строке
  убивает сам pkill. Bracket-трюк `vll[m]` ОБЯЗАТЕЛЕН, а лучше kill по PID.
- **L0-сироты** (после GPU-hang) — только ребут; отличаются от зомби тем, что
  fuser пуст.
- **cd-феномен**: `cd X && cmd & disown; next` — всё до `;` уходит в фон ВМЕСТЕ с
  cd, `next` выполняется в старом cwd. Фоновые запуски: абсолютные пути логов.
- transformers 5.16: `dtype=` в from_pretrained отвергается XLMRoberta —
  try/except fallback на `.to(bfloat16)` (см. judge_quick.py).
- torchcodec на XPU: `libtorchcodec_image.so` не грузится — просто удалить пакет
  (vllm переживает: try/except → placeholder).

## 7. HF-репо

- Модель переводчика: `yurvon-screamo/origa-translator-ep1` (карточка с рецептами)
- Судья: `yurvon-screamo/origa-judge`
- Загрузка карточек: `HF_API_KEY` из `~/.bashrc`
