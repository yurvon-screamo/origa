import concurrent.futures
import json
import sys
from pathlib import Path

from openai import OpenAI


def translate_japanese_to_russian(word: str, client: OpenAI) -> str:
    """
    Translates a Japanese word to Russian using a vLLM server with OpenAI-compatible API.
    """
    prompt = """<prompt>
  <role>Лексикограф-японист</role>
  
  <task>Дай точные, минимальные значения японского слова на русском</task>
  
  <format>
<![CDATA[
- Значение 1
- Значение 2

> Комментарий (только если критично)
]]>
  </format>
  
  <rules>
    <rule>ТОЛЬКО русский текст (никакого японского)</rule>
    <rule>1-5 значений максимум</rule>
    <rule>Без дублей (вечер ≠ вечером ≠ вечерний)</rule>
    <rule>Без вступлений и заключений</rule>
    <rule>Комментарий — только при необходимости</rule>
  </rules>
  
  <examples>
    <good word="かける">
      - Вешать
      - Тратить
      - Звонить
    </good>
    
    <good word="冷蔵庫">
      - Холодильник
    </good>
    
    <bad word="夕べ">
      - Вечер
      - Вечером
      - Вечерний
    </bad>
  </examples>
  
  <instruction>Сразу выдавай результат в формате. Жди слово от пользователя.</instruction>
</prompt>"""

    try:
        response = client.chat.completions.create(
            model="llm",
            messages=[
                {
                    "role": "system",
                    "content": "Ты профессиональный переводчик с японского на русский. Переводи максимально точно и кратко.",
                },
                {"role": "user", "content": prompt},
            ],
            temperature=0.0,
        )
        return response.choices[0].message.content.strip()
    except Exception as e:
        print(f"Error translating '{word}': {str(e)}")
        return None


def process_file(file_path_str: str):
    # Normalize path (handle @ prefix and resolve path relative to workspace root)
    if file_path_str.startswith("@"):
        file_path_str = file_path_str[1:]

    file_path = Path(file_path_str).resolve()
    if not file_path.exists():
        # Try relative to script location if absolute or relative to root fails
        alt_path = (Path(__file__).parent.parent / file_path_str).resolve()
        if alt_path.exists():
            file_path = alt_path
        else:
            print(f"File not found: {file_path_str}")
            return

    print(f"Loading {file_path}...")
    with open(file_path, "r", encoding="utf-8") as f:
        data = json.load(f)

    words = list(data.keys())
    total_words = len(words)
    print(
        f"Found {total_words} words. Starting translation in parallel (chunk size 512)..."
    )

    client = OpenAI(base_url="http://10.2.11.6:8001/v1", api_key="none")

    # Use ThreadPoolExecutor for parallel requests
    # Processing in batches of 512 as requested
    chunk_size = 512
    for i in range(0, total_words, chunk_size):
        chunk = words[i : i + chunk_size]
        print(
            f"Processing chunk {i // chunk_size + 1} ({i} to {min(i + chunk_size, total_words)})..."
        )

        # Parallelize within each chunk
        # max_workers=20 is a safe bet for a local vLLM, can be increased if needed
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
                except Exception as e:
                    print(f"Exception for word '{word}': {e}")

        # Save after each chunk to avoid losing progress
        print(f"Saving progress after chunk {i // chunk_size + 1} to {file_path}...")
        with open(file_path, "w", encoding="utf-8") as f:
            json.dump(data, f, ensure_ascii=False, indent=2)

    print(f"Finished processing {file_path}.")


if __name__ == "__main__":
    if len(sys.argv) > 1:
        target = sys.argv[1]
        if target.endswith(".json"):
            process_file(target)
        else:
            # Single word mode
            client = OpenAI(base_url="http://10.2.11.6:8001/v1", api_key="none")
            result = translate_japanese_to_russian(target, client)
            if result:
                print(result)
    else:
        print("Usage: python scripts/translate.py <word_or_json_file_path>")
