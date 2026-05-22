# TranslateGemma 4B — Batch Translation of 158K Japanese Phrases

## Overview

Regenerate EN and RU translations for all 158,332 Japanese phrases using TranslateGemma 4B via vLLM on A100 80GB.

## Prerequisites

- SSH access: `turbin_y@10.2.11.6`
- Python 3.10+
- vLLM installed: `pip install vllm openai`
- A100 GPU with 80GB VRAM

## Step 1: Transfer files to A100

```bash
# From your local machine:
scp scripts/translate_translategemma.py turbin_y@10.2.11.6:~/origa_translate/
scp -r cdn/phrases/data/ turbin_y@10.2.11.6:~/origa_translate/phrases/data/
```

## Step 2: Install dependencies on A100

```bash
ssh turbin_y@10.2.11.6
pip install vllm openai tqdm
```

## Step 3: Start vLLM server

```bash
# In tmux/screen (this takes ~2 min to load model):
vllm serve Infomaniak-AI/vllm-translategemma-4b-it \
  --tensor-parallel-size 1 \
  --max-model-len 2048 \
  --dtype bfloat16 \
  --gpu-memory-utilization 0.9
```

Wait until you see: `Uvicorn running on http://0.0.0.0:8000`

Verify it works:

```bash
curl http://localhost:8000/v1/completions \
  -H "Content-Type: application/json" \
  -d '{"model":"Infomaniak-AI/vllm-translategemma-4b-it","prompt":"<<<source>>>ja<<<target>>>ru<<<text>>>こんにちは","max_tokens":50,"temperature":0.0}'
```

## Step 4: Run translation

```bash
# In another tmux/screen:
cd ~/origa_translate

python translate_translategemma.py \
  --input phrases/data \
  --output output \
  --workers 50 \
  --languages en,ru
```

## Step 5: Monitor progress

```bash
# Check output files:
ls output/p*.json | wc -l

# Check checkpoint:
cat checkpoint_translate.json | python -m json.tool | head -5

# GPU usage:
nvidia-smi
```

## Step 6: Retrieve results

```bash
# From your local machine:
scp turbin_y@10.2.11.6:~/origa_translate/output/p*.json cdn/phrases/data/
```

## Resume after interruption

Just re-run the same command — checkpoint tracks completed files:

```bash
python translate_translategemma.py \
  --input phrases/data --output output --workers 50 --languages en,ru
```

## Performance estimates

- Model: TranslateGemma 4B (~5GB BF16 on A100)
- Workers: 50 concurrent requests
- Speed: ~5-10 phrases/sec on A100 with vLLM
- Total time: ~4-8 hours for 158K phrases
- VRAM: ~8-10 GB (leaves room for KV cache)

## Troubleshooting

**vLLM OOM**: Reduce `--gpu-memory-utilization` to 0.8, or reduce `--workers` to 20.

**Connection refused**: Make sure vLLM server is running on port 8000.

**Slow performance**: Increase `--workers` to 100. vLLM handles queuing internally.
