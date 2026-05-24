# TranslateGemma 12B — Batch Translation of 158K Japanese Phrases

## Overview

Regenerate EN and RU translations for all 158,332 Japanese phrases using
TranslateGemma 12B via vLLM served in Docker on a remote A100 80GB GPU.

The translation script runs on your **local machine** and connects to the
vLLM API on the A100 host over the network.

## Architecture

```text
Local machine                          A100 host (10.2.11.6)
┌──────────────────────┐               ┌─────────────────────────────┐
│ translate_transla-   │  HTTP :8000   │  Docker container           │
│ tegemma.py           │──────────────▶│  vLLM + TranslateGemma 12B  │
│ (Python, openai,    │               │  ~24.5 GB BF16              │
│  tqdm)               │               │  A100 80 GB VRAM            │
└──────────────────────┘               └─────────────────────────────┘
```

## Prerequisites

- SSH access: `turbin_y@10.2.11.6`
- Docker installed on the A100 host
- Python 3.10+ with `openai` and `tqdm` on your local machine

## Step 1: Transfer phrase data to A100

```bash
scp -r cdn/phrases/data/ turbin_y@10.2.11.6:~/origa_translate/phrases/data/
```

## Step 2: Start Docker vLLM on A100

```bash
ssh turbin_y@10.2.11.6

# In tmux/screen (model download + load takes ~5 min):
docker run --gpus all \
  --ipc=host \
  -p 8000:8000 \
  -v ~/.cache/huggingface:/root/.cache/huggingface \
  vllm/vllm-openai:latest \
  --model Infomaniak-AI/vllm-translategemma-12b-it \
  --tensor-parallel-size 1 \
  --max-model-len 2048 \
  --dtype bfloat16 \
  --gpu-memory-utilization 0.9 \
  --host 0.0.0.0 \
  --port 8000
```

Wait until you see: `Uvicorn running on http://0.0.0.0:8000`

Verify from the A100 host:

```bash
curl http://localhost:8000/v1/models
```

## Step 3: Run translation from your LOCAL machine

```bash
python scripts/translate_translategemma.py \
  --input cdn/phrases/data \
  --output output_phrases \
  --workers 50 \
  --languages en,ru \
  --api-url http://10.2.11.6:8000/v1
```

## Step 4: Monitor progress

```bash
# Output files count:
ls output_phrases/p*.json | wc -l

# Checkpoint:
cat checkpoint_translate.json | python -m json.tool | head -5

# GPU usage on A100:
ssh turbin_y@10.2.11.6 nvidia-smi
```

## Step 5: Retrieve results

```bash
scp turbin_y@10.2.11.6:~/origa_translate/output/p*.json cdn/phrases/data/
```

Or, if running locally with `--output` pointing to a local dir, results are
already on your machine.

## Resume after interruption

Just re-run the same command — the checkpoint file tracks completed chunks:

```bash
python scripts/translate_translategemma.py \
  --input cdn/phrases/data \
  --output output_phrases \
  --workers 50 \
  --languages en,ru \
  --api-url http://10.2.11.6:8000/v1
```

If the Docker container died, restart it (Step 2) first. The checkpoint
ensures no already-translated file is reprocessed.

## Performance estimates

- Model: TranslateGemma 12B (~24.5 GB BF16 on A100)
- Workers: 50 concurrent requests
- Speed: ~4-8 phrases/sec on A100 with vLLM (12B is slower than 4B)
- Total time: ~7-10 hours for 158K phrases × 2 languages
- VRAM: ~28-32 GB (model + KV cache, fits comfortably on 80 GB A100)

## Troubleshooting

**vLLM OOM in Docker**: Reduce `--gpu-memory-utilization` to `0.8`, or add
`--enforce-eager` to skip CUDA graph capture.

**Docker container exits**: Check `docker logs <container_id>`. Common causes
include missing GPU driver or insufficient VRAM.

**Connection refused from local machine**: Ensure Docker publishes port 8000
(`-p 8000:8000`) and the A100 firewall allows inbound TCP on 8000.

**Slow translation**: Try increasing `--workers` to 100. vLLM handles request
queuing and batching internally.

**Model not downloading**: The first Docker run downloads ~24.5 GB. Use the
HuggingFace cache volume (`-v ~/.cache/huggingface:/root/.cache/huggingface`)
so subsequent runs start faster.
