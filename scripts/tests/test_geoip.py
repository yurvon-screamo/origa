"""Integration tests for the GeoIP provisioning script (``origa_ui/geoip.sh``).

The script downloads the GeoLite2-Country mmdb from a fixed URL and installs
it into the TrailBase depot; TrailBase picks the database up at startup for
the admin logs geo enrichment (ADR-052). Size-based validation of the
download was replaced by mmdb format validation: the metadata marker
(``\\xAB\\xCD\\xEFMaxMind.com``) must be present in the tail of the file —
the metadata section always sits at the end of an mmdb file, so a truncated
download loses it. The tail window is 128 KiB, the maximum metadata section
size allowed by the MaxMind-DB spec.

The tests run the script as a black box against a local HTTP server serving
synthetic mmdb fixtures:

* a valid fixture — a >= 128 KB file with the marker ~64 KB from EOF; the
  distance regression-pins the 128 KiB window (shrinking the window drops
  the marker out of it and fails these tests);
* a truncated fixture — the same file cut before the marker.

Behavior contract pinned here:

* a valid database far below the legacy 4 MiB size threshold installs
  successfully (the original bug: the size gate rejected the real ~3.2 MB
  GeoLite2-Country on every weekly refresh, so production silently lived
  without geo enrichment);
* a truncated download is discarded and never reaches the depot (no tmp
  orphan left behind);
* a download whose marker sits beyond the 128 KiB tail window is discarded —
  the window stays pinned at the spec's maximum metadata size;
* a fresh valid depot file is left alone (no download happens);
* a fresh depot file without the marker is re-downloaded;
* a valid depot file older than the freshness window is re-downloaded.
"""

from __future__ import annotations

import os
import subprocess
import threading
import time
from functools import partial
from http.server import SimpleHTTPRequestHandler, ThreadingHTTPServer
from pathlib import Path
from typing import NamedTuple

import pytest

REPO_ROOT = Path(__file__).resolve().parents[2]
SCRIPT = REPO_ROOT / "origa_ui" / "geoip.sh"

MARKER = b"\xab\xcd\xefMaxMind.com"
# Max metadata section size per the MaxMind-DB spec (the script's tail window).
WINDOW_BYTES = 128 * 1024
# Marker distance from EOF in the valid fixture. Deliberately far above any
# plausible smaller window: shrinking the window drops the marker out of it,
# so a window regression fails the install/skip tests instead of passing.
MARKER_DISTANCE_FROM_EOF = 64 * 1024

DEPOT_FILENAME = "GeoLite2-Country.mmdb"


def valid_mmdb_bytes() -> bytes:
    """Synthetic mmdb: 192 KB file, marker ~64 KB from EOF."""
    size = WINDOW_BYTES + MARKER_DISTANCE_FROM_EOF
    marker_offset = size - MARKER_DISTANCE_FROM_EOF - len(MARKER)
    body = bytearray(b"\x00" * size)
    body[marker_offset : marker_offset + len(MARKER)] = MARKER
    return bytes(body)


def truncated_mmdb_bytes() -> bytes:
    """Same file cut before the marker — what a truncated download looks like."""
    return valid_mmdb_bytes()[: MARKER_DISTANCE_FROM_EOF - 1024]


def tail_junk_mmdb_bytes() -> bytes:
    """Valid-shaped file with a junk tail longer than the 128 KiB window.

    The marker ends up beyond the spec's maximum metadata distance from EOF:
    a reader must reject it, and so must the script — pinning the window at
    its spec maximum instead of letting it grow silently.
    """
    junk = b"\xff" * (2 * WINDOW_BYTES)
    return valid_mmdb_bytes() + junk


FRESHNESS_DAYS = 6


class _ServerHandle(NamedTuple):
    base_url: str
    directory: Path
    request_log: list[str]

    @property
    def hits(self) -> int:
        return len(self.request_log)


@pytest.fixture
def serve(tmp_path_factory: pytest.TempPathFactory):
    serve_dir = tmp_path_factory.mktemp("serve")
    request_log: list[str] = []

    class Handler(SimpleHTTPRequestHandler):
        def do_GET(self) -> None:
            request_log.append(self.path)
            super().do_GET()

        def log_message(self, format: str, *args: object) -> None:  # noqa: A002
            pass  # silence per-request stderr noise in test output

    handler = partial(Handler, directory=str(serve_dir))
    server = ThreadingHTTPServer(("127.0.0.1", 0), handler)
    thread = threading.Thread(target=server.serve_forever, daemon=True)
    thread.start()
    try:
        yield _ServerHandle(
            base_url=f"http://127.0.0.1:{server.server_address[1]}",
            directory=serve_dir,
            request_log=request_log,
        )
    finally:
        server.shutdown()
        thread.join(timeout=5)
        server.server_close()


def run_geoip(
    depot: Path, url: str, env_overrides: dict[str, str] | None = None
) -> subprocess.CompletedProcess[str]:
    env = {
        "PATH": os.environ["PATH"],
        "TRAILBASE_DATA_DIR": str(depot),
        "GEOIP_URL": url,
        "GEOIP_TIMEOUT": "30",
    }
    env.update(env_overrides or {})
    return subprocess.run(
        ["sh", str(SCRIPT)],
        env=env,
        capture_output=True,
        text=True,
        timeout=120,
        check=False,
    )


class TestDownloadAndInstall:
    def test_valid_database_below_legacy_size_threshold_installs(
        self, tmp_path: Path, serve: _ServerHandle
    ) -> None:
        # Arrange: the real GeoLite2-Country is ~3.2 MB, well below the old
        # 4 MiB gate; this fixture is valid mmdb shape, and far below it too.
        (serve.directory / "country.mmdb").write_bytes(valid_mmdb_bytes())
        depot = tmp_path / "traildepot"

        # Act
        proc = run_geoip(depot, f"{serve.base_url}/country.mmdb")

        # Assert: the file reached the depot byte-for-byte.
        assert proc.returncode == 0, proc.stdout + proc.stderr
        installed = depot / DEPOT_FILENAME
        assert installed.read_bytes() == valid_mmdb_bytes()

    def test_truncated_download_is_discarded_and_never_installed(
        self, tmp_path: Path, serve: _ServerHandle
    ) -> None:
        # Arrange: a download cut short has no metadata marker in the tail.
        (serve.directory / "truncated.mmdb").write_bytes(truncated_mmdb_bytes())
        depot = tmp_path / "traildepot"

        # Act
        proc = run_geoip(depot, f"{serve.base_url}/truncated.mmdb")

        # Assert: rejected, and the depot stays empty — no partial install,
        # no tmp orphan left behind (the EXIT trap must clean it up).
        assert proc.returncode != 0
        assert not (depot / DEPOT_FILENAME).exists()
        assert list(depot.glob(f"{DEPOT_FILENAME}.tmp.*")) == []

    def test_download_with_marker_beyond_tail_window_is_discarded(
        self, tmp_path: Path, serve: _ServerHandle
    ) -> None:
        # Arrange: the marker sits beyond the 128 KiB tail window (junk
        # appended after the metadata section) — the spec caps the metadata
        # section at 128 KiB from EOF, so this file is not a valid mmdb.
        (serve.directory / "tailjunk.mmdb").write_bytes(tail_junk_mmdb_bytes())
        depot = tmp_path / "traildepot"

        # Act
        proc = run_geoip(depot, f"{serve.base_url}/tailjunk.mmdb")

        # Assert: rejected — the window stays pinned at its spec maximum,
        # and the discarded tmp file left no orphan behind.
        assert proc.returncode != 0
        assert not (depot / DEPOT_FILENAME).exists()
        assert list(depot.glob(f"{DEPOT_FILENAME}.tmp.*")) == []


class TestFreshSkip:
    def test_fresh_valid_depot_file_is_not_re_downloaded(
        self, tmp_path: Path, serve: _ServerHandle
    ) -> None:
        # Arrange: depot holds a fresh valid file; the server would answer
        # with garbage, so any download would corrupt the depot and fail
        # the byte-comparison below.
        depot = tmp_path / "traildepot"
        depot.mkdir(parents=True)
        installed = depot / DEPOT_FILENAME
        installed.write_bytes(valid_mmdb_bytes())
        (serve.directory / "garbage.mmdb").write_bytes(b"\xde\xad" * 1000)

        # Act
        proc = run_geoip(depot, f"{serve.base_url}/garbage.mmdb")

        # Assert: skipped — exit 0, file untouched, server never asked.
        assert proc.returncode == 0, proc.stdout + proc.stderr
        assert installed.read_bytes() == valid_mmdb_bytes()
        assert serve.hits == 0

    def test_fresh_depot_file_without_marker_is_re_downloaded(
        self, tmp_path: Path, serve: _ServerHandle
    ) -> None:
        # Arrange: a corrupt leftover that is still "fresh" by mtime —
        # the size gate trusted such files, the marker check must not.
        depot = tmp_path / "traildepot"
        depot.mkdir(parents=True)
        installed = depot / DEPOT_FILENAME
        installed.write_bytes(truncated_mmdb_bytes())
        (serve.directory / "country.mmdb").write_bytes(valid_mmdb_bytes())

        # Act
        proc = run_geoip(depot, f"{serve.base_url}/country.mmdb")

        # Assert: re-downloaded, the depot now holds the valid file.
        assert proc.returncode == 0, proc.stdout + proc.stderr
        assert installed.read_bytes() == valid_mmdb_bytes()

    def test_stale_valid_depot_file_is_re_downloaded(
        self, tmp_path: Path, serve: _ServerHandle
    ) -> None:
        # Arrange: a valid file older than the freshness window — validity
        # does not freeze the database; the weekly cron must still refresh it.
        depot = tmp_path / "traildepot"
        depot.mkdir(parents=True)
        installed = depot / DEPOT_FILENAME
        installed.write_bytes(valid_mmdb_bytes())
        stale = (FRESHNESS_DAYS + 1) * 86400
        os.utime(installed, (time.time() - stale, time.time() - stale))
        (serve.directory / "country.mmdb").write_bytes(valid_mmdb_bytes())

        # Act
        proc = run_geoip(depot, f"{serve.base_url}/country.mmdb")

        # Assert: freshness check fired — re-downloaded exactly once.
        assert proc.returncode == 0, proc.stdout + proc.stderr
        assert installed.read_bytes() == valid_mmdb_bytes()
        assert serve.hits == 1
