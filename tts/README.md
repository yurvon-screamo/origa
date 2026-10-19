# Example TTS — IndexTTS-2.5 batch pipeline (#528)

Japanese TTS audio for all word-example sentences, generated locally with
[IndexTTS-2.5](https://huggingface.co/IndexTeam/IndexTTS-2.5) voice cloning
and shipped to the CDN as `examples/audio/{id:06d}.opus` (immutable).

## Pipeline

```
phrases corpus ──build_ref_pool.py──▶ ref_pool/ (80 neutral voices)
                                            │
examples corpus (cdn/examples/data) ────────┤
                                            ▼
                                     gen_batch.py ──▶ wav (22 050 Hz)
                                            │
                                     convert_opus.sh
                                            ▼
                                   opus 24 kbps → cdn/examples/audio/
                                            │
                                     scripts/deploy_cdn.py
```

Voice assignment is deterministic: item `id % len(pool)` — the same sentence
always gets the same voice across regenerations.

## Environment

Tested on Intel Arc B390 (Panther Lake iGPU, unified memory), Python 3.12:

```
torch        2.15.0.dev+xpu  (nightly, download.pytorch.org/whl/nightly/xpu)
torchaudio   matching nightly
indextts     pip install -e <index-tts repo>  (v2.5 checkpoints in tts-demo/)
transformers 4.47
soundfile    (required — see patch note)
```

**torchaudio patch**: torchcodec wheels target FFmpeg 4–6 while the workstation
runs FFmpeg 9, so `torchaudio.load/save` are monkey-patched to soundfile inside
`gen_batch.py`. Drop the patch if torchcodec catches up.

## Usage

```bash
# 1) reference pool (80 voices, ~5 min)
python3 tts/build_ref_pool.py --out tts-demo/ref_pool

# 2) full-corpus manifest (id+text from the CDN data chunks)
python3 - << 'EOF'
import json, pathlib
rows = []
for f in sorted(pathlib.Path('cdn/examples/data').glob('s*.json')):
    rows.extend(json.load(open(f)))
seen = {r['i']: r for r in rows}
items = [{"id": r['i'], "text": r['x']} for r in sorted(seen.values(), key=lambda x: x['i'])]
json.dump(items, open('tts-demo/full_corpus.json', 'w'), ensure_ascii=False)
EOF

# 3) generation — N shard workers (2 fit a 30 GB iGPU box)
systemd-run --user --unit=tts-batch@0 --working-directory=tts-demo \
    <python> gen_batch.py --shard 0 --total-shards 2
systemd-run --user --unit=tts-batch@1 --working-directory=tts-demo \
    <python> gen_batch.py --shard 1 --total-shards 2

# 4) convert + deploy
tts/convert_opus.sh            # wav → opus 24 kbps into tts-demo/opus_out
cp tts-demo/opus_out/*.opus cdn/examples/audio/
scripts/.venv/bin/python scripts/deploy_cdn.py
```

## Operational notes (learned the hard way)

- **Checkpointing**: every wav is its own checkpoint (`skip if exists &&
  size > 1 KB`); restarts never lose progress. Failed items self-heal on the
  next worker restart.
- **OOM cascade fix**: exception unwinding does NOT free the tensors a failed
  inference allocated. Without cleanup, one OOM poisons every following item
  until a process restart. The except-handler in `gen_batch.py` runs
  `gc.collect() + torch.xpu.empty_cache() + synchronize()` to break the
  cascade — this single fix took sustained throughput from ~490 to ~700+
  files/hour (2 workers).
- **Circuit breaker**: 5 consecutive failures still exit(1); pair the worker
  with `systemd Restart=always` for a clean-VRAM recovery (~60 s).
- **iGPU = unified memory**: "VRAM" is system RAM; per-worker footprint is
  ~10 GB (RSS + GPU). Two workers saturate a 30 GB box; the 4-worker plan
  OOMs.
- **leak_diag.py**: per-iteration RSS/GPU/gc probe — proved the leak was
  OOM-debris, not a Python-side accumulation. Run it first when throughput
  degrades.

## Numbers (Arc B390, 2 workers)

| Metric | Value |
|---|---|
| Corpus | 22 413 sentences |
| Generation | ~45 h wall (≈ 500–700 files/h) |
| Output | 296 MB opus (24 kbps mono, ~13 KB/file) |
| Median clip | 3.8 s |
