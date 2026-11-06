# Judge d1-2B — training, evaluation, and prod scanning

Translation quality judge based on [LiquidAI/d1-2B](https://huggingface.co/LiquidAI/d1-2B)
decision model. Answers `noul` (yes/no probability) questions about translation faithfulness.

## Architecture

```python
LiquidAI/d1-2B (LFM3-VL backbone)
  + LoRA r16 α32 (24M trainable) ← judge adapter
  → RAW slot-readout: p(Yes) at answer position
```python

**Important:** use RAW slot-readout (not `system_one` API) — the stock
`calibration.json` in d1-2B distorts readouts for adapted models. All evaluation
and serving code here uses the RAW path.

## Files

| File | Purpose |
|------|---------|
| `train_d1.py` | Core trainer: v5 dataset (noul-faithful pairs), LoRA SFT |
| `train_d1_v7.py` | v7 trainer: v5 + prodfix pairs from prod cleanup |
| `eval_d1_faithful_raw.py` | golden_v4c evaluation (RAW readout, pairwise accuracy) |
| `eval_d1_tier_raw.py` | Tier (score) question evaluation (RAW readout) |
| `build_judge_v62.py` | Dataset builder: v5 pairs + tier block from swarm consensus |
| `aggregate_consensus.py` | Swarm consensus aggregator (5-frame voting → gold/silver/weak) |
| `scan_prod.py` | Full prod scan (781k items, batched RAW readout, XPU) |

## Model versions

| Version | golden_v4c | zh-golden | Notes |
|---------|-----------|-----------|-------|
| v3 (e5+InfoNCE) | 81.9% | — | prod judge, 4 locales only |
| d1-60k | 83.5% | — | best faithful, no zh |
| **v7** | **77.6%** | **100%** | **current: v5+prodfix, 6 locales** |

## Training

```bash
# v5 (defect-only, 6 locales)
python train_d1.py  # ~8h on Intel Arc B390 XPU

# v7 (v5 + prodfix from cleanup)
python train_d1_v7.py  # ~8h
```python

XPU requirements: bs4 × accum4, length-bucketing, `empty_cache` every 20 steps,
`use_cache=False` (DEVICE_LOST fix). See trainer code for details.

## Evaluation

```bash
# faithful (noul yes/no) — pairwise accuracy on golden_v4c
python eval_d1_faithful_raw.py <adapter_dir>

# zh — same but on golden_zh_v1
D1_GOLDEN=.../golden_zh_v1.jsonl python eval_d1_faithful_raw.py <adapter_dir>
```python

## Prod scanning

```bash
python scan_prod.py  # scans all cdn phrases × 5 locales (~14h)
```python

Calibrated per-locale thresholds (F1-balanced on golden):
ru 0.80 · en 0.80 · vi 0.85 · ko 0.90 · zh 0.98

## Known issues

- Stock `calibration.json` in d1-2B distorts adapted readouts → always use RAW path
- `d1pkg` (remote code) wipes `sys.environ` on import → read env vars before import
- `system_one` API applies calibration → bypass with direct backbone+head readout
- One LoRA adapter for faithful + tier trades off quality (5-8pp per added skill)
