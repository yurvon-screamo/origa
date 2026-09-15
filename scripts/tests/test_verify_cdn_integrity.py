"""Contract tests for the download-cdn integrity verifier.

The script is a choke-point for CI data integrity (issue #544): four jobs
consume the ``cdn-data`` artifact it protects. These tests pin its CLI
contract — exit codes, report lines (MISSING/STALE/SKIP/ERROR), and the
``presence(file-list) ∧ hash(file-list ∩ hash-manifest)`` invariant — so a
silent semantic change (e.g. in SKIP handling or whitespace parsing) breaks
a test instead of a CI run.

The script is executed as a subprocess on purpose: the CLI boundary
(argv parsing, exit code, stdout reporting) is the contract consumed by
``.github/actions/download-cdn/action.yml``.
"""

from __future__ import annotations

import hashlib
import json
import subprocess
import sys
from pathlib import Path

import pytest

REPO_ROOT = Path(__file__).resolve().parents[2]
SCRIPT = REPO_ROOT / ".github" / "actions" / "download-cdn" / "scripts" / "verify_cdn_integrity.py"

ALPHA = b"alpha content"
BETA = b"beta content"


def sha256(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def write_cdn(cdn_dir: Path, files: dict[str, bytes]) -> None:
    for rel, content in files.items():
        target = cdn_dir / rel
        target.parent.mkdir(parents=True, exist_ok=True)
        target.write_bytes(content)


def write_file_list(path: Path, lines: list[str]) -> Path:
    path.write_text("\n".join(lines) + "\n", encoding="utf-8")
    return path


def write_hash_manifest(path: Path, files: dict[str, str]) -> Path:
    path.write_text(json.dumps({"files": files}), encoding="utf-8")
    return path


def run_script(
    cdn_dir: Path,
    file_list: Path,
    hash_manifest: Path | None = None,
) -> tuple[int, str]:
    command = [
        sys.executable,
        str(SCRIPT),
        "--cdn-dir",
        str(cdn_dir),
        "--file-list",
        str(file_list),
    ]
    if hash_manifest is not None:
        command += ["--hash-manifest", str(hash_manifest)]
    completed = subprocess.run(command, capture_output=True, text=True)
    return completed.returncode, completed.stdout + completed.stderr


def test_happy_path_exit_zero(tmp_path: Path) -> None:
    cdn_dir = tmp_path / "cdn"
    write_cdn(cdn_dir, {"a/one.txt": ALPHA, "two.txt": BETA})
    file_list = write_file_list(
        tmp_path / "list.txt",
        ["a/one.txt", "two.txt", "# a comment", ""],
    )
    hash_manifest = write_hash_manifest(
        tmp_path / "manifest.json",
        {"a/one.txt": sha256(ALPHA), "two.txt": sha256(BETA)},
    )

    code, output = run_script(cdn_dir, file_list, hash_manifest)

    assert code == 0, output
    assert "MISSING" not in output, output
    assert "STALE" not in output, output
    assert "OK:" in output, output


def test_missing_file_reports_missing(tmp_path: Path) -> None:
    cdn_dir = tmp_path / "cdn"
    write_cdn(cdn_dir, {"two.txt": BETA})
    file_list = write_file_list(tmp_path / "list.txt", ["a/one.txt", "two.txt"])

    code, output = run_script(cdn_dir, file_list)

    assert code == 1, output
    assert "MISSING:" in output, output
    assert "a/one.txt" in output, output


def test_empty_file_reports_missing(tmp_path: Path) -> None:
    cdn_dir = tmp_path / "cdn"
    write_cdn(cdn_dir, {"two.txt": b""})
    file_list = write_file_list(tmp_path / "list.txt", ["two.txt"])

    code, output = run_script(cdn_dir, file_list)

    assert code == 1, output
    assert "MISSING:" in output, output


def test_whitespace_only_lines_are_skipped(tmp_path: Path) -> None:
    cdn_dir = tmp_path / "cdn"
    write_cdn(cdn_dir, {"two.txt": BETA})
    file_list = write_file_list(tmp_path / "list.txt", ["two.txt", "   ", "\t"])

    code, output = run_script(cdn_dir, file_list)

    assert code == 0, output
    assert "MISSING" not in output, output


def test_corrupt_file_reports_stale(tmp_path: Path) -> None:
    cdn_dir = tmp_path / "cdn"
    write_cdn(cdn_dir, {"a/one.txt": b"corrupted payload"})
    file_list = write_file_list(tmp_path / "list.txt", ["a/one.txt"])
    hash_manifest = write_hash_manifest(
        tmp_path / "manifest.json", {"a/one.txt": sha256(ALPHA)}
    )

    code, output = run_script(cdn_dir, file_list, hash_manifest)

    assert code == 1, output
    assert "STALE:" in output, output
    assert "expected=" in output, output


def test_hash_entry_outside_file_list_is_skipped(tmp_path: Path) -> None:
    cdn_dir = tmp_path / "cdn"
    write_cdn(cdn_dir, {"a/one.txt": ALPHA})
    file_list = write_file_list(tmp_path / "list.txt", ["a/one.txt"])
    hash_manifest = write_hash_manifest(
        tmp_path / "manifest.json",
        {
            "a/one.txt": sha256(ALPHA),
            "phrases/data_bundle_0.json": sha256(BETA),
        },
    )

    code, output = run_script(cdn_dir, file_list, hash_manifest)

    assert code == 0, output
    assert "SKIP (not in file list): phrases/data_bundle_0.json" in output, output
    assert "MISSING" not in output, output


def test_missing_file_reported_once_with_hash_manifest(tmp_path: Path) -> None:
    cdn_dir = tmp_path / "cdn"
    write_cdn(cdn_dir, {"two.txt": BETA})
    file_list = write_file_list(tmp_path / "list.txt", ["a/one.txt", "two.txt"])
    hash_manifest = write_hash_manifest(
        tmp_path / "manifest.json",
        {"a/one.txt": sha256(ALPHA), "two.txt": sha256(BETA)},
    )

    code, output = run_script(cdn_dir, file_list, hash_manifest)

    assert code == 1, output
    assert output.count("MISSING:") == 1, output


def test_absent_hash_manifest_file(tmp_path: Path) -> None:
    cdn_dir = tmp_path / "cdn"
    write_cdn(cdn_dir, {"two.txt": BETA})
    file_list = write_file_list(tmp_path / "list.txt", ["two.txt"])
    hash_manifest = tmp_path / "manifest.json"

    code, output = run_script(cdn_dir, file_list, hash_manifest)

    assert code == 1, output
    assert output.count("ERROR:") == 1, output
    assert "not found" in output, output
    assert "Traceback" not in output, output


@pytest.mark.parametrize(
    "payload",
    [
        json.dumps({"version": 1}),
        json.dumps({"files": ["a/one.txt"]}),
    ],
    ids=["missing-files-key", "files-not-an-object"],
)
def test_structurally_broken_hash_manifest(tmp_path: Path, payload: str) -> None:
    cdn_dir = tmp_path / "cdn"
    write_cdn(cdn_dir, {"two.txt": BETA})
    file_list = write_file_list(tmp_path / "list.txt", ["two.txt"])
    hash_manifest = tmp_path / "manifest.json"
    hash_manifest.write_text(payload, encoding="utf-8")

    code, output = run_script(cdn_dir, file_list, hash_manifest)

    assert code == 1, output
    assert output.count("ERROR:") == 1, output
    assert "'files'" in output, output
    assert "Traceback" not in output, output


def test_invalid_json_hash_manifest(tmp_path: Path) -> None:
    cdn_dir = tmp_path / "cdn"
    write_cdn(cdn_dir, {"two.txt": BETA})
    file_list = write_file_list(tmp_path / "list.txt", ["two.txt"])
    hash_manifest = tmp_path / "manifest.json"
    hash_manifest.write_text("{not json", encoding="utf-8")

    code, output = run_script(cdn_dir, file_list, hash_manifest)

    assert code == 1, output
    assert output.count("ERROR:") == 1, output
    assert "not valid JSON" in output, output
    assert "Traceback" not in output, output


def test_empty_file_list_fails(tmp_path: Path) -> None:
    file_list = write_file_list(tmp_path / "list.txt", ["# only a comment", ""])

    code, output = run_script(tmp_path / "cdn", file_list)

    assert code == 1, output
    assert "ERROR: file list" in output, output


def test_default_file_list_resolution(tmp_path: Path) -> None:
    # Production invocations in action.yml pass --file-list explicitly, but
    # the argparse default (end2end/cdn-manifest.txt, cwd-relative) is part
    # of the CLI surface: pin it too.
    cdn_dir = tmp_path / "cdn"
    write_cdn(cdn_dir, {"two.txt": BETA})
    (tmp_path / "end2end").mkdir()
    write_file_list(tmp_path / "end2end" / "cdn-manifest.txt", ["two.txt"])

    command = [sys.executable, str(SCRIPT), "--cdn-dir", str(cdn_dir)]
    completed = subprocess.run(command, capture_output=True, text=True, cwd=tmp_path)

    assert completed.returncode == 0, completed.stdout + completed.stderr
    assert "OK:" in completed.stdout, completed.stdout


def test_presence_only_mode(tmp_path: Path) -> None:
    cdn_dir = tmp_path / "cdn"
    write_cdn(cdn_dir, {"two.txt": BETA})
    file_list = write_file_list(tmp_path / "list.txt", ["two.txt"])

    code, output = run_script(cdn_dir, file_list)

    assert code == 0, output
    assert "OK:" in output, output
