#!/usr/bin/env python3
"""Batch IndexTTS-2.5 generation: shard worker with checkpoint/resume.

Usage:
    gen_batch.py --shard 0 --total-shards 4 --limit 25   # test mode
    gen_batch.py --shard 0 --total-shards 4               # full run

Each worker:
    - loads the full corpus, takes its slice (round-robin by index)
    - skips files that already exist (checkpoint/resume)
    - writes 24kHz wav files to out/batch/shard_<n>/
    - logs progress every 10 items
"""
import argparse
import json
import os
import sys
import time

os.environ.setdefault("HF_HOME", os.path.expanduser("~/.cache/huggingface"))

# ── torchaudio patch: soundfile instead of torchcodec (FFmpeg 9 incompatible) ──
import soundfile as sf_lib
import torch
import torchaudio


def _sf_load(filepath, *args, **kwargs):
    data, sr = sf_lib.read(filepath, dtype="float32", always_2d=True)
    return torch.from_numpy(data).T, sr


def _sf_save(filepath, src, sample_rate, *args, **kwargs):
    arr = src.detach().cpu().numpy()
    if arr.ndim == 2:
        arr = arr.T
    sf_lib.write(filepath, arr, sample_rate)


torchaudio.load = _sf_load
torchaudio.save = _sf_save
# ──────────────────────────────────────────────────────────────────────────────

from indextts.infer_v2_5 import IndexTTS2  # noqa: E402

BASE = os.path.dirname(os.path.abspath(__file__))
REF_POOL_DIR = os.path.join(BASE, "ref_pool")
# Один реф (fallback, если пула нет): item_id всегда получает один и тот же голос
REF_SINGLE = os.path.join(BASE, "ref1_24k.wav")


def load_ref_pool():
    """Deterministic ref per item: item_id % pool_size."""
    import glob as _glob
    wavs = sorted(_glob.glob(os.path.join(REF_POOL_DIR, "*.wav")))
    return wavs if wavs else [REF_SINGLE]


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--shard", type=int, required=True)
    parser.add_argument("--total-shards", type=int, default=4)
    parser.add_argument("--limit", type=int, default=0, help="0 = no limit (full)")
    parser.add_argument("--ref", default=REF_SINGLE)
    args = parser.parse_args()

    corpus = json.load(open(os.path.join(BASE, "full_corpus.json")))

    # Round-robin slice: shard k gets items k, k+T, k+2T, ...
    items = corpus[args.shard :: args.total_shards]
    if args.limit > 0:
        items = items[: args.limit]

    out_dir = os.path.join(BASE, "out", "batch", f"shard_{args.shard}")
    os.makedirs(out_dir, exist_ok=True)

    ref_pool = load_ref_pool()
    print(f"[shard {args.shard}] {len(items)} items, ref_pool={len(ref_pool)} refs", flush=True)

    tts = IndexTTS2(
        cfg_path=os.path.join(BASE, "index-tts-src/checkpoints/config.yaml"),
        model_dir=os.path.join(BASE, "index-tts-src/checkpoints"),
        use_bf16=True,
        device="xpu",
    )
    print(f"[shard {args.shard}] model loaded, VRAM={torch.xpu.memory_allocated()/1e9:.1f}G", flush=True)

    done = skip = fail = 0
    consecutive_fails = 0
    t_start = time.perf_counter()

    for i, item in enumerate(items):
        fn = os.path.join(out_dir, f"{item['id']:06d}.wav")

        # Checkpoint: skip existing
        if os.path.exists(fn) and os.path.getsize(fn) > 1000:
            skip += 1
            continue

        try:
            tts.infer(
                spk_audio_prompt=ref_pool[item["id"] % len(ref_pool)],
                text=item["text"],
                lang="JA",
                output_path=fn,
            )
            done += 1
            consecutive_fails = 0
        except Exception as e:
            fail += 1
            consecutive_fails += 1
            print(f"[shard {args.shard}] FAIL id={item['id']}: {e}", flush=True)
            # Remove partial file
            if os.path.exists(fn):
                os.remove(fn)
            # DIAG FIX: OOM-спайк оставляет тензоры после unwinding →
            # чистим СРАЗУ, разрывая каскад (следующий item стартует чистым)
            import gc
            gc.collect()
            torch.xpu.empty_cache()
            torch.xpu.synchronize()
            # Circuit breaker: VRAM leak → clean restart beats zombie-loop
            if consecutive_fails >= 5:
                print(f"[shard {args.shard}] 5 consecutive fails — restarting worker", flush=True)
                sys.exit(1)

        # Anti-fragmentation on long runs: release cached VRAM blocks
        if (done + skip) > 0 and (done + skip) % 20 == 0:
            import gc
            gc.collect()
            torch.xpu.empty_cache()

        if (done + skip) % 10 == 0:
            elapsed = time.perf_counter() - t_start
            rate = (done + skip) / elapsed if elapsed > 0 else 0
            eta_min = (len(items) - done - skip) / rate / 60 if rate > 0 else 0
            print(
                f"[shard {args.shard}] {done+skip}/{len(items)} "
                f"(done={done} skip={skip} fail={fail}) "
                f"rate={rate*60:.0f}/min eta={eta_min:.0f}min",
                flush=True,
            )

    elapsed = time.perf_counter() - t_start
    print(
        f"[shard {args.shard}] FINISHED: done={done} skip={skip} fail={fail} "
        f"in {elapsed/60:.1f}min",
        flush=True,
    )


if __name__ == "__main__":
    main()
