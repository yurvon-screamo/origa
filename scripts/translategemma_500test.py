"""Test TranslateGemma quality on 500 real phrases from CDN dataset."""
import json
import time
import requests
from pathlib import Path

OLLAMA_URL = "http://localhost:11434/api/generate"
MODEL = "translategemma:4b"

# Load first chunk from CDN
cdn_path = Path("cdn/phrases/data/p0000.json")
if not cdn_path.exists():
    print(f"Error: {cdn_path} not found")
    exit(1)

raw = cdn_path.read_bytes()
text = raw.lstrip(b"\xef\xbb\xbf").decode("utf-8")
phrases = json.loads(text)

# Take first 500
test_phrases = phrases[:500]
print(f"Testing {len(test_phrases)} phrases from {cdn_path.name}")
print()

def translate(text_to_translate, target_lang):
    if target_lang == "en":
        target_name = "English"
    else:
        target_name = "Russian"
    prompt = (
        f"You are a professional Japanese to {target_name} translator. "
        f"Produce only the {target_name} translation, without any additional explanations or commentary.\n"
        f"Please translate the following text from Japanese to {target_name}:\n\n"
        f"{text_to_translate}"
    )
    try:
        resp = requests.post(OLLAMA_URL, json={
            "model": MODEL,
            "prompt": prompt,
            "stream": False,
            "options": {"temperature": 0.0, "num_predict": 200}
        }, timeout=120)
        if resp.status_code == 200:
            return resp.json().get("response", "").strip()
        return f"ERROR: {resp.status_code}"
    except Exception as e:
        return f"ERROR: {e}"

# Stats
errors = 0
empty_en = 0
empty_ru = 0
too_short = 0
samples_ok = []
samples_bad = []

t0 = time.time()

for i, p in enumerate(test_phrases):
    jp = p.get("x", "")
    if not jp:
        continue
    
    en = translate(jp, "en")
    ru = translate(jp, "ru")
    
    # Track quality
    en_ok = en and not en.startswith("ERROR") and len(en) >= 3
    ru_ok = ru and not ru.startswith("ERROR") and len(ru) >= 3
    
    if not en_ok:
        empty_en += 1
    if not ru_ok:
        empty_ru += 1
    if en.startswith("ERROR") or ru.startswith("ERROR"):
        errors += 1
    
    # Collect samples
    if i < 10:
        print(f"[{i+1}] JP: {jp[:60]}")
        print(f"     EN: {en[:80]}")
        print(f"     RU: {ru[:80]}")
        print()
    elif i < 500 and not en_ok or not ru_ok:
        samples_bad.append((jp, en, ru))
    elif i < 500 and en_ok and ru_ok and len(samples_ok) < 10:
        samples_ok.append((jp, en, ru))
    
    # Progress every 50
    if (i + 1) % 50 == 0:
        elapsed = time.time() - t0
        rate = (i + 1) / elapsed
        print(f"  ... {i+1}/500 done ({rate:.1f} phrases/sec, {elapsed:.0f}s elapsed)")

elapsed = time.time() - t0
print(f"\n{'='*60}")
print(f"RESULTS: {len(test_phrases)} phrases in {elapsed:.0f}s ({elapsed/len(test_phrases):.1f}s/phrase)")
print(f"Errors: {errors}")
print(f"Empty/short EN: {empty_en}")
print(f"Empty/short RU: {empty_ru}")
print(f"Success rate EN: {(500-empty_en)/500*100:.1f}%")
print(f"Success rate RU: {(500-empty_ru)/500*100:.1f}%")

if samples_bad:
    print(f"\n--- BAD samples ({len(samples_bad)} found, showing first 5) ---")
    for jp, en, ru in samples_bad[:5]:
        print(f"  JP: {jp[:50]}")
        print(f"  EN: {en[:60]}")
        print(f"  RU: {ru[:60]}")
        print()
