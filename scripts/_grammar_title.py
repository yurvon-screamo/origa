"""Title convention for grammar_v2 titles (issue #501).

Convention (documented in docs/plans/grammar-v2-schema.md, "Title
convention"):

    title = "pattern（qualifier）"

- full-width parens, no space before the opening paren;
- qualifier is localized per language (Japanese terms like （伝聞） are
  legitimate in both locales), at most two senses joined by ・, no JLPT
  level inside (the `level` field already carries it), at most 24 chars;
- a qualifier is REQUIRED when the normalized pattern is short (<= 2
  kana): bare particles like ～も or ～の are indistinguishable in card
  lists and lesson questions.

Duplicate detection is per language across different rules; an EN title
colliding with the RU title of the same rule is legitimate by design
(titles of unqualified rules are identical in both locales).
"""

from __future__ import annotations

import re

# Short patterns allowed to stay unqualified. Every entry must record its
# justification here — this list is the only escape hatch from the
# bare-short-pattern rule and must not grow silently.
BARE_PATTERN_ALLOWLIST: dict[str, str] = {
    # Recognizable without context by every learner from N5 up; the e2e
    # suite selects it via substring match (hasText), which a qualifier
    # would not break anyway.
    "ます": "universally recognizable polite form",
}

MAX_QUALIFIER_LEN = 24
MAX_QUALIFIER_SENSES = 2
# Counts normalized-pattern CHARACTERS (kana, kanji, latin) — the name
# stays about length, not script class.
SHORT_PATTERN_MAX_CHARS = 2

_FULLWIDTH_PARENS_RE = re.compile(r"（([^（]*)）")
_ASCII_PARENS_RE = re.compile(r"\(([^()]*)\)")
_PATTERN_LEADING_TILDE_RE = re.compile(r"^[～〜\s]+")
_PATTERN_INNER_TILDE_RE = re.compile(r"[～〜・]")
_PATTERN_DECOR_RE = re.compile(r"[\s/／]")
_JLPT_LEVEL_RE = re.compile(r"\bN[1-5]\b")
_CYRILLIC_RE = re.compile(r"[а-яА-ЯёЁ]")
_LATIN_RE = re.compile(r"[a-zA-Z]")


def split_title(title: str) -> tuple[str, str]:
    """Split a title into (pattern, qualifier).

    The qualifier is the content of the LAST paren group of either width;
    the parens themselves are removed from the pattern part. A title
    without parens yields an empty qualifier.
    """
    matches = list(_FULLWIDTH_PARENS_RE.finditer(title))
    matches += list(_ASCII_PARENS_RE.finditer(title))
    if not matches:
        return title, ""
    last = max(matches, key=lambda m: m.start())
    return title[: last.start()] + title[last.end() :], last.group(1)


def normalize_pattern(pattern: str) -> str:
    """Normalize the pattern part of a title for comparison.

    A leading ～ is pure decoration and is stripped; inner and trailing ～
    are positional placeholders and become "∅" so that ～の (nominalizer)
    and ～の～ (possessive "N の N") stay distinct. Enumeration dots ・
    between pattern members (こ・そ・あれ) become "∅" for the same reason:
    こ・そ (deixis series) must not collide with ～こそ (emphasis). Spaces
    and slashes are decorative and are removed.
    """
    stripped = _PATTERN_LEADING_TILDE_RE.sub("", pattern)
    placeholder = _PATTERN_INNER_TILDE_RE.sub("∅", stripped)
    return _PATTERN_DECOR_RE.sub("", placeholder)


def dup_key(title: str) -> str:
    """Dedup key of a title: normalized pattern + normalized qualifier.

    The qualifier is preserved (whitespace-collapsed, casefolded) so that
    already-disambiguated pairs like ～そうだ（伝聞）／～そうだ（様態） do not
    collide, while bare duplicates like ～の／～の do.
    """
    pattern, qualifier = split_title(title)
    normalized_qualifier = " ".join(qualifier.split()).casefold()
    return f"{normalize_pattern(pattern)}（{normalized_qualifier}）"


def title_lint_messages(title: str, lang: str) -> list[tuple[str, str]]:
    """Lint one title against the convention.

    Returns (level, message) pairs; level is "error" or "warn".
    """
    issues: list[tuple[str, str]] = []
    pattern, qualifier = split_title(title)
    normalized_pattern = normalize_pattern(pattern)

    if (
        len(normalized_pattern) <= SHORT_PATTERN_MAX_CHARS
        and not qualifier
        and normalized_pattern not in BARE_PATTERN_ALLOWLIST
    ):
        issues.append(
            ("error", f"bare short pattern {title!r}: qualifier is required")
        )

    if _ASCII_PARENS_RE.search(title):
        issues.append(("warn", "ASCII parens in title: use full-width （）"))

    if " （" in title or " (" in title:
        issues.append(("warn", "space before opening paren"))

    if not qualifier:
        return issues

    if len(qualifier) > MAX_QUALIFIER_LEN:
        issues.append(
            ("warn", f"qualifier longer than {MAX_QUALIFIER_LEN} chars")
        )
    senses = [sense for sense in qualifier.split("・") if sense]
    if len(senses) > MAX_QUALIFIER_SENSES:
        issues.append(
            ("warn", f"qualifier has {len(senses)} senses (max {MAX_QUALIFIER_SENSES})")
        )
    if _JLPT_LEVEL_RE.search(qualifier):
        issues.append(("warn", "JLPT level inside qualifier (level field exists)"))
    if lang == "English" and _CYRILLIC_RE.search(qualifier):
        issues.append(("warn", "cyrillic in English qualifier (heuristic)"))
    if lang == "Russian" and _LATIN_RE.search(qualifier):
        issues.append(("warn", "latin in Russian qualifier (heuristic)"))

    return issues
