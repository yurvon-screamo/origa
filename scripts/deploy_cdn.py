"""Generate CDN manifest and deploy incrementally to S3."""

from __future__ import annotations

import argparse
import hashlib
import json
import subprocess
import sys
import tempfile
from datetime import datetime, timezone
from pathlib import Path

S3_BUCKET = "adaptable-foodbox-ucep7wx"
S3_PROFILE = "origa"
S3_ENDPOINT = "https://t3.storageapi.dev"

VERSIONED_FILES: list[str] = [
    "dictionary/chunk_01.json",
    "dictionary/chunk_02.json",
    "dictionary/chunk_03.json",
    "dictionary/chunk_04.json",
    "dictionary/chunk_05.json",
    "dictionary/chunk_06.json",
    "dictionary/chunk_07.json",
    "dictionary/chunk_08.json",
    "dictionary/chunk_09.json",
    "dictionary/chunk_10.json",
    "dictionary/chunk_11.json",
    "dictionary/kanji.json",
    "dictionary/radicals.json",
    "grammar/grammar.json",
    "dictionaries/char_def.bin",
    "dictionaries/matrix.mtx",
    "dictionaries/dict.da",
    "dictionaries/dict.vals",
    "dictionaries/unk.bin",
    "dictionaries/dict.wordsidx",
    "dictionaries/dict.words",
    "dictionaries/metadata.json",
    "dictionaries/JmdictFurigana.txt",
    "phrases/phrase_index.json",
    "pitch/index.json",
    "well_known_set/jlpt_n5.json",
    "well_known_set/jlpt_n4.json",
    "well_known_set/jlpt_n3.json",
    "well_known_set/jlpt_n2.json",
    "well_known_set/jlpt_n1.json",
    "well_known_set/well_known_types_meta.json",
    "well_known_set/well_known_sets_meta.json",
]

SYNC_DIRS = [
    "kanji_animations",
    "kanji_frames",
    "ndlocr",
    "phrases/audio",
    "phrases/data",
]

MANIFEST_VERSION = 1
CHUNK_SIZE = 8192


def sha256_hex(path: Path) -> str:
    hasher = hashlib.sha256()
    with path.open("rb") as f:
        while chunk := f.read(CHUNK_SIZE):
            hasher.update(chunk)
    return hasher.hexdigest()


def generate_manifest(cdn_dir: Path) -> dict[str, object]:
    files: dict[str, str] = {}

    for relative_path in VERSIONED_FILES:
        full_path = cdn_dir / relative_path
        if not full_path.is_file():
            print(f"WARNING: {relative_path} not found, skipping", file=sys.stderr)
            continue
        files[relative_path] = sha256_hex(full_path)

    if not files:
        print("ERROR: no versioned files found in cdn/", file=sys.stderr)
        sys.exit(1)

    return {
        "version": MANIFEST_VERSION,
        "generated_at": datetime.now(timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ"),
        "files": dict(sorted(files.items())),
    }


def s3_uri(key: str) -> str:
    return f"s3://{S3_BUCKET}/{key}"


def run_aws_raw(args: list[str]) -> subprocess.CompletedProcess[str]:
    cmd = ["aws", *args]
    try:
        return subprocess.run(
            cmd,
            capture_output=True,
            text=True,
            shell=True,
        )
    except FileNotFoundError:
        print(
            "ERROR: 'aws' CLI not found. Install:"
            " https://docs.aws.amazon.com/cli/latest/userguide/getting-started-install.html",
            file=sys.stderr,
        )
        sys.exit(1)


def run_aws(args: list[str], dry_run: bool) -> subprocess.CompletedProcess[str]:
    if dry_run:
        print(f"  [DRY-RUN] aws {' '.join(args)}")
        return subprocess.CompletedProcess(args, 0, "", "")

    result = run_aws_raw(args)
    if result.returncode != 0:
        print(f"ERROR: aws {' '.join(args)}", file=sys.stderr)
        print(result.stderr, file=sys.stderr)
        sys.exit(1)
    return result


def download_remote_manifest(dry_run: bool) -> dict[str, object] | None:
    with tempfile.NamedTemporaryFile(suffix=".json", delete=False) as tmp:
        tmp_path = Path(tmp.name)

    if dry_run:
        print("  [DRY-RUN] would download remote manifest")
        return None

    result = run_aws_raw([
        "s3", "cp", s3_uri("manifest.json"), str(tmp_path),
        "--profile", S3_PROFILE, "--endpoint-url", S3_ENDPOINT,
    ])

    if result.returncode != 0:
        if "404" in result.stderr or "NoSuchKey" in result.stderr:
            print("  Remote manifest not found (first deployment)")
            return None
        print("ERROR: failed to download remote manifest", file=sys.stderr)
        print(result.stderr, file=sys.stderr)
        sys.exit(1)

    content = tmp_path.read_text(encoding="utf-8")
    return json.loads(content)


def compare_manifests(
    local: dict[str, object],
    remote: dict[str, object] | None,
) -> tuple[list[str], list[str]]:
    if remote is None:
        return list(local["files"].keys()), []

    remote_files = remote.get("files", {})
    local_files = local["files"]

    changed: list[str] = []
    unchanged: list[str] = []

    for path, hash_val in local_files.items():
        if remote_files.get(path) != hash_val:
            changed.append(path)
        else:
            unchanged.append(path)

    return changed, unchanged


def upload_versioned_files(
    cdn_dir: Path,
    files: list[str],
    dry_run: bool,
) -> None:
    if not files:
        print("No changed versioned files to upload.")
        return

    print(f"\nUploading {len(files)} changed versioned file(s):")
    for relative_path in sorted(files):
        local_path = cdn_dir / relative_path
        print(f"  {relative_path}")
        run_aws(
            [
                "s3",
                "cp",
                str(local_path),
                s3_uri(relative_path),
                "--profile",
                S3_PROFILE,
                "--endpoint-url",
                S3_ENDPOINT,
                "--cache-control",
                "public, max-age=31536000, immutable",
            ],
            dry_run,
        )


def sync_immutable_dirs(cdn_dir: Path, dry_run: bool) -> None:
    print("\nSyncing immutable directories:")
    for dir_name in SYNC_DIRS:
        local_dir = cdn_dir / dir_name
        if not local_dir.is_dir():
            print(f"  {dir_name}/ — not found locally, skipping")
            continue

        print(f"  {dir_name}/")
        run_aws(
            [
                "s3",
                "sync",
                str(local_dir),
                s3_uri(dir_name),
                "--profile",
                S3_PROFILE,
                "--endpoint-url",
                S3_ENDPOINT,
                "--exclude",
                "README.md",
                "--cache-control",
                "public, max-age=31536000, immutable",
            ],
            dry_run,
        )


def upload_manifest(cdn_dir: Path, dry_run: bool) -> None:
    manifest_path = cdn_dir / "manifest.json"
    print("\nUploading manifest.json (Cache-Control: no-cache)")
    run_aws(
        [
            "s3",
            "cp",
            str(manifest_path),
            s3_uri("manifest.json"),
            "--profile",
            S3_PROFILE,
            "--endpoint-url",
            S3_ENDPOINT,
            "--cache-control",
            "no-cache",
            "--metadata-directive",
            "REPLACE",
        ],
        dry_run,
    )


def main() -> None:
    parser = argparse.ArgumentParser(description="Deploy CDN to S3")
    parser.add_argument(
        "--dry-run",
        action="store_true",
        help="Show what would be deployed without actually uploading",
    )
    args = parser.parse_args()
    dry_run = args.dry_run

    if dry_run:
        print("=== DRY RUN ===\n")

    project_root = Path(__file__).resolve().parent.parent
    cdn_dir = project_root / "cdn"

    if not cdn_dir.is_dir():
        print(f"ERROR: {cdn_dir} not found", file=sys.stderr)
        sys.exit(1)

    # Step 1: Generate local manifest
    print("Step 1: Generating local manifest...")
    manifest = generate_manifest(cdn_dir)
    manifest_path = cdn_dir / "manifest.json"
    manifest_path.write_text(
        json.dumps(manifest, indent=2, ensure_ascii=False) + "\n",
        encoding="utf-8",
    )
    file_count = len(manifest["files"])
    print(f"  {file_count} files hashed -> cdn/manifest.json")

    # Step 2: Download remote manifest
    print("\nStep 2: Fetching remote manifest...")
    remote_manifest = download_remote_manifest(dry_run)

    # Step 3: Compare
    print("\nStep 3: Comparing manifests...")
    changed, unchanged = compare_manifests(manifest, remote_manifest)
    print(f"  {len(changed)} changed, {len(unchanged)} unchanged")
    if changed:
        for path in sorted(changed):
            print(f"    + {path}")

    # Step 4: Upload changed versioned files
    print(f"\nStep 4: Uploading changed versioned files...")
    upload_versioned_files(cdn_dir, changed, dry_run)

    # Step 5: Sync immutable directories
    print(f"\nStep 5: Syncing immutable directories...")
    sync_immutable_dirs(cdn_dir, dry_run)

    # Step 6: Upload manifest
    print(f"\nStep 6: Uploading manifest...")
    upload_manifest(cdn_dir, dry_run)

    print("\nDone!")


if __name__ == "__main__":
    main()
