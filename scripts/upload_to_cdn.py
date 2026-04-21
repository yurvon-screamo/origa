#! scripts/.venv/Scripts/python.exe
"""Upload phrase dataset and audio files to Railway Storage (S3-compatible CDN).

Usage:
    python upload_to_cdn.py setup                     — configure bucket (CORS, policy)
    python upload_to_cdn.py data [--workers N]        — upload phrase_index.json + chunks
    python upload_to_cdn.py audio --dir <path> [-w N] — upload .opus files
"""

from __future__ import annotations

import argparse
import gzip
import sys
import time
from concurrent.futures import ThreadPoolExecutor, as_completed
from pathlib import Path

import boto3
from botocore.config import Config as BotoConfig

SCRIPT_DIR = Path(__file__).resolve().parent
PROJECT_ROOT = SCRIPT_DIR.parent

DIST_DIR = PROJECT_ROOT / "phrase_dataset" / "dist"

ENDPOINT_URL = "https://t3.storageapi.dev"
BUCKET = "stored-breadbox-fk2diaux"
REGION = "auto"

CORS_POLICY = {
    "CORSRules": [
        {
            "AllowedOrigins": ["*"],
            "AllowedMethods": ["GET", "HEAD"],
            "AllowedHeaders": ["*"],
            "MaxAgeSeconds": 86400,
        }
    ]
}

PUBLIC_READ_POLICY = {
    "Version": "2012-10-17",
    "Statement": [
        {
            "Sid": "PublicReadGetObject",
            "Effect": "Allow",
            "Principal": "*",
            "Action": "s3:GetObject",
            "Resource": f"arn:aws:s3:::{BUCKET}/*",
        }
    ],
}

PROGRESS_INTERVAL = 10

CACHE_INDEX = "public, max-age=86400"
CACHE_IMMUTABLE = "public, max-age=31536000, immutable"


def create_s3_client() -> boto3.client:
    return boto3.client(
        "s3",
        endpoint_url=ENDPOINT_URL,
        region_name=REGION,
        config=BotoConfig(signature_version="s3v4"),
    )


def cmd_setup(_args: argparse.Namespace) -> None:
    client = create_s3_client()
    client.put_bucket_cors(Bucket=BUCKET, CORSConfiguration=CORS_POLICY)
    print("CORS policy applied")
    client.put_bucket_policy(Bucket=BUCKET, Policy=public_read_policy_json())
    print("Public read policy applied")


def public_read_policy_json() -> str:
    import json

    return json.dumps(PUBLIC_READ_POLICY)


def cmd_data(args: argparse.Namespace) -> None:
    index_path = DIST_DIR / "phrase_index.json"
    chunk_dir = DIST_DIR / "data"
    files: list[tuple[Path, str, str]] = []

    if not index_path.exists():
        print(f"Error: {index_path} not found", file=sys.stderr)
        sys.exit(1)

    files.append((index_path, "phrases/phrase_index.json", CACHE_INDEX))

    if not chunk_dir.exists():
        print(f"Error: {chunk_dir} not found", file=sys.stderr)
        sys.exit(1)

    for path in sorted(chunk_dir.glob("p????.json")):
        key = f"phrases/data/{path.name}"
        files.append((path, key, CACHE_IMMUTABLE))

    print(f"Data files to upload: {len(files)}")
    client = create_s3_client()
    stats = upload_parallel(client, files, gzip_json=True, workers=args.workers)
    print_summary(stats)


def cmd_audio(args: argparse.Namespace) -> None:
    audio_dir = Path(args.dir)
    if not audio_dir.is_dir():
        print(f"Error: {audio_dir} is not a directory", file=sys.stderr)
        sys.exit(1)

    opus_files = sorted(audio_dir.glob("*.opus"))
    if not opus_files:
        print("No .opus files found", file=sys.stderr)
        sys.exit(1)

    files = [(p, f"phrases/audio/{p.name}", CACHE_IMMUTABLE) for p in opus_files]
    print(f"Audio files to upload: {len(files)}")
    client = create_s3_client()
    stats = upload_parallel(client, files, gzip_json=False, workers=args.workers)
    print_summary(stats)


def upload_parallel(
    client,
    files: list[tuple[Path, str, str]],
    *,
    gzip_json: bool,
    workers: int,
) -> dict:
    start = time.monotonic()
    stats = {"total": len(files), "uploaded": 0, "skipped": 0, "failed": 0}
    errors: list[str] = []

    with ThreadPoolExecutor(max_workers=workers) as pool:
        futures = {
            pool.submit(
                upload_single, client, local, key, cache, gzip_json
            ): (local, key)
            for local, key, cache in files
        }

        done_count = 0
        for future in as_completed(futures):
            local, key = futures[future]
            done_count += 1
            result = future.result()

            if result == "uploaded":
                stats["uploaded"] += 1
            elif result == "skipped":
                stats["skipped"] += 1
            else:
                stats["failed"] += 1
                errors.append(f"{key}: {result}")

            if done_count % PROGRESS_INTERVAL == 0:
                print(f"  {done_count}/{stats['total']} processed...")

    elapsed = time.monotonic() - start
    stats["elapsed"] = elapsed
    stats["errors"] = errors
    return stats


def upload_single(
    client,
    local: Path,
    key: str,
    cache_control: str,
    gzip_json: bool,
) -> str:
    if file_exists_on_cdn(client, key, local):
        return "skipped"

    try:
        extra = {"CacheControl": cache_control, "ACL": "public-read"}

        if gzip_json:
            body = gzip.compress(local.read_bytes())
            extra["ContentEncoding"] = "gzip"
            extra["ContentType"] = "application/json"
        else:
            body = local.read_bytes()
            extra["ContentType"] = "audio/opus"

        client.put_object(Bucket=BUCKET, Key=key, Body=body, **extra)
        return "uploaded"
    except Exception as exc:
        return str(exc)


def file_exists_on_cdn(client, key: str, local: Path) -> bool:
    try:
        client.head_object(Bucket=BUCKET, Key=key)
        return True
    except client.exceptions.ClientError:
        return False
    except Exception:
        return False


def print_summary(stats: dict) -> None:
    elapsed = stats.get("elapsed", 0)
    print(f"\nDone in {elapsed:.1f}s")
    print(f"  Total:    {stats['total']}")
    print(f"  Uploaded: {stats['uploaded']}")
    print(f"  Skipped:  {stats['skipped']}")
    print(f"  Failed:   {stats['failed']}")

    for err in stats.get("errors", []):
        print(f"  ERROR: {err}")


def build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(
        description="Upload phrase dataset files to Railway Storage CDN"
    )
    sub = parser.add_subparsers(dest="command", required=True)

    sub.add_parser("setup", help="Configure bucket CORS and public read policy")

    data_p = sub.add_parser("data", help="Upload phrase_index.json and chunk files")
    data_p.add_argument(
        "--workers", type=int, default=10, help="Parallel upload threads"
    )

    audio_p = sub.add_parser("audio", help="Upload .opus audio files")
    audio_p.add_argument("--dir", required=True, help="Directory with .opus files")
    audio_p.add_argument(
        "--workers", type=int, default=50, help="Parallel upload threads"
    )

    return parser


def main() -> None:
    parser = build_parser()
    args = parser.parse_args()

    commands = {
        "setup": cmd_setup,
        "data": cmd_data,
        "audio": cmd_audio,
    }
    commands[args.command](args)


if __name__ == "__main__":
    main()
