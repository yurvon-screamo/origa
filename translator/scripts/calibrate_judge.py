#!/usr/bin/env python3
"""Build per-locale judge calibration from native-labeled pairs (issue #713).

Subcommands:
  export  Sample a labeling task from a production-flow candidate pool.
  import  Validate a labeled JSONL (schema, labels, duplicate ids).
  fit     Score pairs via the judge, fit per-locale thresholds, write the
          calibration file consumed by judge_api.Judge.verdict().

Round-trip: export writes the task JSONL (+ optional human-readable markdown);
native speakers mark good/bad in any medium; the operator transfers marks into
the task JSONL "label" field; import/fit validate that JSONL. Markdown is a
one-way view — it is never parsed back.

Sampling rule: the pool must be a plain random slice of the production flow of
the current translator. Do NOT balance classes by construction — balanced
pools distort the score-conditional class rates, and the committed
precision/recall stop matching production prevalence.

Environment (optional): ORIGA_JUDGE_HEADS (path to origa-judge.pt), ORIGA_JUDGE_CALIBRATION.

Usage:
  python calibrate_judge.py export --pool pool.jsonl --locales ko,vi \
      --n 200 --out task_ko_vi.jsonl [--md task_ko_vi.md] [--seed 7]
  python calibrate_judge.py import --labels task_ko_vi.jsonl
  python calibrate_judge.py fit --labels task_ko_vi.jsonl \
      [--policy max_f1|min_precision=0.95|min_recall=0.90] \
      [--out ../calibration/judge_calibration.json] [--seed 7] [--pool-rows N]
"""
from __future__ import annotations

import argparse
import hashlib
import json
import random
import sys
from collections import Counter
from pathlib import Path

import judge_calibration as jc

DEFAULT_SEED = 7
DEFAULT_OUT = Path(__file__).resolve().parent.parent / "calibration" / "judge_calibration.json"


def cmd_export(args: argparse.Namespace) -> int:
    pool = [json.loads(line) for line in
            Path(args.pool).read_text(encoding="utf-8").splitlines() if line.strip()]
    locales = [lc.strip() for lc in args.locales.split(",") if lc.strip()]
    for lc in locales:
        if not jc.LOCALE_RE.match(lc):
            print(f"error: locale {lc!r} does not match ^[a-z]{2}$", file=sys.stderr)
            return 2
    rng = random.Random(args.seed)
    out_rows: list[dict] = []
    for lc in locales:
        candidates = [row for row in pool if row.get("locale") == lc]
        if len(candidates) < args.n:
            print(f"error: pool has {len(candidates)} {lc} candidates < --n {args.n}",
                  file=sys.stderr)
            return 2
        picked = rng.sample(candidates, args.n)
        out_rows += [{"id": f"{lc}-{i:04d}", "locale": lc,
                      "state": row["state"], "translation": row["translation"]}
                     for i, row in enumerate(picked, start=1)]
    out = Path(args.out)
    out.write_text("".join(json.dumps(r, ensure_ascii=False) + "\n" for r in out_rows),
                   encoding="utf-8")
    if args.md:
        md = Path(args.md)
        md.write_text(render_markdown(out_rows), encoding="utf-8")
    print(f"exported {len(out_rows)} pairs -> {out}"
          + (f" (+ {md})" if args.md else ""))
    print("remind: pool_rows =", len(pool), "seed =", args.seed,
          "— pass both to fit via --pool-rows/--seed for provenance")
    return 0


def render_markdown(rows: list[dict]) -> str:
    blocks = ["# Translation labeling task\n",
              "Mark good only when the translation preserves the meaning and reads "
              "naturally. Mark bad when the meaning changes, the register is broken, "
              "or the text is in the wrong language.\n"]
    for row in rows:
        blocks.append(f"### {row['id']} ({row['locale']})\n"
                      f"- JP: {row['state']}\n"
                      f"- Translation: {row['translation']}\n"
                      f"- [ ] good\n- [ ] bad\n")
    return "\n".join(blocks)


def cmd_import(args: argparse.Namespace) -> int:
    lines = Path(args.labels).read_text(encoding="utf-8").splitlines()
    try:
        pairs, warnings = jc.validate_labeled_file(lines)
    except jc.ValidationError as exc:
        print(f"error: {exc}", file=sys.stderr)
        return 1
    for warning in warnings:
        print(f"warning: {warning}", file=sys.stderr)
    by_locale: Counter = Counter(p.locale for p in pairs)
    by_label: Counter = Counter((p.locale, p.label) for p in pairs)
    print(f"{len(pairs)} pairs valid")
    for lc in sorted(by_locale):
        good = by_label[lc, "good"]
        bad = by_label[lc, "bad"]
        verdict = "ok" if (good + bad >= jc.MIN_PAIRS
                           and min(good, bad) >= jc.MIN_PER_CLASS) else "below fit floor"
        print(f"  {lc}: good={good} bad={bad} -> {verdict}")
    return 0


def cmd_fit(args: argparse.Namespace) -> int:
    # torch loads only here; export/import stay dependency-free
    from judge_api import Judge

    lines = Path(args.labels).read_text(encoding="utf-8").splitlines()
    try:
        pairs, warnings = jc.validate_labeled_file(lines)
    except jc.ValidationError as exc:
        print(f"error: {exc}", file=sys.stderr)
        return 1
    for warning in warnings:
        print(f"warning: {warning}", file=sys.stderr)
    policy = jc.parse_policy(args.policy)
    judge = Judge()
    heads_sha = _sha256(judge.heads_path)
    if heads_sha is None:
        print(f"error: heads file not found: {judge.heads_path} "
              f"(set ORIGA_JUDGE_HEADS)", file=sys.stderr)
        return 2
    print(f"scoring {len(pairs)} pairs via {judge.url} …")
    scores = [judge.raw_score(p.state, p.translation) for p in pairs]
    entries, n_used, n_bad = {}, 0, 0
    failed = False
    for lc in sorted({p.locale for p in pairs}):
        lc_pairs = [p for p in pairs if p.locale == lc]
        lc_scores = [s for s, p in zip(scores, pairs) if p.locale == lc]
        try:
            entries[lc] = jc.fit_locale(lc, lc_pairs, lc_scores, policy)
        except (jc.InsufficientData, jc.PolicyInfeasible) as exc:
            print(f"  {lc}: SKIPPED ({exc})", file=sys.stderr)
            failed = True
            continue
        n_used += len(lc_pairs)
        n_bad += sum(p.label == "bad" for p in lc_pairs)
        _report_locale(lc, entries[lc])
    if not entries:
        print("error: no locale fitted", file=sys.stderr)
        return 1
    prevalence = n_bad / n_used if n_used else 0.0
    doc = jc.build_calibration_doc(entries, policy, heads_sha, n_used, prevalence,
                                   embedding_model="e5b", seed=args.seed,
                                   pool_rows=args.pool_rows)
    out = Path(args.out)
    out.parent.mkdir(parents=True, exist_ok=True)
    out.write_text(json.dumps(doc, ensure_ascii=False, indent=1) + "\n", encoding="utf-8")
    print(f"calibration -> {out} (heads_sha256 {heads_sha[:12]}…, {len(entries)} locale(s))")
    return 1 if failed else 0


def _report_locale(lc: str, entry: dict) -> None:
    metrics = entry["metrics"]
    cv = entry["cv"]
    cv_text = (f"cv f1={cv['f1_mean']:.3f}±{cv['f1_std']:.3f} ({cv['folds_used']}/{cv['folds']} folds)"
               if "f1_mean" in cv else f"cv unavailable: {cv.get('reason', '?')}")
    print(f"  {lc}: threshold={entry['threshold']:.4f} "
          f"P={metrics['precision']:.3f} R={metrics['recall']:.3f} F1={metrics['f1']:.3f} | {cv_text}")
    if entry["degenerate"]:
        print(f"  {lc}: WARNING degenerate threshold (accept-all/accept-none) — "
              f"check label quality", file=sys.stderr)


def _sha256(path: Path) -> str | None:
    if not path.exists():
        return None
    return hashlib.sha256(path.read_bytes()).hexdigest()


def main() -> int:
    ap = argparse.ArgumentParser(description=__doc__,
                                 formatter_class=argparse.RawDescriptionHelpFormatter)
    sub = ap.add_subparsers(dest="command", required=True)

    ex = sub.add_parser("export", help="sample a labeling task from a candidate pool")
    ex.add_argument("--pool", required=True, help="candidate pool JSONL: {locale, state, translation}")
    ex.add_argument("--locales", required=True, help="comma-separated, e.g. ko,vi")
    ex.add_argument("--n", type=int, default=200, help="pairs per locale")
    ex.add_argument("--out", required=True, help="task JSONL path")
    ex.add_argument("--md", help="optional human-readable markdown view (one-way)")
    ex.add_argument("--seed", type=int, default=DEFAULT_SEED)
    ex.set_defaults(func=cmd_export)

    im = sub.add_parser("import", help="validate a labeled JSONL")
    im.add_argument("--labels", required=True)
    im.set_defaults(func=cmd_import)

    fi = sub.add_parser("fit", help="score pairs and fit per-locale thresholds")
    fi.add_argument("--labels", required=True, help="labeled JSONL (task rows + label)")
    fi.add_argument("--policy", default="max_f1",
                    help="max_f1 | min_precision=X | min_recall=X")
    fi.add_argument("--out", default=str(DEFAULT_OUT))
    fi.add_argument("--seed", type=int, default=None,
                    help="provenance: seed used at export (see export output)")
    fi.add_argument("--pool-rows", type=int, default=None,
                    help="provenance: candidate pool size at export")
    fi.set_defaults(func=cmd_fit)

    args = ap.parse_args()
    return args.func(args)


if __name__ == "__main__":
    sys.exit(main())
