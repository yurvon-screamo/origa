# Labeling guide — judge calibration (KO / VI)

Native-speaker labeling builds the per-locale thresholds that turn the judge
from a ranker into a grader ([issue #713](https://github.com/yurvon-screamo/origa/issues/713)).
Target: ~200 labeled pairs per locale, ~400 pairs total.

## What gets labeled

One task = one pair:

- **JP** — the source phrase.
- **Translation** — a candidate KO or VI translation produced by the pipeline.

The operator creates tasks with:

```bash
python scripts/calibrate_judge.py export --pool pool.jsonl --locales ko,vi \
    --n 200 --out task_ko_vi.jsonl --md task_ko_vi.md
```

`pool.jsonl` is a plain random slice of the production flow of the current
translator version (rows: `{"locale", "state", "translation"}`).

**Do not build the pool to be balanced** (half good / half bad by
construction). The sample must reflect production as-is: a balanced pool
distorts the score-conditional class rates, and the fitted threshold stops
matching production prevalence. The good/bad split of the *labeled* result is
a reported number, not a goal.

## How to mark

For each pair, mark **good** or **bad**:

- **good** — the translation preserves the meaning and reads naturally.
- **bad** — the meaning changes (addressee, polarity, added/dropped content),
  the register is broken, the text is in the wrong language, or it is an
  untranslated echo of the Japanese.

KO-specific attention point: **phonetic loanwords** (음차 외래어, e.g.
커피, 아르바이트) are a known judge blind spot — it under-scores valid ones.
Make sure the KO sample includes its natural share of them, and label them by
the same meaning+naturalness rule, not by whether they look "safe".

## How labels come back

The markdown file is a one-way human view. The JSONL task file is the source
of truth: the operator copies each mark into the `"label"` field of the
matching row (`"good"` or `"bad"`), then validates:

```bash
python scripts/calibrate_judge.py import --labels task_ko_vi.jsonl
```

Import checks schema, allowed labels and duplicate ids, reports per-locale
class counts, and warns about locales outside en/ru/ko/vi.

## Fitting thresholds

```bash
# requires the e5b embeddings server on :8090 (MEAN pooling, TRITON_ATTN)
# and ORIGA_JUDGE_HEADS pointing to the exact origa-judge.pt used in production
python scripts/calibrate_judge.py fit --labels task_ko_vi.jsonl \
    --seed <seed from export> --pool-rows <pool size from export>
```

- The sample must be scored against the **same `.pt` bytes** that runtime
  loads (`heads_sha256` in the output is a byte hash; a HF re-publish changes
  the bytes and the runtime will refuse the stale file).
- Fit and runtime must use the same encoder serving recipe (MEAN pooling,
  TRITON_ATTN) — the LAST-pooling default silently breaks head compatibility.
- Default policy is max-F1; for pipelines that need a precision floor use
  `--policy min_precision=0.95` (or `min_recall=…`). An infeasible policy
  fails with the best achievable value instead of writing a wrong file.
- A locale needs ≥40 pairs and ≥8 per class to fit; cross-validation stats
  appear only at ≥25 per class.
- The tool writes `judge_calibration.json`; `Judge.verdict()` verifies the
  heads hash at load and fails loudly on mismatch.

## Checklist before sending tasks to natives

- [ ] Pool is a fresh random slice of production flow (not balanced, not
      cherry-picked suspects — suspect-only pools overstate the bad rate).
- [ ] KO sample includes natural share of phonetic loanwords.
- [ ] `--seed` and pool size recorded for `fit --seed/--pool-rows`.
- [ ] IDs are untouched (they tie labels back to pairs).
