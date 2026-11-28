#!/usr/bin/env python3
"""IndexTTS-2.5 batch worker for counter-suffix binding readings.

Mirrors ``gen_batch.py`` (ref-pool rotation, XPU inference, checkpoint/resume,
OOM-hardened failure model) but emits immutable, TTS-marked ULID keys under
``counters/audio/``. Every reading of every suffix in ``counters.json`` gets
one clip; the key is derived from suffix + number + reading, so a changed
reading mints a new key instead of overwriting an immutable CDN object.

Usage (from the tts work dir described in tts/README.md):

    PYTHONPATH=<pylib> python3 tts/gen_counter_audio.py \
        --cdn-dir ~/origa-cdn --pool ref_pool --checkpoints checkpoints \
        --out out/counters

The manifest written beside ``--out`` is the deploy audit trail: it maps each
key to its suffix, number and reading so the CDN upload and the app contract
can be checked independently.
"""

from __future__ import annotations

import argparse
import gc
import hashlib
import json
import os
import sys
import time
from pathlib import Path

os.environ.setdefault("HF_HOME", os.path.expanduser("~/.cache/huggingface"))

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


# torchcodec wheels target FFmpeg 4-6; this workstation runs FFmpeg 9.
torchaudio.load = _sf_load  # type: ignore[assignment]
torchaudio.save = _sf_save  # type: ignore[assignment]

from indextts.infer_v2_5 import IndexTTS2  # noqa: E402

CROCKFORD = "0123456789ABCDEFGHJKMNPQRSTVWXYZ"
MIN_VALID_SECONDS = 0.5


def audio_id(suffix: str, number: int, reading: str) -> str:
    """26-char Crockford ULID with a fixed ``TTS`` marker and zero timestamp.

    The randomness is the low 80 bits of SHA-256 over the three content
    fields, encoded exactly like ``ulid::Ulid::from_parts(0, u128)`` does in
    Rust; the last three chars are replaced by the marker. ``origa`` derives
    the same key at render time (``dictionary::counters::counter_audio_path``).
    """
    payload = f"{suffix}|{number}|{reading}".encode("utf-8")
    random_80 = int.from_bytes(hashlib.sha256(payload).digest()[:10], "big")
    encoded = "".join(CROCKFORD[(random_80 >> (5 * i)) & 31] for i in range(15, -1, -1))
    return f"0000000000{encoded[:13]}TTS"


def load_readings(cdn_dir: Path) -> list[dict]:
    """Flatten ``counters/counters.json`` into one row per binding reading."""
    data = json.loads(
        (cdn_dir / "counters" / "counters.json").read_text(encoding="utf-8")
    )
    rows = []
    for entry in data["counters"]:
        for item in entry["readings"]:
            reading = item["reading"]
            rows.append(
                {
                    "id": audio_id(entry["suffix"], item["number"], reading),
                    "suffix": entry["suffix"],
                    "number": item["number"],
                    "reading": reading,
                }
            )
    return rows


def load_ref_pool(pool_dir: Path) -> list[str]:
    """Pinned 80-voice pool in manifest order; a missing pool is a hard abort."""
    manifest_path = pool_dir / "manifest.json"
    if not manifest_path.is_file():
        sys.exit(f"ref pool manifest not found: {manifest_path}")
    manifest = json.loads(manifest_path.read_text(encoding="utf-8"))
    wavs = [str(pool_dir / f"{entry['idx']:03d}.wav") for entry in manifest]
    missing = [w for w in wavs if not Path(w).is_file()]
    if missing:
        sys.exit(f"ref pool incomplete: {len(missing)} wavs missing, e.g. {missing[0]}")
    return wavs


def is_valid_clip(path: Path) -> bool:
    """A truncated wav (plausible size, broken tail) is retried, not shipped."""
    try:
        return (
            path.stat().st_size > 1000
            and sf_lib.info(path).duration >= MIN_VALID_SECONDS
        )
    except (OSError, RuntimeError):
        return False


def voice_index(row: dict, pool_size: int) -> int:
    """Stable voice per reading from the id, independent of corpus ordering."""
    digest = hashlib.sha256(row["id"].encode("utf-8")).digest()
    return int.from_bytes(digest[:2], "big") % pool_size


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    parser.add_argument("--cdn-dir", default=os.path.expanduser("~/origa-cdn"))
    parser.add_argument("--pool", default="ref_pool")
    parser.add_argument("--checkpoints", default="checkpoints")
    parser.add_argument("--out", default="out/counters")
    parser.add_argument("--shard", type=int, default=0)
    parser.add_argument("--total-shards", type=int, default=1)
    parser.add_argument("--gc-every", type=int, default=5)
    parser.add_argument("--limit", type=int, default=0, help="0 = full corpus")
    return parser.parse_args()


def build_model(checkpoints: str) -> IndexTTS2:
    model = IndexTTS2(
        cfg_path=str(Path(checkpoints) / "config.yaml"),
        model_dir=checkpoints,
        use_bf16=True,
        device="xpu",
    )
    print(f"model loaded, VRAM={torch.xpu.memory_allocated() / 1e9:.1f}G", flush=True)
    return model


def release_xpu() -> None:
    """Break the OOM cascade; a lost device must not mask the original error."""
    gc.collect()
    for cleanup in (torch.xpu.empty_cache, torch.xpu.synchronize):
        try:
            cleanup()
        except RuntimeError:
            pass


def generate(model: IndexTTS2, ref_pool: list[str], out_dir: Path, row: dict) -> bool:
    path = out_dir / f"{row['id']}.wav"
    try:
        model.infer(
            spk_audio_prompt=ref_pool[voice_index(row, len(ref_pool))],
            text=row["reading"],
            lang="JA",
            output_path=str(path),
        )
        return True
    except Exception as error:  # noqa: BLE001 — log-and-continue is the contract
        print(f"FAIL {row['id']} ({row['reading']}): {error}", flush=True)
        path.unlink(missing_ok=True)
        release_xpu()
        return False


def first_pass(
    model: IndexTTS2,
    ref_pool: list[str],
    out_dir: Path,
    rows: list[dict],
    gc_every: int,
) -> tuple[int, int]:
    done = fail = consecutive = 0
    started = time.perf_counter()
    for row in rows:
        if is_valid_clip(out_dir / f"{row['id']}.wav"):
            continue
        if generate(model, ref_pool, out_dir, row):
            done += 1
            consecutive = 0
        else:
            fail += 1
            consecutive += 1
            if consecutive >= 5:
                sys.exit(1)
        if (done + fail) % gc_every == 0:
            release_xpu()
        if done % 10 == 0 and done:
            elapsed = time.perf_counter() - started
            print(
                f"done={done} fail={fail} rate={done / elapsed * 60:.0f}/min",
                flush=True,
            )
    return done, fail


def retry_pass(
    model: IndexTTS2, ref_pool: list[str], out_dir: Path, rows: list[dict]
) -> int:
    fail = consecutive = 0
    for row in rows:
        path = out_dir / f"{row['id']}.wav"
        if is_valid_clip(path):
            continue
        if generate(model, ref_pool, out_dir, row):
            consecutive = 0
        else:
            fail += 1
            consecutive += 1
            if consecutive >= 5:
                sys.exit(1)
    return fail


def main() -> None:
    args = parse_args()
    full_rows = load_readings(Path(args.cdn_dir).expanduser())
    rows = full_rows[args.shard :: args.total_shards]
    if args.limit > 0:
        rows = rows[: args.limit]
    ref_pool = load_ref_pool(Path(args.pool))
    out_dir = Path(args.out)
    out_dir.mkdir(parents=True, exist_ok=True)
    # The converter's completeness gate needs the full corpus. Write it before
    # generation so a supervisor restart on OOM still leaves a valid audit
    # trail (the rows are deterministic, independent of what was generated).
    if args.shard == 0:
        (out_dir / "manifest.json").write_text(
            json.dumps(full_rows, ensure_ascii=False, indent=2), encoding="utf-8"
        )
    print(f"{len(rows)} readings, pool={len(ref_pool)} refs", flush=True)

    model = build_model(args.checkpoints)
    _done, fail = first_pass(model, ref_pool, out_dir, rows, args.gc_every)
    if fail:
        fail = retry_pass(model, ref_pool, out_dir, rows)
    if fail:
        sys.exit(1)

    (out_dir / f"manifest-{args.shard}.json").write_text(
        json.dumps(rows, ensure_ascii=False, indent=2), encoding="utf-8"
    )
    print(f"[shard {args.shard}] FINISHED: {len(rows)} clips", flush=True)


if __name__ == "__main__":
    main()
