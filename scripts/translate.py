import concurrent.futures
import json
import sys
import threading
import time
from datetime import datetime, timedelta
from pathlib import Path

from openai import OpenAI

# Global lock for file saving to avoid concurrent writes
file_lock = threading.Lock()


def translate_japanese_to_russian(word: str, client: OpenAI) -> str:
    """
    Translates a Japanese word to Russian using a vLLM server with OpenAI-compatible API.
    """

    user_prompt = f"""<prompt>
  <task>
    Ты — профессиональный лексикограф-японист. Твоя задача: дать точные, минимальные, но исчерпывающие значения японского слова на русском языке.
  </task>

  <word>
    {word}
  </word>

  <success_brief>
    <format>
      <description>Строго соблюдай следующий формат вывода (markdown):</description>
      <template>
<![CDATA[
- Перевод 1
- Перевод 2
- Перевод 3

> Комментарий (только при острой необходимости)
]]>
      </template>
    </format>

    <quality_criteria>
      <criterion name="минимализм">Не дублируй смыслы (избегай: "вечер/вечером/вечерний", "стирать/стирка")</criterion>
      <criterion name="разные значения">Каждое значение должно быть семантически отличным от других</criterion>
      <criterion name="язык">ТОЛЬКО русский текст (никакого японского: кандзи, каны, ромадзи, прочтений)</criterion>
      <criterion name="структура">Маркированный список + опционально блок цитаты для комментария</criterion>
      <criterion name="объём">1-5 значений максимум (для многозначных слов), 1-2 для однозначных</criterion>
    </quality_criteria>
  </success_brief>

  <rules>
    <rule id="1">НЕ дублируй грамматические формы одного слова (существительное ≠ глагол от того же корня, если смысл тот же)</rule>
    <rule id="2">НЕ добавляй японский текст в ответ (никаких прочтений, кандзи, примеров на японском)</rule>
    <rule id="3">НЕ пиши вступление, заключение, пояснения перед списком (сразу начинай со списка)</rule>
    <rule id="4">Комментарий в блоке цитаты — только если значение неочевидно или требуется уточнение контекста</rule>
    <rule id="5">Группируй близкие семантически смыслы в одно значение</rule>
    <rule id="6">Если слово имеет омонимы — указывай только основные значения, не все возможные</rule>
    <rule id="7">Приоритет: частотные значения → редкие значения</rule>
  </rules>

  <examples>
    <good>
      <word>かける (kakeru)</word>
      <output>
<![CDATA[
- Вешать
- Тратить (время, деньги)
- Звонить (по телефону)
- Надевать (очки, страховку)
]]>
      </output>
      <reason>4 разных семантических значения одного глагола</reason>
    </good>

    <good>
      <word>重い (omoi)</word>
      <output>
<![CDATA[
- Тяжёлый
- Серьёзный (об ошибке, болезни)

> В зависимости от контекста: физический вес или степень важности
]]>
      </output>
      <reason>2 значения + комментарий для уточнения</reason>
    </good>

    <good>
      <word>冷蔵庫 (reizouko)</word>
      <output>
<![CDATA[
- Холодильник
]]>
      </output>
      <reason>Одно значение, не нужно дублировать</reason>
    </good>

    <bad>
      <word>冷蔵庫 (reizouko)</word>
      <output>
<![CDATA[
- Холодильник
- Морозильная камера
- Холод
- Охлаждение
]]>
      </output>
      <reason>Избыточно: по сути одно устройство, остальные — не значения слова</reason>
    </bad>

    <bad>
      <word>夕べ (yuube)</word>
      <output>
<![CDATA[
- Вечер
- Вечером
- Вечерний
]]>
      </output>
      <reason>Грамматические формы одного смысла, нужно одно значение</reason>
    </bad>
  </examples>

  <conversation>
    <instruction>
      Отвечай только markdown по заданному формату
    </instruction>
  </conversation>
</prompt>"""

    max_retries = 3
    for attempt in range(max_retries):
        try:
            response = client.chat.completions.create(
                model="llm",
                messages=[
                    {"role": "user", "content": user_prompt},
                ],
                max_tokens=12144,
                temperature=0.7,
                top_p=0.8,
                presence_penalty=1.5,
                extra_body={
                    "top_k": 20,
                    "chat_template_kwargs": {"enable_thinking": False},
                },
            )
            return response.choices[0].message.content.strip()
        except Exception as e:
            if attempt < max_retries - 1:
                time.sleep(1)  # Brief pause before retry
                continue
            raise e


def process_file(file_path_str: str):
    start_time = time.time()

    # Normalize path
    if file_path_str.startswith("@"):
        file_path_str = file_path_str[1:]

    file_path = Path(file_path_str).resolve()
    if not file_path.exists():
        alt_path = (Path(__file__).parent.parent / file_path_str).resolve()
        if alt_path.exists():
            file_path = alt_path
        else:
            print(
                f"[{datetime.now().strftime('%H:%M:%S')}] Error: File not found: {file_path_str}"
            )
            return

    print(f"[{datetime.now().strftime('%H:%M:%S')}] Loading {file_path}...")
    with open(file_path, "r", encoding="utf-8") as f:
        data = json.load(f)

    words = list(data.keys())
    total_words = len(words)
    print(
        f"[{datetime.now().strftime('%H:%M:%S')}] Found {total_words} words. Starting processing..."
    )

    client = OpenAI(base_url="http://10.2.11.6:8001/v1", api_key="none")

    chunk_size = 512
    completed_total = 0
    errors_total = 0

    for i in range(0, total_words, chunk_size):
        chunk_start_time = time.time()
        chunk = words[i : i + chunk_size]
        chunk_num = i // chunk_size + 1
        total_chunks = (total_words + chunk_size - 1) // chunk_size

        print(
            f"\n--- Chunk {chunk_num}/{total_chunks} ({i} to {min(i + chunk_size, total_words)}) ---"
        )

        completed_in_chunk = 0
        errors_in_chunk = 0

        with concurrent.futures.ThreadPoolExecutor(max_workers=32) as executor:
            future_to_word = {
                executor.submit(translate_japanese_to_russian, word, client): word
                for word in chunk
            }

            for future in concurrent.futures.as_completed(future_to_word):
                word = future_to_word[future]
                try:
                    translation = future.result()
                    if translation:
                        data[word]["russian_translation"] = translation
                        completed_in_chunk += 1

                        # Save immediately on each word translation to avoid losing progress
                        with file_lock:
                            with open(file_path, "w", encoding="utf-8") as f:
                                json.dump(data, f, ensure_ascii=False, indent=2)
                except Exception:
                    errors_in_chunk += 1
                    # print(f"Error for '{word}': {e}") # Silent error to keep logs clean

                # Periodic progress update within chunk
                current_completed = completed_in_chunk + errors_in_chunk
                if current_completed % 50 == 0 or current_completed == len(chunk):
                    percent = (current_completed / len(chunk)) * 100
                    print(
                        f"  Progress in chunk: {current_completed}/{len(chunk)} ({percent:.1f}%)",
                        end="\r",
                    )

        # Stats update
        completed_total += completed_in_chunk
        errors_total += errors_in_chunk

        chunk_duration = time.time() - chunk_start_time
        avg_time_per_word = chunk_duration / len(chunk) if len(chunk) > 0 else 0

        # Calculate ETA
        remaining_words = total_words - (i + len(chunk))
        eta_seconds = remaining_words * avg_time_per_word
        eta_str = str(timedelta(seconds=int(eta_seconds)))

        print(
            f"\n  Chunk finished in {chunk_duration:.2f}s (Avg: {avg_time_per_word:.3f}s/word)"
        )
        print(
            f"  Total progress: {min(i + chunk_size, total_words)}/{total_words} ({(min(i + chunk_size, total_words) / total_words) * 100:.1f}%)"
        )
        print(f"  Errors so far: {errors_total}")
        print(f"  Estimated time remaining: {eta_str}")

    total_duration = time.time() - start_time
    print(f"\n{'=' * 50}")
    print(f"FINISHED processing {file_path}")
    print(f"Total time: {str(timedelta(seconds=int(total_duration)))}")
    print(f"Words processed: {completed_total}")
    print(f"Errors: {errors_total}")
    print(f"{'=' * 50}")


if __name__ == "__main__":
    if len(sys.argv) > 1:
        target = sys.argv[1]
        if target.endswith(".json"):
            process_file(target)
        else:
            client = OpenAI(base_url="http://10.2.11.6:8001/v1", api_key="none")
            try:
                result = translate_japanese_to_russian(target, client)
                if result:
                    print(result)
            except Exception as e:
                print(f"Error: {e}")
    else:
        print("Usage: python scripts/translate.py <word_or_json_file_path>")
