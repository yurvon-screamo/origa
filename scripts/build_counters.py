#!/usr/bin/env python3
"""Build cdn/counters/counters.json from the manual git source.

Expands the compact manual schema (canonical reading + irregular cells
only) into the full wire format the client registry consumes:
every cell 0..=10 (+ registry beyond-ten exceptions) carries its final
reading. Run by deploy_cdn.py before upload; also a standalone gate:
fails loudly on schema violations (issue #415, Slice 4).

Usage: python scripts/build_counters.py [--check]
  --check: verify the artifact is fresh instead of writing it.
"""

from __future__ import annotations

import json
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
MANUAL = ROOT / "scripts" / "data" / "counters" / "counters.manual.json"
ARTIFACT = ROOT / "cdn" / "counters" / "counters.json"


def die(msg: str) -> None:
    print(f"build_counters: {msg}", file=sys.stderr)
    raise SystemExit(1)


def expand(manual: dict) -> dict:
    meta = manual["meta"]
    numbers: dict[str, str] = meta["number_readings"]
    counters_out = []
    for entry in manual["counters"]:
        suffix = entry["suffix"]
        glosses = entry.get("glosses", {})
        for locale in ("ru", "en", "ko", "vi"):
            if not glosses.get(locale, "").strip():
                die(f"{suffix}: empty {locale} gloss (4 locales are mandatory)")
        repr_reading = entry.get("repr_reading")
        irregular: dict[str, str] = entry.get("irregular", {})
        beyond: dict[str, str] = entry.get("beyond_ten", {})

        readings = []
        if repr_reading is None:
            for key in [str(n) for n in range(1, 11)] + ["0"]:
                if key not in irregular:
                    die(f"{suffix}: kun-series counter missing irregular[{key}]")
            ordered = [str(n) for n in range(1, 11)] + ["0"]
            for key in ordered:
                # Кун-серия (つ): вся таблица «нерегулярна» относительно
                # он-конкатенации — подсветка каждой строки не информирует,
                # все ячейки идут без флага.
                readings.append(
                    {"number": int(key), "reading": irregular[key], "irregular": False}
                )
        else:
            ordered = [str(n) for n in range(1, 11)] + ["0"]
            for key in ordered:
                numeral = numbers[key]
                reading = irregular.get(key, numeral + repr_reading)
                readings.append(
                    {
                        "number": int(key),
                        "reading": reading,
                        "irregular": key in irregular,
                    }
                )
        for key, reading in sorted(beyond.items(), key=lambda kv: int(kv[0])):
            number = int(key)
            if number <= 10:
                die(f"{suffix}: beyond_ten entry {key} is not > 10")
            readings.append(
                {"number": number, "reading": reading, "irregular": True}
            )

        counters_out.append(
            {
                "suffix": suffix,
                "level": entry["level"],
                "glosses": {
                    "Russian": glosses["ru"],
                    "English": glosses["en"],
                    "Korean": glosses["ko"],
                    "Vietnamese": glosses["vi"],
                },
                "readings": readings,
            }
        )
    return {"schema": 1, "counters": counters_out}


def main() -> None:
    check_only = "--check" in sys.argv
    manual = json.loads(MANUAL.read_text(encoding="utf-8"))
    expanded = expand(manual)
    n = len(expanded["counters"])
    levels: dict[str, int] = {}
    for c in expanded["counters"]:
        levels[c["level"]] = levels.get(c["level"], 0) + 1
    if n != 76 or levels != {"N5": 19, "N4": 14, "N3": 18, "N2": 16, "N1": 9}:
        die(f"unexpected dataset shape: {n} counters, levels {levels}")

    payload = json.dumps(expanded, ensure_ascii=False, indent=1) + "\n"
    if check_only:
        current = (
            ARTIFACT.read_text(encoding="utf-8") if ARTIFACT.exists() else ""
        )
        if current != payload:
            die(f"artifact stale vs {MANUAL}: rebuild without --check")
        print(f"build_counters: artifact fresh ({n} counters)")
        return

    ARTIFACT.parent.mkdir(parents=True, exist_ok=True)
    ARTIFACT.write_text(payload, encoding="utf-8")
    print(f"build_counters: wrote {ARTIFACT} ({n} counters, levels {levels})")


if __name__ == "__main__":
    main()
