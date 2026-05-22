import requests
import time

OLLAMA_URL = "http://localhost:11434/api/generate"
MODEL = "translategemma:4b"

phrases = [
    "残念ながら普通の焼酎しかないのよ",
    "負けてられませんっ…アンジェがピンチです",
    "今年の夏は！海！",
    "分かんないの、分かんないんだよ",
    "豆腐屋の次男坊だ",
    "あいつに惚れてるのよ",
    "幸いなことに隼斗は人間じゃ",
    "なぜならデートは妹とのみ許可される行為だからです",
    "思い切って彼女を受け入れるのもキミが真相に辿り着く１つの方法かもしれない",
    "はい、朝から終わりまで",
    "うるさいわねっ！",
    "こ、こら…見るでない！",
    "そう考えると辻褄が合う気がするの",
    "まさか、ワタシに見せつけるようにして、一人で食べる気ですか？",
    "新田剣丞、貴様、まだ朕の邪魔をするかっ！",
]

def translate(text, src, tgt):
    prompt = (
        f"You are a professional {src} to {tgt} translator. "
        f"Produce only the {tgt} translation, without any additional explanations or commentary.\n"
        f"Please translate the following text from {src} to {tgt}:\n\n"
        f"{text}"
    )
    resp = requests.post(OLLAMA_URL, json={
        "model": MODEL,
        "prompt": prompt,
        "stream": False,
        "options": {"temperature": 0.0, "num_predict": 200}
    }, timeout=120)
    if resp.status_code == 200:
        return resp.json().get("response", "").strip()
    return f"ERROR: {resp.status_code}"

print("=== TranslateGemma 4B Full Test ===")
print(f"Model: {MODEL}")
print(f"Phrases: {len(phrases)}")
print()

t0 = time.time()
for i, jp in enumerate(phrases):
    en = translate(jp, "Japanese", "English")
    ru = translate(jp, "Japanese", "Russian")
    print(f"[{i+1:2d}] JP: {jp}")
    print(f"     EN: {en}")
    print(f"     RU: {ru}")
    print()

elapsed = time.time() - t0
print(f"Done: {len(phrases)} phrases in {elapsed:.1f}s ({elapsed/len(phrases):.1f}s/phrase)")
