#!/usr/bin/env python3
"""Behavior tests for judge_calibration (pure logic; no torch, no network).

Run: python -m pytest translator/scripts -q
"""
import json

import judge_calibration as jc
import pytest


def make_pairs(n_good: int, n_bad: int, locale: str = "ko") -> list[jc.LabeledPair]:
    pairs = []
    for i in range(n_good):
        pairs.append(jc.LabeledPair(f"{locale}-g{i}", locale, f"jp {i}", f"good {i}", "good"))
    for i in range(n_bad):
        pairs.append(jc.LabeledPair(f"{locale}-b{i}", locale, f"jp {i}", f"bad {i}", "bad"))
    return pairs


def make_scored(n_good: int, n_bad: int, good_range=(0.6, 1.0), bad_range=(0.0, 0.4)):
    """Well-separated synthetic scores (good above bad)."""
    scored = [(good_range[0] + (good_range[1] - good_range[0]) * i / max(n_good - 1, 1), "good")
              for i in range(n_good)]
    scored += [(bad_range[0] + (bad_range[1] - bad_range[0]) * i / max(n_bad - 1, 1), "bad")
               for i in range(n_bad)]
    return scored


class TestParseAndValidate:
    def test_valid_row_parses_with_optional_fields(self):
        row = {"id": "ko-0001", "locale": "ko", "state": "jp", "translation": "t",
               "label": "good", "annotator": "kim", "source": "scan"}
        pair = jc.parse_labeled_pair(row, 1)
        assert pair.annotator == "kim" and pair.source == "scan"

    def test_missing_required_field_reports_line_number(self):
        row = {"id": "ko-0001", "locale": "ko", "state": "jp", "label": "good"}
        with pytest.raises(jc.ValidationError, match="line 1"):
            jc.parse_labeled_pair(row, 1)

    def test_unknown_label_rejected(self):
        row = {"id": "x", "locale": "ko", "state": "s", "translation": "t", "label": "maybe"}
        with pytest.raises(jc.ValidationError, match="label"):
            jc.parse_labeled_pair(row, 3)

    def test_bad_locale_format_rejected(self):
        row = {"id": "x", "locale": "KOR", "state": "s", "translation": "t", "label": "good"}
        with pytest.raises(jc.ValidationError, match="locale"):
            jc.parse_labeled_pair(row, 1)

    def test_duplicate_ids_reported_with_both_lines(self):
        row = json.dumps({"id": "ko-1", "locale": "ko", "state": "s",
                          "translation": "t", "label": "good"})
        with pytest.raises(jc.ValidationError, match="line 2.*duplicate id 'ko-1'.*line 1"):
            jc.validate_labeled_file([row, row])

    def test_unknown_known_locale_is_warning_not_error(self):
        row = json.dumps({"id": "x", "locale": "th", "state": "s",
                          "translation": "t", "label": "good"})
        pairs, warnings = jc.validate_labeled_file([row])
        assert len(pairs) == 1
        assert any("'th'" in w for w in warnings)


class TestFitThreshold:
    def test_separable_classes_get_perfect_fit(self):
        fit = jc.fit_threshold(make_scored(30, 30), jc.Policy("max_f1"))
        assert 0.4 < fit.threshold < 0.6
        assert (fit.precision, fit.recall, fit.f1) == (1.0, 1.0, 1.0)
        assert (fit.tp, fit.fp, fit.tn, fit.fn) == (30, 0, 30, 0)
        assert fit.degenerate is False

    def test_equal_f1_tie_prefers_higher_threshold(self):
        # cutoffs 0.15 and 0.35 both give F1=2/3 (P=.5,R=1 vs P=1,R=.5) -> higher wins
        scored = [(0.1, "bad"), (0.2, "bad"), (0.2, "good"), (0.3, "bad"), (0.4, "good")]
        fit = jc.fit_threshold(scored, jc.Policy("max_f1"))
        assert fit.threshold == pytest.approx(0.35)

    def test_min_precision_constraint_excludes_low_precision_points(self):
        # one clear outlier bad at 0.95 pulls accept-all to precision 0.8
        scored = make_scored(4, 4)
        scored.append((0.95, "bad"))
        fit = jc.fit_threshold(scored, jc.Policy("min_precision", 0.9))
        assert fit.precision >= 0.9

    def test_infeasible_policy_raises_with_best_value(self):
        scored = make_scored(10, 10, good_range=(0.0, 0.5), bad_range=(0.4, 0.9))
        with pytest.raises(jc.PolicyInfeasible, match="min_precision"):
            jc.fit_threshold(scored, jc.Policy("min_precision", 0.99))

    def test_score_tie_breaks_toward_good(self):
        fit = jc.fit_threshold([(0.5, "good"), (0.5, "bad")], jc.Policy("max_f1"))
        assert jc.confusion_at(fit.threshold, [(0.5, "good")]) == (1, 0, 0, 0)


class TestCrossValidation:
    def test_enough_per_class_reports_five_used_folds(self):
        report = jc.cross_validate(make_scored(30, 30), jc.Policy("max_f1"))
        assert report["folds"] == 5
        assert report["folds_used"] == 5
        assert 0.5 <= report["f1_mean"] <= 1.0

    def test_small_class_marks_cv_unavailable_with_reason(self):
        report = jc.cross_validate(make_scored(30, 10), jc.Policy("max_f1"))
        assert report["status"] == "unavailable"
        assert "bad=10" in report["reason"]

    def test_deterministic_across_calls(self):
        first = jc.cross_validate(make_scored(30, 30), jc.Policy("max_f1"))
        second = jc.cross_validate(make_scored(30, 30), jc.Policy("max_f1"))
        assert first == second


class TestFitLocale:
    def test_below_pair_floor_rejected(self):
        pairs = make_pairs(30, 9)  # 39 < MIN_PAIRS
        scores = [0.5] * len(pairs)
        with pytest.raises(jc.InsufficientData, match="39 pairs"):
            jc.fit_locale("ko", pairs, scores, jc.Policy("max_f1"))

    def test_below_class_floor_rejected(self):
        pairs = make_pairs(70, 5)
        scores = [0.5] * len(pairs)
        with pytest.raises(jc.InsufficientData, match="good=70 bad=5"):
            jc.fit_locale("ko", pairs, scores, jc.Policy("max_f1"))

    def test_misaligned_scores_rejected(self):
        pairs = make_pairs(50, 50)
        with pytest.raises(ValueError, match="misaligned"):
            jc.fit_locale("ko", pairs, [0.5] * 10, jc.Policy("max_f1"))

    def test_full_fit_entry_shape(self):
        pairs = make_pairs(60, 60)
        scores = [0.9 - 0.005 * i for i in range(60)] + [0.1 + 0.005 * i for i in range(60)]
        entry = jc.fit_locale("ko", pairs, scores, jc.Policy("max_f1"))
        assert entry["n_good"] == 60 and entry["n_bad"] == 60
        assert 0.4 < entry["threshold"] < 0.6
        assert entry["metrics"]["f1"] == 1.0
        assert entry["degenerate"] is False
        assert entry["cv"]["folds_used"] == 5
        assert entry["good_stats"]["p50"] > entry["good_stats"]["p05"]


class TestGoodStats:
    def test_quantiles_ordered_and_bounded(self):
        stats = jc.good_scores_stats([float(i) for i in range(100)])
        assert 0 < stats["p05"] < stats["p50"] < stats["p95"] < 99
        assert stats["mean"] == pytest.approx(49.5)

    def test_single_value_has_zero_std(self):
        stats = jc.good_scores_stats([0.7])
        assert stats["std"] == 0.0 and stats["p50"] == 0.7


class TestCalibration:
    def make_doc(self, heads_sha: str = "a" * 64) -> dict:
        return jc.build_calibration_doc(
            {"ko": {"threshold": 0.5, "n_good": 100, "n_bad": 100,
                    "metrics": {"precision": 0.9, "recall": 0.9, "f1": 0.9},
                    "confusion": {"tp": 90, "fp": 10, "tn": 90, "fn": 10},
                    "degenerate": False,
                    "cv": {"folds": 5, "folds_used": 5, "f1_mean": 0.88, "f1_std": 0.02},
                    "good_stats": {"mean": 0.8, "std": 0.1, "p05": 0.6, "p50": 0.8, "p95": 0.95}}},
            jc.Policy("max_f1"), heads_sha, 200, 0.5, embedding_model="e5b")

    def test_decide_uses_score_at_least_threshold(self, tmp_path):
        path = tmp_path / "cal.json"
        path.write_text(json.dumps(self.make_doc()), encoding="utf-8")
        calibration = jc.Calibration.load(path)
        assert calibration.decide("ko", 0.5) is True
        assert calibration.decide("ko", 0.49) is False

    def test_unknown_locale_error_names_fitted_locales(self, tmp_path):
        path = tmp_path / "cal.json"
        path.write_text(json.dumps(self.make_doc()), encoding="utf-8")
        with pytest.raises(jc.UnknownLocaleError, match="'ko'"):
            jc.Calibration.load(path).entry("vi")

    def test_missing_file_error_is_actionable(self, tmp_path):
        with pytest.raises(jc.CalibrationLoadError, match="calibrate_judge"):
            jc.Calibration.load(tmp_path / "absent.json")

    def test_unsupported_format_rejected(self, tmp_path):
        path = tmp_path / "cal.json"
        path.write_text(json.dumps({"format": 99}), encoding="utf-8")
        with pytest.raises(jc.CalibrationLoadError, match="format"):
            jc.Calibration.load(path)

    def test_head_hash_mismatch_fails_loud(self, tmp_path):
        heads = tmp_path / "heads.pt"
        heads.write_bytes(b"weights-v3")
        with pytest.raises(jc.HeadMismatchError, match="Re-fit"):
            jc.verify_head_hash("b" * 64, heads)

    def test_head_hash_match_passes(self, tmp_path):
        heads = tmp_path / "heads.pt"
        heads.write_bytes(b"weights-v3")
        import hashlib
        jc.verify_head_hash(hashlib.sha256(b"weights-v3").hexdigest(), heads)


class TestPolicyParsing:
    def test_max_f1_has_no_minimum(self):
        assert jc.parse_policy("max_f1") == jc.Policy("max_f1", None)

    def test_min_precision_parses_value(self):
        assert jc.parse_policy("min_precision=0.95") == jc.Policy("min_precision", 0.95)

    def test_unknown_policy_rejected(self):
        with pytest.raises(ValueError, match="unknown policy"):
            jc.parse_policy("balanced")
