# Origa Translator

[English](README.md) | [Русский](README.ru.md) | [한국어](README.ko.md) | [Tiếng Việt](README.vi.md)

Стек перевода для контент-пайплайна Origa: переводчик JA→{EN,RU,KO,VI} и судья качества.

Здесь код и набор для оценки качества. Веса моделей лежат на Hugging Face; обучающие корпуса внутренние и не публикуются.

## Компоненты

**Переводчик — `ep1`** ([HF: origa-translator-ep1](https://huggingface.co/yurvon-screamo/origa-translator-ep1))
LoRA-адаптер (r=16 α=32, 24.16M обучаемых параметров, 205 модулей текстового стека) поверх `google/gemma-4-E2B-it-qat-w4a16-ct`. Обучен 1 эпоху на 600k парах фраз (150k × 4 локали), val_loss 0.6720.

**Судья** ([HF: origa-judge](https://huggingface.co/yurvon-screamo/origa-judge))
`intfloat/multilingual-e5-base` (mean-pooling) + InfoNCE-головы. Обучен на 59 973 парах. Внутренняя оценка: 81.9% на golden_v4c (662 пары, 7 итераций аудита). Судья ранжирует кандидатов (Best-of-N); абсолютные скоры не откалиброваны по локалям — см. [issue #713](https://github.com/yurvon-screamo/origa/issues/713).

## Цифры качества

Все числа ниже — *внутренние оценки*: ручная проверка автором проекта, не независимый бенчмарк. Считайте их ориентировочными.

| Проверка | Результат |
| --- | --- |
| Ручная проверка, 200 фраз (50 × 4 локали) | 83% полных / 99.5% пригодных |
| Роутинг языков, 200 фраз | 200/200 с первой попытки |
| Zero-shot локали вне обучения (my/bm/th), 24 фразы | 24/24 пригодных (базовая модель без адаптера: 0/24) |
| Судья на golden_v4c | 542/662 (81.9%) |

Разбивка по локалям и таксономия ошибок: [docs/REPORT_AB_6MODELS.md](docs/REPORT_AB_6MODELS.md). Сравнения chrF по 6 моделям-кандидатам — в том же отчёте; финальный стек выбран вместо всех них.

## Быстрый старт

Поднять адаптер через vLLM:

```bash
vllm serve google/gemma-4-E2B-it-qat-w4a16-ct \
  --enable-lora --lora-modules tr600k=<path-to-ep1> \
  --max-lora-rank 16
```

Или загрузить через transformers (на QAT-базе важны детали, см. грабли):

```python
import torch
from transformers import AutoModelForCausalLM, AutoTokenizer, CompressedTensorsConfig
from peft import PeftModel

BASE = "google/gemma-4-E2B-it-qat-w4a16-ct"
model = AutoModelForCausalLM.from_pretrained(
    BASE, dtype=torch.bfloat16, low_cpu_mem_usage=False,
    quantization_config=CompressedTensorsConfig(run_compressed=False),
).to("xpu")
model = PeftModel.from_pretrained(model, "yurvon-screamo/origa-translator-ep1").eval()
tok = AutoTokenizer.from_pretrained(BASE)
```

Без `low_cpu_mem_usage=False` + `CompressedTensorsConfig(run_compressed=False)` слои загружаются как заглушки без упакованных весов, и PEFT падает.

Инференс судьи: поднять `intfloat/multilingual-e5-base` через vLLM pooling с `--pooler-config '{"pooling_type":"MEAN"}'` (дефолтный LAST pooling ломает совместимость с головами) и `--attention-backend TRITON_ATTN`. Вызывать через `scripts/judge_api.py` — типизированный API вопросов (noul/choice/score) поверх голов.

## Скрипты

| Скрипт | Что делает |
| --- | --- |
| `build_datasets.py` | Собирает SFT-пулы из исходных данных |
| `train_sft_qat.py` | Тренер QAT-native LoRA (рецепт ниже) |
| `run_600k.sh` | Полный прогон 600k с чекпоинтами и авто-resume |
| `eval_wrapper.py` | Одиночная генерация + детект языка + retry |
| `eval_model.py` / `bench_train_qat.py` | Оценка модели и бенчмарк обучения |
| `judge_api.py` | Типизированный API вопросов поверх голов судьи |
| `train_judge_heads.py` | Тренер голов судьи (потребляет vLLM-эмбеддинги) |
| `build_judge_v3.py` | Сборка датасета судьи (v3) |
| `pipeline_translate_judge.py` | Сквозной: перевод → проверки → судья → маржа |
| `scan_phrases_v3.py` / `scan_dictionary_v3.py` | Продакшн-сканы фраз и словаря CDN |

Скрипты читают пути из переменных окружения (все опциональны, дефолты относительны репозиторию):

| Переменная | Назначение |
| --- | --- |
| `ORIGA_DATA_DIR` | Корень обучающих данных |
| `ORIGA_RUNS_DIR`, `ORIGA_WORK_DIR` | Чекпоинты и рабочее пространство |
| `ORIGA_JUDGE_HEADS` | Путь к `judge_heads_v3.pt` |
| `ORIGA_JUDGE_DATA` | Датасеты судьи |
| `ORIGA_PIPELINE_OUT` | Вывод конвейера чистки |
| `ORIGA_CDN_PHRASES`, `ORIGA_CDN_DICTIONARY` | Деревья боевого CDN-контента |

## Рецепт обучения (QAT-база)

1. Загрузите QAT-базу с `CompressedTensorsConfig(run_compressed=False)` (алиас `dequantize=True`) + `low_cpu_mem_usage=False`. Это деквантизует упакованный int4 в настоящие bf16 `nn.Linear`-слои с исходными QAT-весами. Не наслаивайте bitsandbytes сверху.
2. Соберите LoRA-таргеты из реального дерева модулей (`Linear` с ненулевым `weight`, только текстовый стек — без vision/audio/мультимодальности): на этой архитектуре 205 модулей. `target_modules="all-linear"` падает на кастомных clippable-слоях.
3. QAT-native обучение в 2.4× быстрее загрузки базы через bitsandbytes-NF4 и в наших прогонах даёт то же качество.
4. Батч-генерация с left-padding на Intel XPU искажает вывод; все финальные оценки шли одиночными запросами (batch=1). Батчинг vLLM безопасен.

Полный разбор, включая инфраструктурные фиксы (OOM-клетка cgroup, masked CE для словаря на 262k токенов, фрагментация XPU-аллокатора), — в [docs/REPORT_AB_6MODELS.md](docs/REPORT_AB_6MODELS.md).

## Известные ограничения

- Судья ранжирует, но не ставит оценку. Абсолютные пороги не откалиброваны по локалям (доказательства по KO и VI в [#713](https://github.com/yurvon-screamo/origa/issues/713)). Поддерживаемый режим использования — Best-of-N.
- `pipeline_translate_judge.py` по умолчанию всё ещё указывает на головы судьи v2 (исторически); отдельные сканеры (`scan_phrases_v3.py`, `scan_dictionary_v3.py`) используют v3. Это осознанно, описано как есть, унификации пока нет.
- Судья даёт ложно-негативы на фонетических заимствованиях; поэтому продакшн-сканы требуют правила двойного подтверждения (перцентиль × алгоритмический флаг), а не голого порога по скору.
- У переводчика остаются остаточные 🟡 на VI (сдвиги регистра/вежливости) и редкие инверсии в EN. Мы считаем это пределом способности базы 2B; данные проблему не сняли.
- Обучение — только на Intel XPU; инференс работает везде, где vLLM поддерживает базу.

## Лицензия

- Код в этой директории: BSL 1.1, как и весь репозиторий. См. корневой [LICENSE](../LICENSE).
- Базовая модель `google/gemma-4-E2B-it-qat-w4a16-ct`: Apache 2.0 (по её карточке на Hugging Face).
- Обучающие корпуса: производные от продуктового контента Origa, внутренние и не публикуются.
