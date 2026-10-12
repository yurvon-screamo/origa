---
license: other
license_name: bsl-1.1
license_link: https://github.com/yurvon-screamo/origa/blob/master/LICENSE
base_model:
- google/gemma-4-E2B-it-qat-w4a16-ct
language:
- ja
- en
- ru
- ko
- vi
library_name: peft
tags:
- translation
- lora
- qat
- japanese
- sft
---

# Origa Translator ep1 — JA→{EN,RU,KO,VI} LoRA adapter

LoRA adapter (r=16 α=32, 24.16M trainable parameters, 205 text-stack modules) on top of `google/gemma-4-E2B-it-qat-w4a16-ct`. Trained for 1 epoch on 600k phrase pairs (150k per locale) extracted from Origa's Japanese learning content.

Part of the [Origa](https://github.com/yurvon-screamo/origa) translation stack: this adapter plus a retrieval judge for candidate ranking, plus a production cleanup pipeline.

## Results (internal eval)

Manual review by the project author, 200 phrases (50 × 4 locales). Not an independent benchmark.

| Metric | Value |
| --- | --- |
| Full translations | 83% (166/200) |
| Usable translations (full + partial) | 99.5% |
| Language routing (target locale hit) | 200/200 first try |
| Zero-shot locales not in training (my, bm, th), 24 phrases | 24/24 usable; base model without the adapter: 0/24 |

Single failed case out of 200: an English possessive inversion, present since the 40k checkpoint. We assessed it as a capability limit of the 2B base that did not go away with more data.

Per-locale breakdown, chrF comparison against 6 candidate base models, and the error taxonomy: [A/B report](https://github.com/yurvon-screamo/origa/blob/master/translator/docs/REPORT_AB_6MODELS.md).

## Usage

Serve with vLLM:

```bash
vllm serve google/gemma-4-E2B-it-qat-w4a16-ct \
  --enable-lora --lora-modules tr600k=yurvon-screamo/origa-translator-ep1 \
  --max-lora-rank 16
```

Or transformers. Two flags matter when loading the QAT base; without them the layers load as skeletons and PEFT fails:

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

Generation notes:

- Greedy decoding; `enable_thinking=False` always.
- Single-shot generation (batch=1) on Intel XPU: batched generation with left-padding distorts output there. vLLM batching is safe.
- Prompt format: an English instruction naming the target locale, then the Japanese source. JA instruction as fallback for retry. See `build_prompt` in [eval_wrapper.py](https://github.com/yurvon-screamo/origa/blob/master/translator/scripts/eval_wrapper.py).

## Training

- Data: 600k JA→X pairs (150k × 4 locales) from Origa's cleaned content corpus. **Internal corpus, not released** — it is derived from product content.
- 1 epoch, 18,750 steps, val_loss 0.6720 (40k subset: 0.9505 — no plateau).
- QAT-native: the base loads dequantized to bf16 via `CompressedTensorsConfig`, LoRA trains directly on the QAT weight values. 2.4× faster than bitsandbytes-NF4 at equal quality in our runs.
- Hardware: Intel Arc B390 (XPU), 21.2 hours, zero crashes with checkpoint-every-50k + auto-resume.

## Limitations

- Residual partial-translation cases on VI (register/politeness shifts between thân/mình/bạn) and rare EN possessive inversions. We assessed these as a capability limit of the 2B base; scaling data did not remove them.
- Zero-shot locales not in training produce usable but unaudited output.
- This adapter is a content-pipeline component, not a general-purpose MT system.

## License

Adapter and code: BSL 1.1 ([text](https://github.com/yurvon-screamo/origa/blob/master/LICENSE)). Base model: Apache 2.0. Training corpus: not released.
