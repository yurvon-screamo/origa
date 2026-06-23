"""S3 CLI wrappers shared by ``deploy_cdn.py``.

Centralises ``aws`` CLI invocation (shelled out via ``pwsh`` on Windows because
that is how the operator's PowerShell environment resolves the AWS wrapper on
the deployment host) so that ``deploy_cdn.py`` stays a deployment orchestrator
rather than a transport layer. All S3 paths route through here: upload, sync,
manifest download.
"""

from __future__ import annotations

import json
import subprocess
import sys
import tempfile
from pathlib import Path

S3_BUCKET = "adaptable-foodbox-ucep7wx"
S3_PROFILE = "origa"
S3_ENDPOINT = "https://t3.storageapi.dev"


def s3_uri(key: str) -> str:
    return f"s3://{S3_BUCKET}/{key}"


def run_aws_raw(args: list[str]) -> subprocess.CompletedProcess[str]:
    cmd = ["pwsh", "-Command", "aws", *args]
    try:
        return subprocess.run(
            cmd,
            capture_output=True,
            text=True,
        )
    except FileNotFoundError:
        print("ERROR: 'aws' CLI not found.", file=sys.stderr)
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
    # delete=False because the aws CLI runs as a separate process and re-opens
    # the path by name on Windows; cleanup happens in the ``finally`` below so
    # the temp file cannot leak even on dry-run / error paths.
    with tempfile.NamedTemporaryFile(suffix=".json", delete=False) as tmp:
        tmp_path = Path(tmp.name)

    try:
        if dry_run:
            print("  [DRY-RUN] would download remote manifest")
            return None

        result = run_aws_raw(
            [
                "s3",
                "cp",
                s3_uri("manifest.json"),
                str(tmp_path),
                "--profile",
                S3_PROFILE,
                "--endpoint-url",
                S3_ENDPOINT,
            ]
        )

        if result.returncode != 0:
            if "404" in result.stderr or "NoSuchKey" in result.stderr:
                print("  Remote manifest not found (first deployment)")
                return None
            print("ERROR: failed to download remote manifest", file=sys.stderr)
            print(result.stderr, file=sys.stderr)
            sys.exit(1)

        content = tmp_path.read_text(encoding="utf-8")
        return json.loads(content)
    finally:
        tmp_path.unlink(missing_ok=True)
