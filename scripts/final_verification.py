import json, re, glob

# === VOCABULARY CHECK ===
print("=" * 60)
print("VOCABULARY DICTIONARY CHECK")
print("=" * 60)

JP_REGEX = re.compile(r'[\u3040-\u309F\u30A0-\u30FF\u4E00-\u9FFF\u3400-\u4DBF]')
# Simplified Chinese characters that don't exist in Japanese
SIMPLIFIED_CN = re.compile(r'[织预绕宽结书计线贤适间杂节说纲压产广剧亲华汉导]')
CYRILLIC = re.compile(r'[\u0400-\u04FF]')
LATIN = re.compile(r'[a-zA-Z]')

vocab_stats = {
    "total": 0,
    "gt_blocks": 0,
    "simplified_chinese_ru": 0,
    "simplified_chinese_en": 0,
    "jp_in_ru": 0,
    "jp_in_en": 0,
    "cross_ru_en": 0,
    "cross_en_ru": 0,
}

for f in sorted(glob.glob('cdn/dictionary/chunk_*.json')):
    with open(f, encoding='utf-8') as fh:
        data = json.load(fh)
    vocab_stats["total"] += len(data)
    
    for word, entry in data.items():
        ru = entry.get('russian_translation', '')
        en = entry.get('english_translation', '')
        
        # Check for > blocks
        for text, field in [(ru, 'ru'), (en, 'en')]:
            for line in text.split('\n'):
                stripped = line.lstrip()
                if stripped.startswith('>') and len(stripped) > 1:
                    vocab_stats["gt_blocks"] += 1
                    print(f"  > BLOCK REMAINING: {word} ({field}): {stripped[:80]}")
        
        # Check for simplified Chinese in RU
        if SIMPLIFIED_CN.search(ru):
            vocab_stats["simplified_chinese_ru"] += 1
            matches = SIMPLIFIED_CN.findall(ru)
            print(f"  SIMPLIFIED CN IN RU: {word}: {matches}")
        
        # Check for simplified Chinese in EN
        if SIMPLIFIED_CN.search(en):
            vocab_stats["simplified_chinese_en"] += 1
            matches = SIMPLIFIED_CN.findall(en)
            print(f"  SIMPLIFIED CN IN EN: {word}: {matches}")
        
        # Check for JP chars in RU
        if JP_REGEX.search(ru):
            vocab_stats["jp_in_ru"] += 1
        
        # Check for JP chars in EN
        if JP_REGEX.search(en):
            vocab_stats["jp_in_en"] += 1

print(f"\n  Total entries: {vocab_stats['total']}")
print(f"  > blocks remaining: {vocab_stats['gt_blocks']}")
print(f"  Simplified Chinese in RU: {vocab_stats['simplified_chinese_ru']}")
print(f"  Simplified Chinese in EN: {vocab_stats['simplified_chinese_en']}")
print(f"  JP chars in RU (includes legitimate): {vocab_stats['jp_in_ru']}")
print(f"  JP chars in EN (includes legitimate): {vocab_stats['jp_in_en']}")

# === PHRASES CHECK ===
print(f"\n{'=' * 60}")
print("PHRASES CHECK")
print("=" * 60)

phrase_stats = {
    "total": 0,
    "cross_script": 0,
    "jp_in_ru": 0,
    "jp_in_en": 0,
}

def has_mixed_script(word):
    """Check if a word contains both Cyrillic and Latin characters."""
    has_cyrillic = bool(CYRILLIC.search(word))
    has_latin = bool(LATIN.search(word))
    return has_cyrillic and has_latin

for f in sorted(glob.glob('cdn/phrases/data/p*.json')):
    with open(f, encoding='utf-8') as fh:
        phrases = json.load(fh)
    vocab_stats["total"]  # not counting phrases in total
    phrase_stats["total"] += len(phrases)
    
    for p in phrases:
        ru = p.get('ru') or ''
        en = p.get('en') or ''
        
        # Check for mixed script words
        for text, field in [(ru, 'ru'), (en, 'en')]:
            words = re.findall(r'\S+', text)
            for w in words:
                # Strip punctuation
                clean = re.sub(r'[^\w]', '', w)
                if len(clean) >= 2 and has_mixed_script(clean):
                    phrase_stats["cross_script"] += 1
                    print(f"  CROSS-SCRIPT: {p['i'][:20]} ({field}): '{clean}' in '{text[:60]}'")
        
        # Check for JP chars
        if JP_REGEX.search(ru):
            phrase_stats["jp_in_ru"] += 1
        if JP_REGEX.search(en):
            phrase_stats["jp_in_en"] += 1

print(f"\n  Total phrases: {phrase_stats['total']}")
print(f"  Cross-script words remaining: {phrase_stats['cross_script']}")
print(f"  JP chars in RU: {phrase_stats['jp_in_ru']}")
print(f"  JP chars in EN: {phrase_stats['jp_in_en']}")

print(f"\n{'=' * 60}")
print("DONE")
print("=" * 60)
