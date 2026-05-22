import requests

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

def translate(text, src_code, tgt_code):
    prompt = f"You are a professional {src_code} to {tgt_code} translator. Your goal is to accurately convey the meaning and nuances of the original text while adhering to {tgt_code} grammar, vocabulary, and cultural sensitivities. Produce only the {tgt_code} translation, without any additional explanations or commentary.\nPlease translate the following text from {src_code} to {tgt_code}:\n\n{text}"
    resp = requests.post(OLLAMA_URL, json={
        "model": MODEL,
        "prompt": prompt,
        "stream": False,
        "options": {"temperature": 0.0, "num_predict": 200}
    }, timeout=120)
    if resp.status_code == 200:
        return resp.json().get("response", "").strip()
    return f"ERROR: {resp.status_code}"

print("=== TranslateGemma 4B (official prompt) ===\n")

for jp in phrases:
    en = translate(jp, "Japanese", "English")
    ru = translate(jp, "Japanese", "Russian")
    print(f"JP: {jp}")
    print(f"EN: {en}")
    print(f"RU: {ru}")
    print()
