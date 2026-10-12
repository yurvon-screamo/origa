---
license: other
license_name: bsl-1.1
license_link: https://github.com/yurvon-screamo/origa/blob/master/LICENSE
base_model:
- intfloat/multilingual-e5-base
language:
- ja
- en
- ru
- ko
- vi
library_name: peft
tags:
- quality-estimation
- translation-ranking
- infonce
- japanese
---

# Origa Judge v3 — translation quality ranker

InfoNCE heads over `intfloat/multilingual-e5-base` (mean-pooling, `query:`/`passage:` prefixes) that score JA→X translation pairs for the Origa content pipeline.

**This is a ranker, not a grader.** It compares candidates against a reference (Best-of-N selection). Absolute scores are not calibrated across locales — do not use a raw score as a pass/fail threshold. Per-locale calibration is tracked in [issue #713](https://github.com/yurvon-screamo/origa/issues/713).

Part of the [Origa](https://github.com/yurvon-screamo/origa) translation stack.

## Results (internal eval)

- Test: `golden_v4c` — 662 pairs, 7 audit iterations. Score: 542 (81.9%). Control set v7c: 547 (82.6%), parity.
- Known false-negative class: phonetic loanwords. Production scans therefore pair the score with a per-locale percentile and an independent algorithmic flag (double confirmation), never a raw cutoff.

## Training data

59,973 pairs:

| Component | Pairs |
| --- | --- |
| Synthetic negatives | 35,000 |
| Metadata-junk negatives | 10,300 |
| Inversions | 6,100 |
| Deep pipeline errors (margin < −0.50, 75% real) | 2,200 |
| Paraphrase positives (double-filtered) | 5,900 |
| Audit pairs | 500 |

Training: 5 epochs, ~5 minutes on vLLM-produced embeddings. InfoNCE heads held parity on a control set even with 70% noisy negatives: noise cleaning matters for interpretability, not for the score.

## Usage

Serve the encoder through vLLM pooling:

```bash
vllm serve intfloat/multilingual-e5-base --served-model-name e5b \
  --runner pooling --port 8090 --gpu-memory-utilization 0.45 \
  --max-num-seqs 256 --attention-backend TRITON_ATTN \
  --pooler-config '{"pooling_type":"MEAN"}'
```

`MEAN` pooling is required; the vLLM default (LAST) breaks head compatibility. The default `FLASH_ATTN` backend fails on this model; use `TRITON_ATTN`.

Call the heads through the typed-question API (noul/choice/score), which handles rank calibration and the context-first layout:

[scripts/judge_api.py](https://github.com/yurvon-screamo/origa/blob/master/translator/scripts/judge_api.py)

Standalone scanners that pair this judge with the double-confirmation rule:

[scan_phrases_v3.py](https://github.com/yurvon-screamo/origa/blob/master/translator/scripts/scan_phrases_v3.py) · [scan_dictionary_v3.py](https://github.com/yurvon-screamo/origa/blob/master/translator/scripts/scan_dictionary_v3.py)

## Production record

One cleanup pass over the live Origa CDN with this judge in the loop: 2,066 phrase fixes and 1,693 dictionary fixes (Japanese insertions, wrong meanings, hallucinated senses, wrong-language entries), each fix double-confirmed and regression-checked.

## Limitations

- Ranking-only; no calibrated absolute thresholds (see #713).
- False negatives on phonetic loanwords.
- Trained and evaluated on ja→{en,ru,ko,vi} pairs from one domain (learning content). Out-of-domain behavior is untested.

## License

BSL 1.1 ([text](https://github.com/yurvon-screamo/origa/blob/master/LICENSE)). Base encoder: MIT (per its model card).
