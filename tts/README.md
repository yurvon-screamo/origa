# Example TTS — IndexTTS-2.5 batch pipeline (#528)

Japanese TTS audio for all word-example sentences, generated locally with
[IndexTTS-2.5](https://huggingface.co/IndexTeam/IndexTTS-2.5) voice cloning
and shipped to the CDN as `examples/audio/{id:06d}.opus` (immutable).

## Pipeline

```text
phrases corpus ──build_ref_pool.py──▶ ref_pool/ (80 neutral voices)
                                            │
examples corpus (cdn/examples/data) ────────┤
                                            ▼
                                     gen_batch.py ──▶ wav (22 050 Hz, model-native)
                                            │
                                     convert_opus.sh (completeness gate)
                                            ▼
                                   opus 24 kbps → cdn/examples/audio/
                                            │
                                     scripts/deploy_cdn.py
```

## Voice assignment — a pinned contract, not a recomputable value

Rotation is `item_id % len(pool)`: deterministic **only relative to the exact
pool**. The production pool (80 voices, ordering included) is archived here as
`ref_pool_manifest.json`; `gen_batch.py` requires the pool's `manifest.json`
to exist and aborts otherwise.

**Regeneration rule**: the `examples/audio/` CDN keys are immutable. Re-running
with the *pinned* pool reproduces byte-similar audio (safe). Rebuilding the
pool via `build_ref_pool.py` over a changed phrases corpus produces a
*different* pool — every sentence would get a different voice, and re-uploading
under the same keys would split-brain edge caches (the PR #182 immutable
overwrite class). A re-voiced regeneration must move to content-addressed
keys (`examples/audio-v2/{hash}/…`), never overwrite.

## Environment

Tested on Intel Arc B390 (Panther Lake iGPU, unified memory), Python 3.12:

```text
torch        2.15.0.dev+xpu  (download.pytorch.org/whl/nightly/xpu)
torchaudio   matching nightly
indextts     pip install -e <index-tts repo>  (v2.5 checkpoints, ~5.2 GB)
transformers 4.47
soundfile    (required — see patch note)
```

**torchaudio patch**: torchcodec wheels target FFmpeg 4–6 while the reference
workstation runs FFmpeg 9, so `torchaudio.load/save` are monkey-patched to
soundfile inside `gen_batch.py`. Drop the patch if torchcodec catches up.

## Usage

All commands run from a work dir (`tts-demo/` below) that is **not** the repo;
scripts stay in the repo and are referenced by path.

```bash
# 0) work dir + checkpoints (once)
mkdir tts-demo && cd tts-demo
git clone https://github.com/index-tts/index-tts && pip install -e index-tts
# fetch IndexTTS-2.5 checkpoints into index-tts/checkpoints/ (~5.2 GB)

# 1) reference pool: 80 voices from the pinned manifest
python3 /path/to/repo/tts/build_ref_pool.py --out ref_pool
# (ref_pool/manifest.json must match /path/to/repo/tts/ref_pool_manifest.json
#  for regenerating the production voices; diff them if unsure)

# 2) corpus manifest (id + text from the CDN data chunks)
python3 - << 'EOF'
import json, pathlib
rows = []
for f in sorted(pathlib.Path('/path/to/repo/cdn/examples/data').glob('s*.json')):
    rows.extend(json.load(open(f)))
seen = {r['i']: r for r in rows}
items = [{"id": r['i'], "text": r['x']}
         for r in sorted(seen.values(), key=lambda x: x['i'])]
json.dump(items, open('full_corpus.json', 'w'), ensure_ascii=False)
EOF

# 3) generation — N shard workers (2 fit a 30 GB iGPU box; pair each with
#    systemd Restart=always, see "Failure model" below)
systemd-run --user --unit=tts-batch@0 --working-directory="$PWD" \
    <python> /path/to/repo/tts/gen_batch.py --shard 0 --total-shards 2
systemd-run --user --unit=tts-batch@1 --working-directory="$PWD" \
    <python> /path/to/repo/tts/gen_batch.py --shard 1 --total-shards 2

# 4) convert + completeness gate (fails loudly on a partial corpus)
bash /path/to/repo/tts/convert_opus.sh

# 5) deploy
cp opus_out/*.opus /path/to/repo/cdn/examples/audio/
cd /path/to/repo && scripts/.venv/bin/python scripts/deploy_cdn.py
```

## Failure model (root-caused via `leak_diag.py`)

- **OOM cascade**: exception unwinding does NOT free tensors a failed
  inference allocated. One OOM poisons every following item until a process
  restart. The except-handler in `gen_batch.py` runs
  `gc.collect() + torch.xpu.empty_cache() + synchronize()` to break the
  cascade — this single fix took sustained throughput from ~490 to ~700+
  files/hour (2 workers).
- **Circuit breaker**: 5 consecutive failures `exit(1)`; systemd
  `Restart=always` recovers with clean VRAM in ~60 s. Checkpoints make the
  restart free.
- **Scattered failures** (non-cascade) get one retry pass; if any remain the
  worker exits 1 — an incomplete corpus must never reach the immutable CDN
  prefix (a missing file 404s forever: the pitch/audio incident class).
  `convert_opus.sh` re-checks completeness as the final gate.
- **iGPU = unified memory**: "VRAM" is system RAM; per-worker footprint is
  ~10 GB (RSS + GPU). Two workers saturate a 30 GB box; a 4-worker plan OOMs.
- A finished worker does NOT retry its failures — re-run the same command
  (checkpoints skip the good clips and retry only the gaps).

## Checkpoint validity

A wav counts as done only when `soundfile.info()` reads a duration ≥ 0.5 s —
a file truncated by the OOM-killer (plausible size, broken tail) is retried,
not shipped.

## Numbers (Arc B390, 2 workers)

| Metric | Value |
| :--- | :--- |
| Corpus | 22 413 sentences |
| Generation | ~45 h wall (≈ 500–700 files/h) |
| Output | 296 MB opus (24 kbps mono, ~13 KB/file) |
| Median clip | 3.8 s |
