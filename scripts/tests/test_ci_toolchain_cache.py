"""Regression pins for the CI toolchain cache layer (task 20261110044319, PR #920).

The task moved CI toolchain downloads (cargo-binstall, trunk, tauri-cli,
wasm-pack) onto "pin + platform-qualified cache + retry". The layer lives in
workflow YAML, so nothing below is exercised by cargo/bun tests — CI itself is
the only runtime. These tests pin the cross-file contracts that keep the layer
from quietly degrading; each of them was a real review finding or the original
bug class:

1. Cache keys carry runner.os/arch qualifiers — the GitHub cache service does
   not partition by platform and the binaries are platform-specific. The
   first, unqualified attempt (04852d3e) let macOS jobs exact-hit a linux ELF
   (review round 2, High).
2. Every cache key family is seeded in cleanup-cache.yml MASTER_SEEDS — an
   unseeded family gets its master copies reaped as orphans after the grace
   period (delete/recreate cycle on every push to master).
3. The trunk pin lives in two places (setup-frontend action and the
   ARG TRUNK_VERSION of origa_ui/Dockerfile); a one-sided bump silently splits
   the CI and Docker builds — the exact drift (0.22.1 vs 0.21.14) this task
   was created to kill.
4. Binary names in cache paths match install reality (tauri-cli installs
   cargo-tauri) — a wrong path restores nothing and the cache never converges.
5. The source-build fallback keeps the pin (`cargo install --version <pin>`),
   and Docker release fetches carry --retry.

The tests read the live workflow files and re-use the real seed regex and the
real pin-resolution shell block, so either side of any contract going stale
turns pytest red before CI runs.
"""

from __future__ import annotations

import re
import shlex
import subprocess
from pathlib import Path

import yaml

_REPO_ROOT = Path(__file__).resolve().parents[2]
ACTION_FILE = _REPO_ROOT / ".github" / "actions" / "setup-frontend" / "action.yml"
CI_FILE = _REPO_ROOT / ".github" / "workflows" / "ci.yml"
CLEANUP_FILE = _REPO_ROOT / ".github" / "workflows" / "cleanup-cache.yml"
UI_DOCKERFILE = _REPO_ROOT / "origa_ui" / "Dockerfile"
LANDING_DOCKERFILE = _REPO_ROOT / "origa_landing" / "Dockerfile"

# Platform combinations the cleanup seeds accept and GitHub runners actually
# resolve (ubuntu-24.04 → Linux/X64, macos-15 → macOS/ARM64; Windows kept for
# the cross-compiling build-windows job consistency).
PLATFORM_QUALIFIERS = [
    (os_, arch) for os_ in ("Linux", "macOS", "Windows") for arch in ("X64", "ARM64")
]


def _load_yaml(path: Path) -> dict:
    return yaml.safe_load(path.read_text(encoding="utf-8"))


def _master_seeds() -> re.Pattern[str]:
    """Extract the live MASTER_SEEDS regex from the cleanup workflow script."""
    workflow = _load_yaml(CLEANUP_FILE)
    script = workflow["jobs"]["scheduled-cleanup"]["steps"][0]["run"]
    match = re.search(r"MASTER_SEEDS = re\.compile\((.*?)\)\n", script, re.DOTALL)
    assert match, "MASTER_SEEDS definition not found in cleanup-cache.yml"
    parts = re.findall(r'r"([^"]+)"', match.group(1))
    assert parts, "MASTER_SEEDS has no r-string parts"
    return re.compile("".join(parts))


def _action_steps() -> list[dict]:
    return _load_yaml(ACTION_FILE)["runs"]["steps"]


_FRONTEND_KEY_RE = r"frontend-tools-\$\{\{ runner\.os \}\}-\$\{\{ runner\.arch \}\}-(?P<tool>[a-z-]+)-v\d+"


def _frontend_tool_restores() -> dict[str, dict]:
    """Per-tool cache restores of the action: tool name -> restore step."""
    restores = {}
    for step in _action_steps():
        key = (step.get("with") or {}).get("key", "")
        match = re.fullmatch(_FRONTEND_KEY_RE, key)
        if match:
            restores[match.group("tool")] = step
    return restores


def _pinned_tools_run() -> str:
    for step in _action_steps():
        if step.get("name") == "Install pinned tools via cargo-binstall":
            return step["run"]
    raise AssertionError("pin registry step not found in action.yml")


def _wasm_test_steps() -> list[dict]:
    ci = _load_yaml(CI_FILE)
    return ci["jobs"]["wasm-test"]["steps"]


# --- Contract 1: platform-qualified keys (review round 2 High) ---------------


def test_frontend_tool_keys_are_platform_qualified():
    restores = _frontend_tool_restores()
    assert set(restores) == {"binstall", "trunk", "tauri-cli"}, (
        f"unexpected per-tool cache set: {sorted(restores)}"
    )
    for tool, step in restores.items():
        key = step["with"]["key"]
        assert "${{ runner.os }}" in key and "${{ runner.arch }}" in key, (
            f"{tool} cache key lost its platform qualifier: {key}"
        )


def test_every_frontend_cache_key_follows_the_tool_schema():
    # No frontend-tools key may exist outside the qualified tool schema:
    # a new shape would silently escape the seeding tests above.
    for step in _action_steps():
        key = (step.get("with") or {}).get("key", "")
        if key.startswith("frontend-tools-"):
            assert re.fullmatch(_FRONTEND_KEY_RE, key), (
                f"frontend-tools key outside the platform-qualified tool schema: {key}"
            )


def test_wasm_pack_key_is_platform_qualified():
    restore = next(s for s in _wasm_test_steps() if s.get("id") == "wasm-pack-cache")
    key = restore["with"]["key"]
    assert "${{ runner.os }}" in key and "${{ runner.arch }}" in key, (
        f"wasm-pack cache key lost its platform qualifier: {key}"
    )


# --- Contract 2: every key family is seeded (cleanup lifecycle) --------------


def test_every_frontend_tool_key_form_is_seeded():
    seeds = _master_seeds()
    for tool, step in _frontend_tool_restores().items():
        for os_, arch in PLATFORM_QUALIFIERS:
            key = (
                step["with"]["key"]
                .replace("${{ runner.os }}", os_)
                .replace("${{ runner.arch }}", arch)
            )
            assert seeds.fullmatch(key), f"unseeded cache key: {key}"


def test_wasm_pack_key_form_is_seeded():
    seeds = _master_seeds()
    restore = next(s for s in _wasm_test_steps() if s.get("id") == "wasm-pack-cache")
    for os_, arch in PLATFORM_QUALIFIERS:
        key = (
            restore["with"]["key"]
            .replace("${{ runner.os }}", os_)
            .replace("${{ runner.arch }}", arch)
        )
        assert seeds.fullmatch(key), f"unseeded cache key: {key}"


def test_seeds_do_not_match_stale_unqualified_keys():
    # The pre-qualification key shapes must stay unseeded: caches on PR refs
    # are reaped by the non-master TTL rule, and nothing should keep the
    # cross-platform-contaminated entries alive on master.
    seeds = _master_seeds()
    for stale in (
        "frontend-tools-binstall-v1",
        "frontend-tools-trunk-v1",
        "frontend-tools-tauri-cli-v1",
        "wasm-pack-v0.15.0",
    ):
        assert not seeds.fullmatch(stale), f"stale key shape got seeded: {stale}"


# --- Contract 3: cache paths match install reality (binaries, not crates) ----


def test_cache_paths_match_installed_binary_names():
    expected_paths = {
        "binstall": "~/.cargo/bin/cargo-binstall",
        "trunk": "~/.cargo/bin/trunk",
        "tauri-cli": "~/.cargo/bin/cargo-tauri",  # NOT tauri-cli (review round 1)
    }
    for tool, path in expected_paths.items():
        step = _frontend_tool_restores()[tool]
        assert step["with"]["path"] == path, (
            f"{tool} cache path drifted from the installed binary name: "
            f"{step['with']['path']}"
        )


# --- Contract 4: pins and the trunk dual-source sync -------------------------


def test_case_block_pins_are_versioned():
    run = _pinned_tools_run()
    pins = re.findall(r'^\s+\w+\) bin="[^"]+"; pin="([^"]+)"', run, re.MULTILINE)
    assert pins, "pin registry (case block) not found"
    for pin in pins:
        assert re.fullmatch(r"[a-z0-9_-]+@\d+\.\d+\.\d+", pin), (
            f"unpinned/unversioned tool in the pin registry: {pin}"
        )


def test_trunk_pin_synced_between_action_and_dockerfile():
    run = _pinned_tools_run()
    action_pin = re.search(r'trunk\) bin="trunk"; pin="trunk@(\d+\.\d+\.\d+)"', run)
    assert action_pin, "trunk pin not found in the action case block"
    dockerfile = UI_DOCKERFILE.read_text(encoding="utf-8")
    docker_arg = re.search(
        r"^ARG TRUNK_VERSION=(\d+\.\d+\.\d+)$", dockerfile, re.MULTILINE
    )
    assert docker_arg, "ARG TRUNK_VERSION not found in origa_ui/Dockerfile"
    assert action_pin.group(1) == docker_arg.group(1), (
        f"trunk pin drift: action has {action_pin.group(1)}, "
        f"Dockerfile has {docker_arg.group(1)}"
    )


def test_wasm_pack_pin_matches_release_url():
    steps = _wasm_test_steps()
    key = next(s for s in steps if s.get("id") == "wasm-pack-cache")["with"]["key"]
    install = next(s for s in steps if s.get("name") == "Install wasm-pack")
    key_version = re.search(r"-v(\d+\.\d+\.\d+)$", key)
    url = re.search(
        r"wasm-pack/releases/download/v(\d+\.\d+\.\d+)/wasm-pack-v\d+\.\d+\.\d+-([a-z0-9_-]+)\.tar\.gz",
        install["run"],
    )
    assert key_version and url, "wasm-pack pin not found in key or release URL"
    assert key_version.group(1) == url.group(1), (
        f"wasm-pack pin drift: cache key has {key_version.group(1)}, "
        f"release URL has {url.group(1)}"
    )
    # The job is ubuntu-only; the musl tarball must stay on the matching arch.
    assert url.group(2) == "x86_64-unknown-linux-musl"


def test_wasm_pack_install_is_miss_only_and_verify_is_unconditional():
    steps = _wasm_test_steps()
    install = next(s for s in steps if s.get("name") == "Install wasm-pack")
    assert "steps.wasm-pack-cache.outputs.cache-hit != 'true'" in install.get("if", "")
    verify = next(s for s in steps if s.get("name") == "Verify wasm-pack")
    assert "if" not in verify, "verify step must run on every pass, not only on miss"
    assert "wasm-pack --version" in verify["run"]


# --- Contract 5: retry and pin-keeping on the failure paths ------------------


def test_fallback_keeps_pin():
    """Execute the live pin-resolution block for pinned and bare pins."""
    run = _pinned_tools_run()
    block = re.search(
        r'(crate="\$\{pin%@\*\}"\n\s*case "\$pin" in\n(?:.*\n)*?\s*esac)', run
    )
    assert block, "pin-resolution block not found in action.yml"

    def resolve(pin: str) -> tuple[str, list[str]]:
        script = (
            "pin="
            + shlex.quote(pin)
            + "\n"
            + block.group(1)
            + "\n"
            + "printf '%s\\n' \"crate=$crate\"\n"
            + "printf '%s\\n' \"ver=${ver[*]+${ver[*]}}\"\n"
        )
        proc = subprocess.run(
            ["bash", "-c", script],
            capture_output=True,
            text=True,
            timeout=30,
            check=True,
        )
        crate = re.search(r"^crate=(.*)$", proc.stdout, re.MULTILINE)
        ver_line = re.search(r"^ver=(.*)$", proc.stdout, re.MULTILINE)
        assert crate and ver_line, f"unexpected resolver output:\n{proc.stdout}"
        return crate.group(1), ver_line.group(1).split()

    assert resolve("trunk@0.21.14") == ("trunk", ["--version", "0.21.14"])
    assert resolve("tauri-cli@2.12.4") == ("tauri-cli", ["--version", "2.12.4"])
    assert resolve("some-future-tool") == ("some-future-tool", [])

    assert 'cargo install "$crate" --locked --force "${ver[@]}"' in run, (
        "source-build fallback must keep the pin via ${ver[@]}"
    )


def test_dockerfile_release_fetches_have_retry():
    for dockerfile in (UI_DOCKERFILE, LANDING_DOCKERFILE):
        text = dockerfile.read_text(encoding="utf-8")
        for url in re.findall(
            r"curl [^\n]*(?:releases/download|sh\.rustup\.rs)[^\n]*", text
        ):
            assert "--retry" in url and "--retry-all-errors" in url, (
                f"{dockerfile.name}: release fetch without --retry:\n{url.strip()}"
            )
