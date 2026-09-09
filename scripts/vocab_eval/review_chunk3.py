#!/usr/bin/env python3
"""Review chunk_3.jsonl for translation errors. Targeted checks."""
import json
import re
from pathlib import Path

INPUT = Path("/tmp/opencode/solar_chunks/chunk_3.jsonl")
CANDIDATES = Path("/home/yurvon/origa/scripts/vocab_eval/results/candidates.jsonl")
FIXES = Path("/home/yurvon/origa/scripts/vocab_eval/results/solar_fix_chunk3.jsonl")

# Specific error patterns
EN_IN_VI_KO = re.compile(r'\b(Chứng|impotent|Ugh|Europi|Mãĳo)\b', re.I)
MOJIBAKE = re.compile(r'[ĨŐŏőĳĲĴĵĶķĸĹĺĻļĽľĿŀŁłŃńŅņŇňŉŊŋŌōŎŏŐőŒœŔŕŖŗŘřŚśŜŝŞşȘšŢţŤťŦŧŨũŪūŬŭŮůŰűŲųŴŵŶŷŸŹźŻżŽž]')
BROKEN_TAIL = re.compile(r'\(biến thể cổ của|\( 의 정중어', re.I)
DUP_IN_BULLET = re.compile(r'Đột ngột, đột ngột', re.I)
WRONG_NUM_VI = re.compile(r'(sa nhị|sa nhì|tư ngày)', re.I)  # should be na nhị
LITERAL_VI = re.compile(r'(đề phương mét|đi phương mét|Cider cứng)', re.I)

# Known bad translations mapping
WRONG_MEANING = {
    'sparrow': 'Hoshigarasu (starling) NOT sparrow',
    'cypress': 'Tsuga (juniper) NOT cypress',
    'research': 'Renku (linked verse) NOT research',
    'chim sẻ': 'Hoshigarasu (starling) NOT sparrow',
    'tùng bách': 'Tsuga (juniper) NOT cypress',
    'nghiên cứu': 'Renku (linked verse) NOT research',
}

def check_bullets(text, lang):
    """Check a bullet text for specific issues."""
    found = []
    # English non-loanwords in vi/ko
    if EN_IN_VI_KO.search(text):
        found.append(f"EN word in {lang}: {text}")
    # Mojibake
    if MOJIBAKE.search(text):
        found.append(f"Mojibake in {lang}: {text}")
    # Broken tail
    if BROKEN_TAIL.search(text):
        found.append(f"Broken tail in {lang}: {text}")
    # Duplicate within bullet
    if DUP_IN_BULLET.search(text):
        found.append(f"Dup within bullet in {lang}: {text}")
    # Wrong number
    if WRONG_NUM_VI.search(text):
        found.append(f"Wrong number in {lang}: {text}")
    # Literalism
    if LITERAL_VI.search(text):
        found.append(f"Literalism in {lang}: {text}")
    return found

def check_duplicates(bullets, lang):
    """Find duplicate or near-duplicate bullets."""
    dups = []
    seen = {}
    for b in bullets:
        norm = re.sub(r'\s+', ' ', b.strip().lower())
        if norm in seen:
            dups.append((b, seen[norm]))
        else:
            seen[norm] = b
    # Near-duplicates (similar first 20 chars)
    for i, b1 in enumerate(bullets):
        for j, b2 in enumerate(bullets):
            if i < j:
                n1 = re.sub(r'\s+', ' ', b1.strip().lower())[:20]
                n2 = re.sub(r'\s+', ' ', b2.strip().lower())[:20]
                if n1 == n2 and len(n1) >= 10:
                    if (b1, b2) not in dups and (b2, b1) not in dups:
                        dups.append((b1, b2, 'near'))
    return dups

def check_vi_ko_field_mix(word, vi, ko):
    """Check if vi text appears in ko or vice versa."""
    issues = []
    # Vietnamese diacritics (specific range)
    vi_diacritics = re.compile(r'[àáạảãâầấậẩẫăằắặẳẵèéẹẻẽêềếệểễìíịỉĩòóọỏõôồốộổỗơờớợởỡùúụủũưừứựửữỳýỵỷỹđ]')
    # Korean hangul
    ko_hangul = re.compile(r'[가-힣]')
    
    for b in vi:
        if ko_hangul.search(b) and not vi_diacritics.search(b):
            issues.append(f"Ko text in vi field: {b}")
    for b in ko:
        if vi_diacritics.search(b) and not ko_hangul.search(b):
            issues.append(f"Vi text in ko field: {b}")
    return issues

issues = []
with open(INPUT, encoding='utf-8') as f:
    for i, line in enumerate(f, 1):
        line = line.strip()
        if not line:
            continue
        rec = json.loads(line)
        word = rec.get('word', '')
        vi = rec.get('vi', [])
        ko = rec.get('ko', [])
        vi_note = rec.get('vi_note', '')
        ko_note = rec.get('ko_note', '')
        
        word_issues = []
        
        # Check vi bullets
        for b in vi:
            word_issues.extend(check_bullets(b, 'vi'))
        # Check ko bullets
        for b in ko:
            word_issues.extend(check_bullets(b, 'ko'))
        
        # Check field mixing
        word_issues.extend(check_vi_ko_field_mix(word, vi, ko))
        
        # Check duplicates
        vi_dups = check_duplicates(vi, 'vi')
        ko_dups = check_duplicates(ko, 'ko')
        if vi_dups:
            word_issues.append(f"Vi duplicates: {vi_dups}")
        if ko_dups:
            word_issues.append(f"Ko duplicates: {ko_dups}")
        
        # Check wrong meaning
        vi_text = ' '.join(vi).lower()
        ko_text = ' '.join(ko).lower()
        en_text = ' '.join(rec.get('en', [])).lower()
        
        # Hoshigarasu = starling, not sparrow
        if word == 'ホシガラス' or 'hoshigarasu' in word.lower():
            if 'sparrow' in vi_text or 'chim sẻ' in vi_text:
                word_issues.append("WRONG: Hoshigarasu = sparrow (should be starling)")
        
        # Tsuga = juniper, not cypress
        if 'tsuga' in word.lower() or '杜松' in word:
            if 'cypress' in vi_text or 'tùng bách' in vi_text:
                word_issues.append("WRONG: Tsuga = cypress (should be juniper)")
        
        # Renku = linked verse, not research
        if 'renku' in word.lower() or '連句' in word:
            if 'research' in vi_text or 'nghiên cứu' in vi_text:
                word_issues.append("WRONG: Renku = research (should be linked verse)")
        
        # Wrong number 四日间
        if 'sa nhị' in vi_text or 'sa nhì' in vi_text:
            word_issues.append("WRONG NUMBER: sa nhị (should be na nhị for 四日间)")
        
        # Literalism 面積
        if 'đề phương mét' in vi_text or 'đi phương mét' in vi_text:
            word_issues.append("LITERALISM: đề phương mét")
        
        # Cider cứng
        if 'cider cứng' in vi_text:
            word_issues.append("LITERALISM: Cider cứng")
        
        # Empty fields
        if not vi and not vi_note:
            word_issues.append("Empty vi field")
        if not ko and not ko_note:
            word_issues.append("Empty ko field")
        
        if word_issues:
            issues.append({
                "line": i,
                "word": word,
                "issues": word_issues,
                "vi": vi,
                "ko": ko,
                "en": rec.get('en', [])
            })

with open(CANDIDATES, 'w', encoding='utf-8') as f:
    for issue in issues:
        f.write(json.dumps(issue, ensure_ascii=False))
        f.write('\n')

print(f"Found {len(issues)} words with potential issues")
print(f"Candidates written to {CANDIDATES}")
