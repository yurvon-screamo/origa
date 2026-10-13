#!/usr/bin/env python3
"""Per-locale calibration of judge absolute scores (issue #713).

Pure logic + data contracts: no torch, no network. Both judge_api.py (runtime
verdicts) and calibrate_judge.py (CLI: export/import/fit) build on this module.

Labeled pair (JSONL row):
    {"id": str, "locale": "^[a-z]{2}$", "state": str, "translation": str,
     "label": "good"|"bad", "source"?: str, "annotator"?: str, "note"?: str}

Calibration file (format 1):
    {"format": 1,
     "fitted_at": ISO-8601 UTC,
     "policy": {"name": "max_f1"|"min_precision"|"min_recall", "min"?: float},
     "heads_sha256": str,                  # SHA256 of the exact .pt bytes fitted against
     "embedding_model": str | null,        # model id reported by the embeddings API at fit
     "sampling": {"seed": int|null,        # export-time fact, carried via --seed/--pool-rows
                  "pool_rows": int|null,   # export-time fact
                  "sampled": int,          # labeled rows used by this fit
                  "prevalence_bad": float} # computed by fit from labels (never null)
     "locales": {lc: {"threshold": float, "n_good": int, "n_bad": int,
                      "metrics": {"precision", "recall", "f1"},
                      "confusion": {"tp", "fp", "tn", "fn"},
                      "degenerate": bool,  # accept-all / accept-none threshold
                      "cv": {"folds": 5, "folds_used": int, "f1_mean": float, "f1_std": float}
                            | {"status": "unavailable", "reason": str}}}}

Threshold semantics: ok = score >= threshold. Ties go to "good".
Policies: max_f1; min_precision=X / min_recall=X keep only feasible candidates,
then max F1; equal F1 -> higher threshold (fewer auto-accepts, deterministic).
An infeasible policy raises PolicyInfeasible(best_value).

Method requirements:
  - the labeled sample must be a plain random sample from the production flow
    (balanced pools distort score-conditional class rates and make committed
    P/R/F1 wrong for production prevalence);
  - fit against the exact .pt bytes that runtime will load (heads_sha256 is a
    byte hash of the heads file; a HF re-publish changes the bytes);
  - fit with the same encoder serving recipe as runtime (MEAN pooling,
    TRITON_ATTN — see hf/origa-judge.md).

Guards: a locale needs >= MIN_PAIRS pairs and >= MIN_PER_CLASS per class to fit;
cross-validation runs only at >= CV_MIN_PER_CLASS per class (5 stratified folds,
seed CV_SEED; folds where the policy is infeasible are excluded and folds_used
records how many survived).
"""
from __future__ import annotations

import hashlib
import json
import random
import re
import statistics
from dataclasses import dataclass
from datetime import datetime, timezone
from pathlib import Path
from typing import Final

FORMAT_VERSION: Final = 1
LABELS: Final = ("good", "bad")
KNOWN_LOCALES: Final = frozenset({"en", "ru", "ko", "vi"})
LOCALE_RE: Final = re.compile(r"^[a-z]{2}$")
CV_FOLDS: Final = 5
CV_SEED: Final = 7
MIN_PAIRS: Final = 40
MIN_PER_CLASS: Final = 8
CV_MIN_PER_CLASS: Final = 25


class CalibrationError(Exception):
    """Base class for calibration failures (runtime side)."""


class CalibrationLoadError(CalibrationError):
    """Calibration file missing or unreadable."""


class UnknownLocaleError(CalibrationError):
    """No fitted calibration for the requested locale."""


class HeadMismatchError(CalibrationError):
    """Calibration was fitted against different judge heads bytes."""


class ValidationError(Exception):
    """Labeled-file structure errors, with row numbers."""


class PolicyInfeasible(Exception):
    """No threshold satisfies the policy on the given scores."""

    def __init__(self, policy: str, best_value: float):
        self.best_value = best_value
        super().__init__(f"policy {policy} infeasible: best achievable {best_value:.3f}")


class InsufficientData(Exception):
    """A locale has too few labeled pairs to fit (see MIN_PAIRS / MIN_PER_CLASS)."""


@dataclass(frozen=True)
class LabeledPair:
    id: str
    locale: str
    state: str
    translation: str
    label: str
    source: str | None = None
    annotator: str | None = None
    note: str | None = None


@dataclass(frozen=True)
class Policy:
    name: str  # max_f1 | min_precision | min_recall
    minimum: float | None = None


@dataclass(frozen=True)
class ThresholdFit:
    threshold: float
    precision: float
    recall: float
    f1: float
    tp: int
    fp: int
    tn: int
    fn: int
    degenerate: bool  # accept-all or accept-none


def parse_policy(spec: str) -> Policy:
    """Parse "max_f1", "min_precision=0.95" or "min_recall=0.9"."""
    if spec == "max_f1":
        return Policy("max_f1")
    name, sep, raw = spec.partition("=")
    if name not in ("min_precision", "min_recall") or not sep:
        raise ValueError(f"unknown policy {spec!r}")
    return Policy(name, float(raw))


def parse_labeled_pair(obj: object, line_no: int) -> LabeledPair:
    if not isinstance(obj, dict):
        raise ValidationError(f"line {line_no}: not a JSON object")
    missing = [k for k in ("id", "locale", "state", "translation", "label") if not obj.get(k)]
    if missing:
        raise ValidationError(f"line {line_no}: missing fields {missing}")
    if obj["label"] not in LABELS:
        raise ValidationError(f"line {line_no}: label must be one of {LABELS}")
    if not LOCALE_RE.match(str(obj["locale"])):
        raise ValidationError(f"line {line_no}: locale must match ^[a-z]{2}$")
    return LabeledPair(
        id=str(obj["id"]), locale=str(obj["locale"]), state=str(obj["state"]),
        translation=str(obj["translation"]), label=str(obj["label"]),
        source=obj.get("source"), annotator=obj.get("annotator"), note=obj.get("note"),
    )


def validate_labeled_file(lines: list[str]) -> tuple[list[LabeledPair], list[str]]:
    """Parse all lines; return (pairs, warnings). Errors carry row numbers.

    A valid locale outside KNOWN_LOCALES is a warning, not an error: future
    locales are fittable, the runtime lookup stays strict.
    """
    pairs: list[LabeledPair] = []
    errors: list[str] = []
    warnings: list[str] = []
    seen: dict[str, int] = {}
    for line_no, line in enumerate(lines, start=1):
        line = line.strip()
        if not line:
            continue
        try:
            pair = parse_labeled_pair(json.loads(line), line_no)
        except json.JSONDecodeError as exc:
            errors.append(f"line {line_no}: invalid JSON ({exc})")
            continue
        except ValidationError as exc:
            errors.append(str(exc))
            continue
        if pair.id in seen:
            errors.append(f"line {line_no}: duplicate id {pair.id!r} (first at line {seen[pair.id]})")
            continue
        seen[pair.id] = line_no
        if pair.locale not in KNOWN_LOCALES:
            warnings.append(f"line {line_no}: locale {pair.locale!r} outside {sorted(KNOWN_LOCALES)}")
        pairs.append(pair)
    if errors:
        raise ValidationError(f"{len(errors)} invalid rows:\n" + "\n".join(errors))
    return pairs, warnings


def confusion_at(threshold: float, scored: list[tuple[float, str]]) -> tuple[int, int, int, int]:
    """Return (tp, fp, tn, fn) for prediction ok = score >= threshold."""
    tp = fp = tn = fn = 0
    for score, label in scored:
        accepted = score >= threshold
        good = label == "good"
        tp += accepted and good
        fp += accepted and not good
        tn += not accepted and not good
        fn += not accepted and good
    return tp, fp, tn, fn


def _prf(tp: int, fp: int, tn: int, fn: int) -> tuple[float, float, float]:
    precision = tp / (tp + fp) if tp + fp else 0.0
    recall = tp / (tp + fn) if tp + fn else 0.0
    f1 = 2 * precision * recall / (precision + recall) if precision + recall else 0.0
    return precision, recall, f1


def _policy_value(policy: Policy, precision: float, recall: float) -> float:
    if policy.name == "min_precision":
        return precision
    if policy.name == "min_recall":
        return recall
    return 1.0  # max_f1: every candidate is feasible


def fit_threshold(scored: list[tuple[float, str]], policy: Policy) -> ThresholdFit:
    """Pick the operating point per policy; deterministic on ties.

    Candidates: midpoints between adjacent unique scores plus one edge below
    the minimum (accept-all) and one above the maximum (accept-none).
    """
    if not scored:
        raise ValueError("no scored pairs")
    unique = sorted({s for s, _ in scored})
    mids = [(a + b) / 2 for a, b in zip(unique, unique[1:])]
    edge_low, edge_high = unique[0] - 1.0, unique[-1] + 1.0
    feasible = []
    best_infeasible = 0.0
    for t in [edge_low, *mids, edge_high]:
        tp, fp, tn, fn = confusion_at(t, scored)
        precision, recall, f1 = _prf(tp, fp, tn, fn)
        met = policy.minimum is None or _policy_value(policy, precision, recall) >= policy.minimum
        if not met:
            best_infeasible = max(best_infeasible, _policy_value(policy, precision, recall))
            continue
        degenerate = t == edge_low or t == edge_high
        fit = ThresholdFit(t, precision, recall, f1, tp, fp, tn, fn, degenerate)
        # max F1; tie -> higher threshold (fewer auto-accepts)
        feasible.append((f1, t, fit))
    if not feasible:
        raise PolicyInfeasible(policy.name, best_infeasible)
    feasible.sort(key=lambda row: (row[0], row[1]))
    return feasible[-1][2]


def good_scores_stats(scores: list[float]) -> dict[str, float]:
    """mean/std/quantiles of good-class scores (for percentile double-confirmation scans)."""

    def quantile(sorted_values: list[float], q: float) -> float:
        pos = (len(sorted_values) - 1) * q
        low, high = int(pos), min(int(pos) + 1, len(sorted_values) - 1)
        return sorted_values[low] + (sorted_values[high] - sorted_values[low]) * (pos - low)

    ordered = sorted(scores)
    return {
        "mean": statistics.fmean(ordered),
        "std": statistics.stdev(ordered) if len(ordered) > 1 else 0.0,
        "p05": quantile(ordered, 0.05),
        "p50": quantile(ordered, 0.50),
        "p95": quantile(ordered, 0.95),
    }


def cross_validate(scored: list[tuple[float, str]], policy: Policy,
                   folds: int = CV_FOLDS, seed: int = CV_SEED) -> dict:
    """Stratified k-fold CV of the whole fit procedure (threshold included).

    Folds where the policy is infeasible are excluded; folds_used records how
    many survived. Returns the object form or {"status": "unavailable", "reason"}.
    """
    rng = random.Random(seed)
    by_class = {label: [s for s, l in scored if l == label] for label in ("good", "bad")}
    if min(len(v) for v in by_class.values()) < CV_MIN_PER_CLASS:
        smallest = min(by_class, key=lambda k: len(by_class[k]))
        return {"status": "unavailable",
                "reason": f"per-class count {smallest}={len(by_class[smallest])} < {CV_MIN_PER_CLASS}"}
    deal: dict[str, list[list[float]]] = {}
    for label, scores in by_class.items():
        rng.shuffle(scores)
        deal[label] = [scores[i::folds] for i in range(folds)]
    fold_f1: list[float] = []
    infeasible = 0
    for fold in range(folds):
        train = [(s, "good") for i, chunk in enumerate(deal["good"]) if i != fold for s in chunk]
        train += [(s, "bad") for i, chunk in enumerate(deal["bad"]) if i != fold for s in chunk]
        test = [(s, "good") for s in deal["good"][fold]] + [(s, "bad") for s in deal["bad"][fold]]
        try:
            fit = fit_threshold(train, policy)
        except PolicyInfeasible:
            infeasible += 1
            continue
        tp, fp, tn, fn = confusion_at(fit.threshold, test)
        fold_f1.append(_prf(tp, fp, tn, fn)[2])
    if not fold_f1:
        return {"status": "unavailable", "reason": f"policy infeasible in all {folds} folds"}
    return {
        "folds": folds,
        "folds_used": len(fold_f1),
        "f1_mean": statistics.fmean(fold_f1),
        "f1_std": statistics.stdev(fold_f1) if len(fold_f1) > 1 else 0.0,
        **({"infeasible_folds": infeasible} if infeasible else {}),
    }


def fit_locale(locale: str, pairs: list[LabeledPair], scores: list[float],
               policy: Policy) -> dict:
    """Fit one locale end-to-end: guard -> threshold -> CV -> stats -> entry dict.

    `scores` is parallel to `pairs` (raw judge scores).
    """
    if len(scores) != len(pairs):
        raise ValueError(f"{locale}: scores ({len(scores)}) and pairs ({len(pairs)}) misaligned")
    if len(pairs) < MIN_PAIRS:
        raise InsufficientData(f"{locale}: {len(pairs)} pairs < {MIN_PAIRS}")
    scored = list(zip(scores, (p.label for p in pairs)))
    n_bad = sum(p.label == "bad" for p in pairs)
    n_good = len(pairs) - n_bad
    if min(n_good, n_bad) < MIN_PER_CLASS:
        raise InsufficientData(f"{locale}: per-class counts good={n_good} bad={n_bad} "
                               f"< {MIN_PER_CLASS}")
    fit = fit_threshold(scored, policy)
    good_scores = [s for s, p in zip(scores, pairs) if p.label == "good"]
    return {
        "threshold": fit.threshold,
        "n_good": n_good,
        "n_bad": n_bad,
        "metrics": {"precision": fit.precision, "recall": fit.recall, "f1": fit.f1},
        "confusion": {"tp": fit.tp, "fp": fit.fp, "tn": fit.tn, "fn": fit.fn},
        "degenerate": fit.degenerate,
        "cv": cross_validate(scored, policy),
        "good_stats": good_scores_stats(good_scores),
    }


def verify_head_hash(expected_sha256: str, heads_path: Path) -> None:
    """Fail loud when the loaded heads bytes differ from the fitted ones."""
    digest = hashlib.sha256(heads_path.read_bytes()).hexdigest()
    if digest != expected_sha256:
        raise HeadMismatchError(
            f"calibration was fitted against different judge heads: "
            f"fitted {expected_sha256[:12]}…, loaded {digest[:12]}… ({heads_path}). "
            f"Re-fit with calibrate_judge.py against this exact .pt file.")


class Calibration:
    """Loaded calibration file: per-locale thresholds + lookup."""

    def __init__(self, doc: dict):
        self.doc = doc

    @classmethod
    def load(cls, path: Path) -> Calibration:
        if not path.exists():
            raise CalibrationLoadError(
                f"calibration file not found: {path}. "
                f"Fit one with calibrate_judge.py (issue #713); "
                f"until then use ranking (Best-of-N), not absolute verdicts.")
        doc = json.loads(path.read_text(encoding="utf-8"))
        if not isinstance(doc, dict) or doc.get("format") != FORMAT_VERSION:
            raise CalibrationLoadError(f"unsupported calibration format in {path}")
        return cls(doc)

    def entry(self, locale: str) -> dict:
        locales = self.doc.get("locales") or {}
        if locale not in locales:
            raise UnknownLocaleError(
                f"no calibration for locale {locale!r} "
                f"(fitted: {sorted(locales)}). Sample and label pairs with "
                f"calibrate_judge.py export/import, then fit (issue #713).")
        return locales[locale]

    def decide(self, locale: str, score: float) -> bool:
        return score >= self.entry(locale)["threshold"]


def build_calibration_doc(locales: dict, policy: Policy, heads_sha256: str,
                          sampled: int, prevalence_bad: float,
                          embedding_model: str | None = None,
                          seed: int | None = None, pool_rows: int | None = None) -> dict:
    return {
        "format": FORMAT_VERSION,
        "fitted_at": datetime.now(timezone.utc).isoformat(timespec="seconds"),
        "policy": {"name": policy.name, **({"min": policy.minimum} if policy.minimum is not None else {})},
        "heads_sha256": heads_sha256,
        "embedding_model": embedding_model,
        "sampling": {"seed": seed, "pool_rows": pool_rows,
                     "sampled": sampled, "prevalence_bad": prevalence_bad},
        "locales": locales,
    }
