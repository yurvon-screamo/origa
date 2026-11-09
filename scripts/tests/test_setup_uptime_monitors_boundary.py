"""Boundary tests for the HetrixTools API surface of ``setup_uptime_monitors``.

The pure unit tests (``test_setup_uptime_monitors.py``) pin payload shapes and
diff logic. This module pins the HTTP boundary itself, with ``urlopen``
monkeypatched — no network:

* the v3 read path hits ``/v3/uptime-monitors`` with a Bearer header and
  parses the flat ``monitors`` listing;
* API failures on read exit 2 before any write happens;
* the v2 write puts the token in the URL path (per the vendor docs) and never
  in headers or the JSON body;
* a write failure surfaces as a counted failure (exit 1) and the token must
  not leak into any output (the token rides in the URL path, so it must never
  be logged or echoed).
"""

from __future__ import annotations

import io
import json
import sys
import urllib.error
import urllib.request
from contextlib import nullcontext
from pathlib import Path

import pytest

sys.path.insert(0, str(Path(__file__).resolve().parent.parent))

import setup_uptime_monitors as setup

V3_TOKEN = "v3-account-key"  # nosemgrep: python-hardcoded-secrets (test fixture, not a credential)
V2_TOKEN = "v2-uptime-token"  # nosemgrep: python-hardcoded-secrets (test fixture, not a credential)


class UrlopenRecorder:
    """Monkeypatched urlopen: scripted payload, recorded requests.

    Returns a ``nullcontext`` around ``BytesIO`` — the same shape the real
    urlopen exposes to the ``with`` block in the setup script.
    """

    def __init__(self) -> None:
        self.requests: list[urllib.request.Request] = []
        self.payload: bytes | None = None
        self.error: Exception | None = None

    def __call__(
        self, request: urllib.request.Request, timeout: float = 0
    ) -> nullcontext[io.BytesIO]:
        assert isinstance(request, urllib.request.Request)
        self.requests.append(request)
        if self.error is not None:
            raise self.error
        assert self.payload is not None, "script a payload or an error"
        return nullcontext(io.BytesIO(self.payload))


@pytest.fixture
def recorder(monkeypatch: pytest.MonkeyPatch) -> UrlopenRecorder:
    rec = UrlopenRecorder()
    monkeypatch.setattr(setup.urllib.request, "urlopen", rec)
    return rec


def test_v3_read_hits_uptime_monitors_with_bearer(recorder: UrlopenRecorder) -> None:
    recorder.payload = json.dumps({"monitors": [{"name": "x", "id": "1"}]}).encode()
    monitors = setup.fetch_existing_monitors(V3_TOKEN)
    assert monitors == [{"name": "x", "id": "1"}]
    (request,) = recorder.requests
    assert request.full_url == f"{setup.V3_BASE}/uptime-monitors"
    assert request.get_header("Authorization") == f"Bearer {V3_TOKEN}"


def test_v3_read_failure_exits_2(recorder: UrlopenRecorder) -> None:
    recorder.error = urllib.error.URLError("connection reset")
    with pytest.raises(SystemExit) as raised:
        setup.fetch_existing_monitors(V3_TOKEN)
    assert raised.value.code == 2


def test_v2_write_carries_token_in_url_path_only(recorder: UrlopenRecorder) -> None:
    recorder.payload = b'{"status": "success"}'
    result = setup.v2_call(V2_TOKEN, "add", {"Name": "x"})
    assert result == {"status": "success"}
    (request,) = recorder.requests
    assert request.full_url == f"{setup.V2_BASE}/{V2_TOKEN}/uptime/add/"
    assert request.get_method() == "POST"
    # The token rides in the URL path per the vendor docs — nowhere else.
    assert V2_TOKEN not in json.dumps(dict(request.header_items()))
    body = request.data.decode()
    assert V2_TOKEN not in body
    assert json.loads(body) == {"Name": "x"}


def test_v2_write_failure_exits_2(recorder: UrlopenRecorder) -> None:
    recorder.error = urllib.error.URLError("timeout")
    with pytest.raises(SystemExit) as raised:
        setup.v2_call(V2_TOKEN, "edit", {"Name": "x"})
    assert raised.value.code == 2


def test_execute_plan_counts_refused_writes(
    recorder: UrlopenRecorder, capsys: pytest.CaptureFixture[str]
) -> None:
    monitor = setup.DESIRED_MONITORS[0]
    in_sync = setup.DESIRED_MONITORS[1]
    actions = [
        setup.PlanAction("skip", in_sync, "42", "in sync"),
        setup.PlanAction("add", monitor),
    ]
    recorder.payload = b'{"status": "error", "message": "nope"}'
    failures = setup.execute_plan(actions, V2_TOKEN)
    assert failures == 1
    out, err = capsys.readouterr()
    assert "skip" in out and "in sync" in out
    assert "API refused" in err
    # The token lives in the URL path: a refusal must never echo it.
    assert V2_TOKEN not in out
    assert V2_TOKEN not in err


def test_execute_plan_succeeds_on_vendor_ok_shape(
    recorder: UrlopenRecorder, capsys: pytest.CaptureFixture[str]
) -> None:
    # The v2 API acknowledges adds with {"status": "success"}.
    recorder.payload = b'{"status": "success"}'
    actions = [setup.PlanAction("add", setup.DESIRED_MONITORS[2])]
    assert setup.execute_plan(actions, V2_TOKEN) == 0
    assert "ok" in capsys.readouterr().out
