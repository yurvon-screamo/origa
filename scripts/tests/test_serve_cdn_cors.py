"""Integration tests for the CI CDN mirror server (``serve-cdn-cors.py``).

The ``wasm-test`` job serves the ``cdn-data`` artifact with this stdlib server
and bakes ``http://localhost:8080`` into the wasm test binary at compile time
(hermetic wasm suite, PR #886). These tests pin the observable server contract
the suite relies on:

- a 200 carries ``Access-Control-Allow-Origin: *`` (the wasm-bindgen harness
  page is a different origin, so every fetch is cross-origin) and byte-exact
  file content;
- no ``Content-Encoding`` header — mirror bytes are already raw-deflate and
  the app inflates them itself; a server-side announcement would cause a
  double inflate;
- a 404 also carries the CORS header (otherwise the browser hides the error
  behind an opaque fetch failure);
- path traversal stays inside the served root;
- the access log records one line per request — the hermeticity evidence the
  ``Dump CDN mirror access log`` CI step greps;
- wrong arguments fail loudly with usage, never start a half-configured
  server.

The server runs as a real subprocess (``python3 serve-cdn-cors.py root port``),
the same invocation the ``Serve CDN mirror locally`` CI step uses. No network
beyond loopback, no keys, no prod access.
"""

from __future__ import annotations

import socket
import subprocess
import sys
import time
import urllib.error
import urllib.request
from pathlib import Path

import pytest

REPO_ROOT = Path(__file__).resolve().parents[2]
SCRIPT = REPO_ROOT / ".github" / "scripts" / "serve-cdn-cors.py"

# Fake raw-deflate payload (the real mirror files are pre-deflated bytes the
# app inflates itself — the exact bytes only matter for byte-exactness here).
DICT_BYTES = b"\x1f\x8b" + bytes(range(256)) * 64
DICT_REL = "dictionaries/sudachidict-20260723/dict.words"


def _free_port() -> int:
    with socket.socket() as sock:
        sock.bind(("127.0.0.1", 0))
        return sock.getsockname()[1]


@pytest.fixture(scope="module")
def mirror(tmp_path_factory):
    """Run the server as a subprocess against a tiny mirror; wait for it up."""
    root = tmp_path_factory.mktemp("mirror")
    dict_file = root / DICT_REL
    dict_file.parent.mkdir(parents=True)
    dict_file.write_bytes(DICT_BYTES)
    (root / "manifest.json").write_text('{"version": "test"}', encoding="utf-8")

    # Secret deliberately outside the served root: traversal must never
    # reach it regardless of the runner's filesystem layout.
    outside = tmp_path_factory.mktemp("outside")
    (outside / "secret.txt").write_text("must not leak", encoding="utf-8")

    port = _free_port()
    log_path = tmp_path_factory.mktemp("logs") / "access.log"
    with log_path.open("wb") as log:
        proc = subprocess.Popen(
            [sys.executable, str(SCRIPT), str(root), str(port)],
            stdout=subprocess.DEVNULL,
            stderr=log,
        )
    try:
        deadline = time.monotonic() + 10
        while True:
            try:
                _get(port, "manifest.json")
                break
            except (urllib.error.URLError, ConnectionError, OSError):
                if proc.poll() is not None:
                    raise AssertionError(
                        f"server died during startup: rc={proc.returncode}"
                    )
                if time.monotonic() > deadline:
                    raise AssertionError("server did not come up within 10s")
                time.sleep(0.1)
        yield {"port": port, "log_path": log_path}
    finally:
        proc.terminate()
        proc.wait(timeout=5)


def _get(port: int, rel: str) -> tuple[int, dict[str, str], bytes]:
    """GET a path with urllib; return (status, headers, body) including 404s."""
    request = urllib.request.Request(f"http://127.0.0.1:{port}/{rel}")
    try:
        with urllib.request.urlopen(request, timeout=10) as response:
            return response.status, dict(response.headers), response.read()
    except urllib.error.HTTPError as error:
        return error.code, dict(error.headers), error.read()


def _raw_status_line(port: int, path: str) -> str:
    """Send a request path verbatim over a raw socket (no URL normalization)."""
    with socket.create_connection(("127.0.0.1", port), timeout=10) as sock:
        sock.sendall(
            f"GET {path} HTTP/1.1\r\nHost: 127.0.0.1\r\nConnection: close\r\n\r\n".encode()
        )
        data = b""
        while chunk := sock.recv(65536):
            data += chunk
    return data.split(b"\r\n", 1)[0].decode()


# ---------------------------------------------------------------------------
# Main contract: the dictionary fetch the wasm suite makes
# ---------------------------------------------------------------------------


def test_dictionary_file_served_byte_exact_with_cors(mirror):
    port = mirror["port"]
    status, headers, body = _get(port, DICT_REL)
    assert status == 200
    # Harness page is a different origin — without ACAO the fetch dies
    # before the suite even sees a response.
    assert headers["Access-Control-Allow-Origin"] == "*"
    # Byte-exact: mirror files are the prod bytes, app-side inflate depends
    # on getting them unchanged.
    assert body == DICT_BYTES
    # No Content-Encoding: the server must not announce an encoding for
    # already-deflated bytes, or the app's own inflate step breaks.
    assert "Content-Encoding" not in headers


def test_serves_text_files_with_cors(mirror):
    port = mirror["port"]
    status, headers, body = _get(port, "manifest.json")
    assert status == 200
    assert headers["Access-Control-Allow-Origin"] == "*"
    assert body == b'{"version": "test"}'


# ---------------------------------------------------------------------------
# Negative branches
# ---------------------------------------------------------------------------


def test_missing_file_is_404_with_cors(mirror):
    # 404 must still carry ACAO: on a bare 404 the browser surfaces the app's
    # own error instead of swallowing it as an opaque CORS failure.
    port = mirror["port"]
    status, headers, _ = _get(port, "dictionaries/sudachidict-20260723/nope.bin")
    assert status == 404
    assert headers["Access-Control-Allow-Origin"] == "*"


@pytest.mark.parametrize(
    "path",
    [
        "/../secret.txt",
        "/../../secret.txt",
        "/..%2fsecret.txt",
        "/%2e%2e/secret.txt",
        "/dictionaries/../../secret.txt",
    ],
)
def test_traversal_stays_inside_root(mirror, path):
    port = mirror["port"]
    status_line = _raw_status_line(port, path)
    assert "404" in status_line, f"{path} escaped the root: {status_line}"


def test_wrong_arguments_fail_with_usage():
    proc = subprocess.run(
        [sys.executable, str(SCRIPT)],
        capture_output=True,
        text=True,
        timeout=10,
    )
    assert proc.returncode != 0
    assert "usage" in proc.stderr.lower()


# ---------------------------------------------------------------------------
# Access log: the hermeticity evidence the CI dump step greps
# ---------------------------------------------------------------------------


def test_access_log_records_every_request(mirror):
    # Read the log while the server is still running (CI dumps it after the
    # test step); lines must map one-to-one to requests with status codes.
    log = mirror["log_path"].read_text(encoding="utf-8", errors="replace")
    assert f'"GET /{DICT_REL} HTTP/1.1" 200' in log
    assert '"GET /manifest.json HTTP/1.1" 200' in log
    assert '"GET /dictionaries/sudachidict-20260723/nope.bin HTTP/1.1" 404' in log
