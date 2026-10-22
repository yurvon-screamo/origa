#!/usr/bin/env python3
"""IndexTTS-2.5 shard worker: checkpoint/resume, ref-pool rotation, OOM-hardened.

Runs from a work directory that holds the corpus manifest, the reference
pool (``ref_pool/`` from ``build_ref_pool.py``) and the model checkpoints
(see tts/README.md for the full recipe). Each worker takes a round-robin
slice of the corpus, skips completed clips, and writes 22 050 Hz wav files
(``{id:06d}.wav``) under ``--out``.

Exit codes:
    0 — every item in the shard has a valid clip (incl. the final retry pass)
    1 — some items still failing after the retry pass, OR a broken pool

Failure model (iGPU/unified-memory, root-caused via leak_diag.py):
exception unwinding does NOT free tensors a failed inference allocated —
one OOM poisons every following item until the process restarts. The
except-handler therefore runs ``gc.collect() + empty_cache() +
synchronize()`` to break the cascade, and 5 consecutive failures exit(1)
so a systemd ``Restart=always`` unit recovers with clean VRAM (~60 s).
Scattered non-cascade failures get one full retry pass at the end; if any
remain, the worker exits 1 — an operator signal, NOT a deployable state
(the corpus must be complete before the immutable CDN upload).
"""

from __future__ import annotations

import argparse
import gc
import json
import os
import sys
import time

os.environ.setdefault("HF_HOME", os.path.expanduser("~/.cache/huggingface"))

# torchaudio load/save via soundfile: torchcodec wheels target FFmpeg 4–6
# while the reference workstation runs FFmpeg 9 (see tts/README.md).
import soundfile as sf_lib  # noqa: E402
import torch  # noqa: E402
import torchaudio  # noqa: E402


def _sf_load(filepath: str, *args, **kwargs):
    data, sr = sf_lib.read(filepath, dtype="float32", always_2d=True)
    return torch.from_numpy(data).T, sr


def _sf_save(filepath: str, src, sample_rate: float, *args, **kwargs) -> None:
    arr = src.detach().cpu().numpy()
    if arr.ndim == 2:
        arr = arr.T
    sf_lib.write(filepath, arr, sample_rate)


torchaudio.load = _sf_load  # type: ignore[assignment]
torchaudio.save = _sf_save  # type: ignore[assignment]

from indextts.infer_v2_5 import IndexTTS2  # noqa: E402

# A clip shorter than this is treated as corrupt (killed mid-write by the
# OOM-killer — a plain size check cannot tell a truncated wav from a real
# one; duration metadata can).
MIN_VALID_SECONDS = 0.5


def is_valid_clip(path: str) -> bool:
    """Checkpoint test: exists, non-trivial, readable, plausible duration."""
    try:
        if os.path.getsize(path) <= 1000:
            return False
        return sf_lib.info(path).duration >= MIN_VALID_SECONDS
    except (OSError, RuntimeError):
        return False


def load_ref_pool(pool_dir: str) -> list[str]:
    """Reference wavs in manifest order — the pool is a pinned contract.

    Rotation is ``item_id % len(pool)``; the exact pool (ordering AND
    members) is archived in the repo as ``tts/ref_pool_manifest.json``.
    Changing the pool changes the voice of essentially every sentence, and
    the CDN keys are immutable — so regeneration must either reuse the
    pinned pool or move to content-addressed keys (never overwrite).
    Missing pool is therefore a hard abort, not a fallback.
    """
    manifest_path = os.path.join(pool_dir, "manifest.json")
    if not os.path.isfile(manifest_path):
        sys.exit(
            f"ref pool manifest not found: {manifest_path}\n"
            "Build it with tts/build_ref_pool.py (see tts/README.md); "
            "regenerating with a DIFFERENT pool must not overwrite the "
            "immutable examples/audio/ keys."
        )
    manifest = json.load(open(manifest_path, encoding="utf-8"))
    wavs = [os.path.join(pool_dir, f"{entry['idx']:03d}.wav") for entry in manifest]
    missing = [w for w in wavs if not os.path.isfile(w)]
    if missing:
        sys.exit(f"ref pool incomplete: {len(missing)} wavs missing, e.g. {missing[0]}")
    return wavs


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    parser.add_argument("--shard", type=int, required=True)
    parser.add_argument("--total-shards", type=int, default=2)
    parser.add_argument("--limit", type=int, default=0, help="0 = full slice")
    parser.add_argument("--corpus", default="full_corpus.json")
    parser.add_argument("--pool", default="ref_pool")
    parser.add_argument(
        "--checkpoints",
        default="index-tts-src/checkpoints",
        help="IndexTTS-2.5 checkpoints dir (config.yaml + weights)",
    )
    parser.add_argument("--out", default="out/batch")
    args = parser.parse_args()

    corpus = json.load(open(args.corpus, encoding="utf-8"))
    items = corpus[args.shard :: args.total_shards]
    if args.limit > 0:
        items = items[: args.limit]

    ref_pool = load_ref_pool(args.pool)
    out_dir = os.path.join(args.out, f"shard_{args.shard}")
    os.makedirs(out_dir, exist_ok=True)
    print(
        f"[shard {args.shard}] {len(items)} items, pool={len(ref_pool)} refs",
        flush=True,
    )

    tts = IndexTTS2(
        cfg_path=os.path.join(args.checkpoints, "config.yaml"),
        model_dir=args.checkpoints,
        use_bf16=True,
        device="xpu",
    )
    print(
        f"[shard {args.shard}] model loaded, VRAM={torch.xpu.memory_allocated() / 1e9:.1f}G",
        flush=True,
    )

    done = fail = 0
    consecutive_fails = 0
    t_start = time.perf_counter()
    last_report_done = 0  # done-only work for a truthful rate (skips inflate it)

    def generate(item: dict) -> bool:
        """One clip; False on failure (with cascade-breaking cleanup)."""
        fn = os.path.join(out_dir, f"{item['id']:06d}.wav")
        try:
            tts.infer(
                spk_audio_prompt=ref_pool[item["id"] % len(ref_pool)],
                text=item["text"],
                lang="JA",
                output_path=fn,
            )
            return True
        except Exception as e:  # noqa: BLE001 — log-and-continue is the contract
            print(f"[shard {args.shard}] FAIL id={item['id']}: {e}", flush=True)
            if os.path.exists(fn):
                os.remove(fn)
            # OOM debris: unwinding does not free failed-inference tensors;
            # without this cleanup one OOM poisons every following item.
            gc.collect()
            torch.xpu.empty_cache()
            torch.xpu.synchronize()
            return False

    # Pass 1: everything missing/invalid; pass 2 (below): retries failures.
    for item in items:
        fn = os.path.join(out_dir, f"{item['id']:06d}.wav")
        if is_valid_clip(fn):
            continue

        if generate(item):
            done += 1
            consecutive_fails = 0
        else:
            fail += 1
            consecutive_fails += 1
            if consecutive_fails >= 5:
                print(
                    f"[shard {args.shard}] 5 consecutive fails — "
                    "restarting worker (systemd Restart=always)",
                    flush=True,
                )
                # Remember the failure count: a restart retries this tail.
                sys.exit(1)

        processed = done + fail
        if processed > 0 and processed % 20 == 0:
            gc.collect()
            torch.xpu.empty_cache()
        if done // 10 > last_report_done // 10:
            last_report_done = done
            elapsed = time.perf_counter() - t_start
            rate = done / elapsed if elapsed > 0 else 0.0
            print(
                f"[shard {args.shard}] done={done} fail={fail} "
                f"rate={rate * 60:.0f}/min elapsed={elapsed / 60:.0f}min",
                flush=True,
            )

    # Pass 2: one retry round for scattered (non-cascade) failures.
    if fail > 0:
        print(f"[shard {args.shard}] retry pass for {fail} failed items", flush=True)
        retried = 0
        for item in items:
            fn = os.path.join(out_dir, f"{item['id']:06d}.wav")
            if not is_valid_clip(fn):
                if generate(item):
                    done += 1
                    fail -= 1
                retried += 1
        print(f"[shard {args.shard}] retry pass done: {retried} retried", flush=True)

    elapsed = time.perf_counter() - t_start
    print(
        f"[shard {args.shard}] FINISHED: done={done} fail={fail} "
        f"in {elapsed / 60:.1f}min",
        flush=True,
    )
    if fail > 0:
        # Incomplete shard is NOT a deployable state: examples/audio/ is
        # immutable, a missing file would 404 forever (the pitch/audio
        # incident class). Exit 1 so the orchestrating layer notices.
        sys.exit(1)


if __name__ == "__main__":
    main()
