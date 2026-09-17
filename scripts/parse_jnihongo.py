#!/usr/bin/env python3
"""Parse downloaded J-nihongo grammar pages into structured facts.

Input:  /tmp/opencode/jnihongo/pages/*.html + catalog.json
Output: /tmp/opencode/jnihongo/facts.json — one record per page:

    {
      "slug": "teiru-state-habit", "level": "N5",
      "title": "〜ている／〜ています",
      "summary_ja": "動詞のて形に…",
      "summary_en": "Adds iru or imasu…",
      "usages": [
        {"name": "動作の進行", "desc_ja": "…",
         "examples": [{"jp": "今、本を読んでいます。", "en": "I'm reading…"}]}
      ],
      "formation": [{"kind": "普通形", "connection": "Vて形 + いる", "example": "本を読んでいる。"}],
      "points": ["…", "…"],
      "similar": [{"title": "〜てある（結果状態）", "level": "N5", "slug": "jlpt-n5-0041"}]
    }

Furigana is dropped (base text kept). These facts ground the generation
prompt: usage categories, formation table, nuance points, and the
similar-grammar map.
"""

import html as htmllib
import json
import re
import sys
from pathlib import Path

ROOT = Path("/tmp/opencode/jnihongo")


def clean_text(s: str) -> str:
    s = re.sub(r"<rt[^>]*>[^<]*</rt>", "", s)  # furigana readings
    s = re.sub(r"<[^>]+>", "", s)
    return htmllib.unescape(s).strip()


def parse_page(path: Path) -> dict | None:
    html = path.read_text(encoding="utf-8")

    m = re.search(r"<h1[^>]*>(.*?)</h1>", html, re.S)
    title = clean_text(m.group(1)) if m else ""
    if not title:
        return None

    # summary: JP paragraph + EN translation
    summary_ja = summary_en = ""
    m = re.search(r'<section class="summary[^"]*"[^>]*>(.*?)</section>', html, re.S)
    if m:
        block = m.group(1)
        p = re.findall(r"<p[^>]*>(.*?)</p>", block, re.S)
        if p:
            summary_ja = clean_text(p[0])
        m_en = re.search(r'class="summary-translation"[^>]*>(.*?)</p>', block, re.S)
        if m_en:
            summary_en = clean_text(m_en.group(1))

    # usage sections (only newer pages have them) + ALL example rows,
    # assigned to categories by position; rows outside any section go to
    # a synthetic "general" bucket.
    usage_spans = []  # (start, end, name, desc)
    for sec in re.finditer(
        r'<section class="example-usage"[^>]*id="usage-(\d+)"[^>]*>', html
    ):
        head = re.search(
            r"<h3[^>]*>(.*?)</h3>.*?<p[^>]*>(.*?)</p>", html[sec.start() : sec.start() + 2500], re.S
        )
        end_match = re.search(r'<section class="example-usage"', html[sec.end() :])
        end = sec.end() + end_match.start() if end_match else len(html)
        usage_spans.append(
            (
                sec.start(),
                end,
                clean_text(head.group(1)) if head else f"用法 {sec.group(1)}",
                clean_text(head.group(2)) if head else "",
            )
        )

    usages: list[dict] = [
        {"name": name, "desc_ja": desc, "examples": []} for _, _, name, desc in usage_spans
    ] or [{"name": "general", "desc_ja": "", "examples": []}]
    for row in re.finditer(
        r'<div class="example-row"><div><p>(.*?)</p><small class="example-translation"[^>]*>(.*?)</small>',
        html,
        re.S,
    ):
        entry = {"jp": clean_text(row.group(1)), "en": clean_text(row.group(2))}
        for i, (start, end, _, _) in enumerate(usage_spans):
            if start <= row.start() < end:
                usages[i]["examples"].append(entry)
                break
        else:
            usages[-1]["examples"].append(entry)
    usages = [u for u in usages if u["examples"]]

    # formation table
    formation = []
    m = re.search(r"<table>.*?<tbody>(.*?)</tbody>", html, re.S)
    if m:
        for tr in re.finditer(r"<tr>(.*?)</tr>", m.group(1), re.S):
            cells = [clean_text(c) for c in re.findall(r"<td[^>]*>(.*?)</td>", tr.group(1), re.S)]
            if len(cells) >= 3:
                formation.append({"kind": cells[0], "connection": cells[1], "example": cells[2]})

    # usage points
    points = []
    m = re.search(r'id="usage-points".*?<ul>(.*?)</ul>', html, re.S)
    if m:
        points = [clean_text(li) for li in re.findall(r"<li[^>]*>(.*?)</li>", m.group(1), re.S)]

    # similar grammar cards
    similar = []
    for card in re.finditer(
        r'<a href="/([a-z0-9_-]+)/?" class="similar-grammar-card level-(n[1-5])"(.*?)</a>',
        html,
        re.S,
    ):
        inner = card.group(3)
        t = re.search(r"<h3[^>]*>(.*?)</h3>", inner, re.S)
        if t:
            similar.append(
                {
                    "slug": card.group(1),
                    "level": card.group(2).upper(),
                    "title": clean_text(t.group(1)),
                }
            )

    return {
        "slug": path.stem,
        "title": title,
        "summary_ja": summary_ja,
        "summary_en": summary_en,
        "usages": usages,
        "formation": formation,
        "points": points,
        "similar": similar,
    }


def main() -> int:
    catalog = json.loads((ROOT / "catalog.json").read_text(encoding="utf-8"))
    level_by_slug = {c["slug"]: c["level"].upper() for c in catalog}

    facts, failures = [], []
    for entry in sorted(catalog, key=lambda c: c["slug"]):
        path = ROOT / entry["file"]
        try:
            rec = parse_page(path)
        except Exception as e:  # noqa: BLE001 — report, don't die
            failures.append((entry["slug"], str(e)))
            continue
        if not rec or not rec["title"]:
            failures.append((entry["slug"], "no title"))
            continue
        rec["level"] = level_by_slug.get(entry["slug"], "?")
        facts.append(rec)

    out = ROOT / "facts.json"
    out.write_text(json.dumps(facts, ensure_ascii=False, indent=1), encoding="utf-8")

    n = len(facts)
    with_usages = sum(1 for f in facts if f["usages"])
    with_formation = sum(1 for f in facts if f["formation"])
    with_points = sum(1 for f in facts if f["points"])
    with_similar = sum(1 for f in facts if f["similar"])
    ex_total = sum(len(u["examples"]) for f in facts for u in f["usages"])
    print(f"parsed {n}/{len(catalog)} pages -> {out}")
    print(
        f"  usages: {with_usages} | formation: {with_formation} | "
        f"points: {with_points} | similar: {with_similar} | examples total: {ex_total}"
    )
    if failures:
        print(f"  FAILURES ({len(failures)}):")
        for slug, err in failures[:10]:
            print(f"    {slug}: {err}")
    return 1 if failures else 0


if __name__ == "__main__":
    sys.exit(main())
