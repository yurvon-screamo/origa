import concurrent.futures
import json
import sys
import threading
import time
from datetime import datetime, timedelta
from pathlib import Path

from openai import OpenAI

file_lock = threading.Lock()


def translate_japanese_to_english(word: str, client: OpenAI) -> str:
    user_prompt = f"""<prompt>
  <task>
    You are a professional Japanese-English lexicographer. Your task: provide accurate, minimal, but comprehensive meanings of the Japanese word in English.
  </task>

  <word>
    {word}
  </word>

  <success_brief>
    <format>
      <description>Strictly follow this output format (markdown):</description>
      <template>
<![CDATA[
- Translation 1
- Translation 2
- Translation 3

> Comment (only when absolutely necessary)
]]>
      </template>
    </format>

    <quality_criteria>
      <criterion name="minimalism">Do not duplicate meanings (avoid: "evening/in the evening/evening (adj)")</criterion>
      <criterion name="different meanings">Each meaning must be semantically distinct</criterion>
      <criterion name="language">ONLY English text (no Japanese: kanji, kana, romaji, readings)</criterion>
      <criterion name="structure">Bulleted list + optional blockquote for comments</criterion>
      <criterion name="volume">1-5 meanings maximum (for polysemous words), 1-2 for monosemous words</criterion>
    </quality_criteria>
  </success_brief>

  <rules>
    <rule id="1">DO NOT duplicate grammatical forms of the same word (noun ≠ verb from the same root if meaning is the same)</rule>
    <rule id="2">DO NOT add Japanese text in the answer (no readings, kanji, examples in Japanese)</rule>
    <rule id="3">DO NOT write introduction, conclusion, explanations before the list (start with the list immediately)</rule>
    <rule id="4">Comment in blockquote — only if meaning is not obvious or context clarification is needed</rule>
    <rule id="5">Group semantically close meanings into one meaning</rule>
    <rule id="6">If word has homonyms — indicate only main meanings, not all possible ones</rule>
    <rule id="7">Priority: frequent meanings → rare meanings</rule>
  </rules>

  <examples>
    <good>
      <word>かける (kakeru)</word>
      <output>
<![CDATA[
- To hang
- To spend (time, money)
- To make a phone call
- To put on (glasses, insurance)
]]>
      </output>
      <reason>4 different semantic meanings of one verb</reason>
    </good>

    <good>
      <word>重い (omoi)</word>
      <output>
<![CDATA[
- Heavy
- Serious (about a mistake, illness)

> Depending on context: physical weight or degree of importance
]]>
      </output>
      <reason>2 meanings + comment for clarification</reason>
    </good>

    <good>
      <word>冷蔵庫 (reizouko)</word>
      <output>
<![CDATA[
- Refrigerator
]]>
      </output>
      <reason>One meaning, no need to duplicate</reason>
    </good>

    <bad>
      <word>冷蔵庫 (reizouko)</word>
      <output>
<![CDATA[
- Refrigerator
- Freezer
- Cold
- Cooling
]]>
      </output>
      <reason>Redundant: essentially one device, the rest are not meanings of the word</reason>
    </bad>

    <bad>
      <word>夕べ (yuube)</word>
      <output>
<![CDATA[
- Evening
- In the evening
- Evening (adj)
]]>
      </output>
      <reason>Grammatical forms of one meaning, need one meaning</reason>
    </bad>
  </examples>

  <conversation>
    <instruction>
      Reply only in markdown according to the specified format
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
                time.sleep(1)
                continue
            raise e


def _resolve_file_path(file_path_str: str) -> Path | None:
    if file_path_str.startswith("@"):
        file_path_str = file_path_str[1:]

    file_path = Path(file_path_str).resolve()
    if file_path.exists():
        return file_path

    alt_path = (Path(__file__).parent.parent / file_path_str).resolve()
    if alt_path.exists():
        return alt_path

    print(
        f"[{datetime.now().strftime('%H:%M:%S')}] Error: File not found: {file_path_str}"
    )
    return None


def _process_chunk(chunk: list, data: dict, client: OpenAI, file_path: Path):
    completed = 0
    errors = 0

    with concurrent.futures.ThreadPoolExecutor(max_workers=32) as executor:
        future_to_word = {
            executor.submit(translate_japanese_to_english, word, client): word
            for word in chunk
        }

        for future in concurrent.futures.as_completed(future_to_word):
            word = future_to_word[future]
            try:
                translation = future.result()
                if translation:
                    data[word]["english_translation"] = translation
                    completed += 1
                    with file_lock:
                        with open(file_path, "w", encoding="utf-8") as f:
                            json.dump(data, f, ensure_ascii=False, indent=2)
            except Exception:
                errors += 1

            current = completed + errors
            if current % 50 == 0 or current == len(chunk):
                percent = (current / len(chunk)) * 100
                print(
                    f"  Progress in chunk: {current}/{len(chunk)} ({percent:.1f}%)",
                    end="\r",
                )

    return completed, errors


def process_file(file_path_str: str):
    start_time = time.time()

    file_path = _resolve_file_path(file_path_str)
    if not file_path:
        return

    print(f"[{datetime.now().strftime('%H:%M:%S')}] Loading {file_path}...")
    with open(file_path, "r", encoding="utf-8") as f:
        data = json.load(f)

    words_to_translate = [
        word
        for word, translations in data.items()
        if not translations.get("english_translation")
    ]

    if not words_to_translate:
        print("All words already have English translations.")
        return

    total_words = len(words_to_translate)
    print(
        f"[{datetime.now().strftime('%H:%M:%S')}] Found {total_words} words without English translation. Starting..."
    )

    client = OpenAI(base_url="http://10.2.11.6:8001/v1", api_key="none")

    chunk_size = 512
    completed_total = 0
    errors_total = 0

    for i in range(0, total_words, chunk_size):
        chunk_start_time = time.time()
        chunk = words_to_translate[i : i + chunk_size]
        chunk_num = i // chunk_size + 1
        total_chunks = (total_words + chunk_size - 1) // chunk_size

        print(
            f"\n--- Chunk {chunk_num}/{total_chunks} ({i} to {min(i + chunk_size, total_words)}) ---"
        )

        completed_in_chunk, errors_in_chunk = _process_chunk(
            chunk, data, client, file_path
        )

        completed_total += completed_in_chunk
        errors_total += errors_in_chunk

        chunk_duration = time.time() - chunk_start_time
        avg_time_per_word = chunk_duration / len(chunk) if len(chunk) > 0 else 0
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
                result = translate_japanese_to_english(target, client)
                if result:
                    print(result)
            except Exception as e:
                print(f"Error: {e}")
    else:
        print("Usage: python scripts/translate_english.py <word_or_json_file_path>")
