# Origa Translator

[English](README.md) | [Русский](README.ru.md) | [한국어](README.ko.md) | [Tiếng Việt](README.vi.md)

Translation stack for the Origa content pipeline: a JA→{EN,RU,KO,VI} translator, a quality judge, and the production cleanup pipeline that ran against the live CDN.

This is the code and evaluation harness. Model weights live on Hugging Face; training corpora are internal and not released.

## Components

**Translator — `ep1`** ([HF: origa-translator-ep1](https://huggingface.co/yurvon-screamo/origa-translator-ep1))
LoRA adapter (r=16 α=32, 24.16M trainable, 205 text-stack modules) over `google/gemma-4-E2B-it-qat-w4a16-ct`. Trained for 1 epoch on 600k phrase pairs (150k × 4 locales), val_loss 0.6720.

**Judge — `v3`** ([HF: origa-judge-v3](https://huggingface.co/yurvon-screamo/origa-judge-v3))
`intfloat/multilingual-e5-base` (mean-pooling) + InfoNCE heads. Trained on 59,973 pairs. Internal eval: 81.9% on golden_v4c (662 pairs, 7 audit iterations). The judge ranks candidates (Best-of-N); its absolute scores are not calibrated per locale — see [issue #713](https://github.com/yurvon-screamo/origa/issues/713).

**Cleanup pipeline** (completed run)
Scan with the judge → double confirmation (per-locale percentile × algorithmic flag) → subagent rewrite → per-record revision → judge regression → CDN patch → deploy + HTTP verify. One production pass fixed 2,066 phrase records and 1,693 dictionary records.

## Quality numbers

All numbers below are *internal evals*: manual review by the project author, not an independent benchmark. Treat them as directional.

| Check | Result |
| --- | --- |
| Manual review, 200 phrases (50 × 4 locales) | 83% full / 99.5% usable |
| Language routing, 200 phrases | 200/200 first try |
| Zero-shot locales not in training (my/bm/th), 24 phrases | 24/24 usable (base model without adapter: 0/24) |
| Judge on golden_v4c | 542/662 (81.9%) |

Per-locale breakdown and error taxonomy: [docs/REPORT_AB_6MODELS.md](docs/REPORT_AB_6MODELS.md). chrF comparisons across 6 candidate models are in the same report; the final stack was chosen over all of them.

## Quickstart

Serve the adapter with vLLM:

```bash
vllm serve google/gemma-4-E2B-it-qat-w4a16-ct \
  --enable-lora --lora-modules tr600k=<path-to-ep1> \
  --max-lora-rank 16
```

Or load with transformers (details matter on the QAT base, see gotchas):

```python
import torch
from transformers import AutoModelForCausalLM, AutoTokenizer, CompressedTensorsConfig
from peft import PeftModel

BASE = "google/gemma-4-E2B-it-qat-w4a16-ct"
model = AutoModelForCausalLM.from_pretrained(
    BASE, dtype=torch.bfloat16, low_cpu_mem_usage=False,
    quantization_config=CompressedTensorsConfig(run_compressed=False),
).to("xpu")
model = PeftModel.from_pretrained(model, "yurvon-screamo/origa-translator-ep1").eval()
tok = AutoTokenizer.from_pretrained(BASE)
```

Without `low_cpu_mem_usage=False` + `CompressedTensorsConfig(run_compressed=False)` the layers load as skeletons without packed weights and PEFT fails.

Judge inference: serve `intfloat/multilingual-e5-base` through vLLM pooling with `--pooler-config '{"pooling_type":"MEAN"}'` (the default LAST pooling breaks head compatibility) and `--attention-backend TRITON_ATTN`. Call it through `scripts/judge_api.py`, a typed-question API (noul/choice/score) over the heads.

## Scripts

| Script | What it does |
| --- | --- |
| `build_datasets.py` | Assemble SFT pools from source data |
| `train_sft_qat.py` | QAT-native LoRA trainer (see recipe below) |
| `run_600k.sh` | Full 600k run with checkpointing and auto-resume |
| `eval_wrapper.py` | Single-shot generation + language detection + retry |
| `eval_model.py` / `bench_train_qat.py` | Model eval and training benchmark |
| `judge_api.py` | Typed-question API over judge heads |
| `train_judge_heads.py` | Judge head trainer (consumes vLLM embeddings) |
| `build_judge_v3.py` | Judge dataset assembly (v3) |
| `pipeline_translate_judge.py` | End-to-end: translate → checks → judge → margin |
| `scan_phrases_v3.py` / `scan_dictionary_v3.py` | Production scans of CDN phrases/dictionary |

Scripts read locations from environment variables (all optional, defaults are repo-relative):

| Variable | Purpose |
| --- | --- |
| `ORIGA_DATA_DIR` | Training data root |
| `ORIGA_RUNS_DIR`, `ORIGA_WORK_DIR` | Checkpoints and scratch space |
| `ORIGA_JUDGE_HEADS` | Path to `judge_heads_v3.pt` |
| `ORIGA_JUDGE_DATA` | Judge datasets |
| `ORIGA_PIPELINE_OUT` | Cleanup pipeline output |
| `ORIGA_CDN_PHRASES`, `ORIGA_CDN_DICTIONARY` | Live CDN content trees |

## Training recipe (QAT base)

1. Load the QAT base with `CompressedTensorsConfig(run_compressed=False)` (alias `dequantize=True`) + `low_cpu_mem_usage=False`. This dequantizes packed int4 into real bf16 `nn.Linear` layers with the original QAT weight values. Do not stack bitsandbytes on top.
2. Collect LoRA targets from the actual module tree (`Linear` + non-None `weight`, text stack only — skip vision/audio/multimodal): 205 modules on this architecture. `target_modules="all-linear"` crashes on the custom clippable layers.
3. QAT-native training is 2.4× faster than loading the base through bitsandbytes-NF4, and reaches the same quality in our runs.
4. Batch generation with left-padding on Intel XPU distorts output; all final evals ran single-shot (batch=1). vLLM batching is safe.

The full post-mortem, including infra fixes (OOM cgroup cage, masked CE for the 262k vocab, XPU allocator fragmentation), is in [docs/REPORT_AB_6MODELS.md](docs/REPORT_AB_6MODELS.md).

## Known limitations

- The judge ranks, it does not grade. Absolute thresholds are not calibrated per locale (evidence for KO and VI in [#713](https://github.com/yurvon-screamo/origa/issues/713)). Best-of-N is the supported usage.
- `pipeline_translate_judge.py` still points at judge heads v2 by default (historical); the standalone scanners (`scan_phrases_v3.py`, `scan_dictionary_v3.py`) use v3. This is intentional, described as is, not yet unified.
- The judge produces false negatives on phonetic loanwords; production scans therefore require the double-confirmation rule (percentile × algorithmic flag), never a raw score cutoff.
- The translator keeps residual 🟡 issues on VI (register/politeness shifts) and rare EN inversions. We assessed these as a capability limit of the 2B base; more data did not remove them.
- Intel XPU only for training; inference runs anywhere vLLM supports the base.

## License

- Code in this directory: BSL 1.1, same as the rest of the repository. See the root [LICENSE](../LICENSE).
- Base model `google/gemma-4-E2B-it-qat-w4a16-ct`: Apache 2.0 (per its Hugging Face model card).
- Training corpora: derived from Origa product content, internal and not released.
