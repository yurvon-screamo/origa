"""Unit tests for the store-asset publisher (``publish_media``).

The publisher moves the store assets out of the repository and into the CDN,
and keeps ``docs/media/store-assets.json`` as their address book. The tests
pin the parts that can silently go wrong: a key that does not carry the hash
of the file (the edge cache would serve an old picture under a new URL), an
unsafe key (the aws CLI re-parses its arguments on Windows), a registry that
loses the upload timestamp of an unchanged asset, and a personal-data flag
that nobody checked file by file.

The tests are pure: no S3, no network, no git write. The git calls are
read-only and run against the real repository history.
"""

from __future__ import annotations

import struct
import sys
import zlib
from pathlib import Path

import pytest

import publish_media
from _cdn_cache import IMMUTABLE, cache_control_for
from _media_registry import (
    Asset,
    AssetGroup,
    GROUPS,
    PERSONAL_DATA,
    REGISTRY_RELPATH,
    RegistrySchemaError,
    assets_from_registry,
    build_key,
    locale_for,
    merge_assets,
    png_size,
    read_registry,
    write_registry,
)

REPO_ROOT = Path(__file__).resolve().parents[2]


def _png_bytes(width: int = 3, height: int = 2) -> bytes:
    """A minimal valid PNG, so the size reader is tested against a real file."""

    def chunk(kind: bytes, payload: bytes) -> bytes:
        body = kind + payload
        return (
            struct.pack(">I", len(payload)) + body + struct.pack(">I", zlib.crc32(body))
        )

    header = struct.pack(">IIBBBBB", width, height, 8, 2, 0, 0, 0)
    raw = b"".join(b"\x00" + b"\xff\x00\x00" * width for _ in range(height))
    return (
        b"\x89PNG\r\n\x1a\n"
        + chunk(b"IHDR", header)
        + chunk(b"IDAT", zlib.compress(raw))
        + chunk(b"IEND", b"")
    )


def _asset(asset_id: str = "media/x/y", key: str = "media/x/y-aaaaaaaa.png") -> Asset:
    return Asset(
        id=asset_id,
        kind="app-store-screenshot",
        channel="app-store-iphone-en",
        locale="en",
        slug="y",
        width=10,
        height=20,
        bytes=30,
        sha256="a" * 64,
        key=key,
        url=f"https://s3.origa.uwuwu.net/{key}",
        uploadedAt="2026-10-22T00:00:00+00:00",
        uploadedSha256="a" * 64,
    )


# --- the key -----------------------------------------------------------------


def test_key_carries_the_hash_of_the_file() -> None:
    key = build_key("media/app-store-iphone-en", "1_hero", "0123456789abcdef")

    assert key == "media/app-store-iphone-en/1_hero-01234567.png"


def test_the_same_bytes_keep_the_same_key() -> None:
    first = build_key("media/social-poster", "poster", "0123456789abcdef")
    second = build_key("media/social-poster", "poster", "0123456789abcdef")

    assert first == second


def test_changed_bytes_get_a_different_key() -> None:
    first = build_key("media/social-poster", "poster", "0123456789abcdef")
    second = build_key("media/social-poster", "poster", "fedcba9876543210")

    assert first != second


def test_every_published_prefix_is_immutable() -> None:
    for known_group in GROUPS:
        key = build_key(known_group.prefix, "1_hero", "0123456789abcdef")

        assert cache_control_for(key) == IMMUTABLE


def test_a_key_outside_the_known_prefixes_is_not_immutable() -> None:
    # A blanket "media/" rule would make every future object under that
    # prefix immutable, including content that does not carry a hash.
    assert cache_control_for("media/whatever/new-thing.json") != IMMUTABLE


# --- the unsafe key ----------------------------------------------------------


def test_an_unsafe_key_is_refused_before_anything_is_uploaded() -> None:
    # The check runs before the upload loop: a bad asset in the middle of the
    # set would otherwise leave its uploaded siblings without a registry row.
    good = _asset()
    evil = Asset(
        **{
            **good.to_json(),
            "id": "media/x/evil",
            "slug": "evil; calc",
            "key": "media/x/evil; calc-bbbbbbbb.png",
        }
    )

    with pytest.raises(publish_media.UnsafeKeyError):
        publish_media.preflight([good, evil])


def test_a_safe_set_passes_the_preflight() -> None:
    good = _asset()

    publish_media.preflight([good])


def test_a_dry_run_never_touches_the_transport_or_the_registry(
    monkeypatch: pytest.MonkeyPatch, tmp_path: Path
) -> None:
    # A dry run is a plan, not a state change: no upload, and no registry row
    # that claims "uploaded" for an object the CDN has never seen. The source
    # must be a real directory with a real PNG — an empty set exits before
    # the dry-run branch and the test would prove nothing.
    staging = tmp_path / "store-assets"
    staging.mkdir()
    _write_tmp(staging, "shot.png", _png_bytes())
    uploaded: list[object] = []
    monkeypatch.setattr(
        publish_media, "upload_file", lambda *args, **kwargs: uploaded.append(args)
    )
    registry = tmp_path / "registry.json"
    monkeypatch.setattr(
        sys,
        "argv",
        [
            "publish_media.py",
            "--dry-run",
            "--source",
            str(staging),
            "--prefix",
            "media/app-store-iphone-en",
            "--kind",
            "app-store-screenshot",
            "--channel",
            "app-store-iphone-en",
            "--registry",
            str(registry),
        ],
    )

    exit_code = publish_media.main()

    assert exit_code == 0
    assert uploaded == []
    assert not registry.exists()


def test_a_partial_upload_still_records_what_reached_the_cdn(
    monkeypatch: pytest.MonkeyPatch, tmp_path: Path
) -> None:
    registry = tmp_path / "registry.json"
    good = _asset()
    first = _png_bytes()
    second = _png_bytes(width=4, height=3)
    items = [
        (good, _write_tmp(tmp_path, "good.png", first)),
        (
            Asset(
                **{
                    **good.to_json(),
                    "id": "media/x/broken",
                    "slug": "broken",
                    "sha256": "c" * 64,
                    "key": "media/x/broken-cccccccc.png",
                }
            ),
            _write_tmp(tmp_path, "broken.png", second),
        ),
    ]

    def fail_on_second(*args: object, **kwargs: object) -> None:
        if len(published_seen) >= 1:
            raise SystemExit(1)
        published_seen.append(args)

    published_seen: list[object] = []
    monkeypatch.setattr(publish_media, "upload_file", fail_on_second)

    with pytest.raises(SystemExit):
        publish_media.upload_all(items, published := [])

    publish_media._write(registry, published, "2026-10-22T00:00:00+00:00")
    restored = assets_from_registry(read_registry(registry))

    assert [asset.id for asset in restored] == [good.id]


def _write_tmp(directory: Path, name: str, payload: bytes) -> Path:
    path = directory / name
    path.write_bytes(payload)
    return path


# --- the ad-hoc mode ---------------------------------------------------------


def test_the_ad_hoc_group_refuses_an_unknown_kind_and_channel() -> None:
    with pytest.raises(ValueError, match="kind"):
        publish_media.adhoc_group("store-assets/x", "media/x", "nope", "marketing")
    with pytest.raises(ValueError, match="channel"):
        publish_media.adhoc_group("store-assets/x", "media/x", "store-creative", "nope")


def test_an_ad_hoc_asset_carries_no_legacy_path(tmp_path: Path) -> None:
    # A staging file was never in git: a legacyPath would point at a working
    # directory, and the registry guard would go red on a clean checkout.
    group = publish_media.adhoc_group(
        str(tmp_path), "media/x", "store-creative", "marketing", "en"
    )
    source = _write_tmp(tmp_path, "shot.png", _png_bytes())

    asset, path = publish_media.build_asset(REPO_ROOT, group, source, "now")

    assert asset.legacyPath == ""
    assert asset.addedAt == ""
    assert asset.sourceCommit == ""
    assert path == source


def test_an_ad_hoc_source_is_accepted_from_anywhere(tmp_path: Path) -> None:
    # The staging directory may sit outside the repository; the publisher
    # must not crash with a traceback on such a path.
    group = publish_media.adhoc_group(
        str(tmp_path), "media/x", "store-creative", "marketing", "en"
    )
    source = _write_tmp(tmp_path, "shot.png", _png_bytes())

    items = publish_media.collect(REPO_ROOT, group, "now")

    assert [asset.id for asset, _ in items] == ["media/x/shot"]
    assert items[0][1] == source


# --- the schema guard --------------------------------------------------------


def test_a_registry_of_another_schema_is_refused(tmp_path: Path) -> None:
    path = tmp_path / "registry.json"
    write_registry(path, [_asset()], "2026-10-22T00:00:00+00:00")
    payload = read_registry(path)
    payload["schema"] = 99
    path.write_text(__import__("json").dumps(payload), encoding="utf-8")

    with pytest.raises(RegistrySchemaError, match="schema"):
        assets_from_registry(read_registry(path))


def test_an_unknown_registry_field_is_an_error_not_a_silent_drop(
    tmp_path: Path,
) -> None:
    path = tmp_path / "registry.json"
    write_registry(path, [_asset()], "2026-10-22T00:00:00+00:00")
    payload = read_registry(path)
    entry = payload["assets"][0]
    assert isinstance(entry, dict)
    entry["reviewedBy"] = "someone"
    path.write_text(__import__("json").dumps(payload), encoding="utf-8")

    with pytest.raises(RegistrySchemaError, match="unknown fields"):
        assets_from_registry(read_registry(path))


# --- the registry ------------------------------------------------------------


def test_an_unchanged_asset_keeps_its_upload_timestamp() -> None:
    previous = _asset()
    incoming = Asset(
        **{**previous.to_json(), "uploadedAt": "2026-11-01T00:00:00+00:00"}
    )

    merged = merge_assets([previous], [incoming])

    assert merged[0].uploadedAt == "2026-10-22T00:00:00+00:00"


def test_a_changed_asset_gets_a_new_timestamp() -> None:
    previous = _asset()
    incoming = Asset(
        **{
            **previous.to_json(),
            "key": "media/x/y-bbbbbbbb.png",
            "uploadedAt": "2026-11-01T00:00:00+00:00",
        }
    )

    merged = merge_assets([previous], [incoming])

    assert merged[0].uploadedAt == "2026-11-01T00:00:00+00:00"
    assert merged[0].key == "media/x/y-bbbbbbbb.png"


def test_the_registry_round_trips(tmp_path: Path) -> None:
    path = tmp_path / "registry.json"

    write_registry(path, [_asset()], "2026-10-22T00:00:00+00:00")
    restored = assets_from_registry(read_registry(path))

    assert len(restored) == 1
    assert restored[0].id == "media/x/y"
    assert restored[0].sha256 == "a" * 64


def test_an_absent_registry_reads_as_empty(tmp_path: Path) -> None:
    data = read_registry(tmp_path / "missing.json")

    assert assets_from_registry(data) == []


# --- the personal-data flag --------------------------------------------------


def test_the_personal_data_table_holds_only_published_files() -> None:
    # After the cleanup (ADR-068) the source binaries are gone from the tree,
    # so the set is checked against the registry: every flagged path must be
    # a path the publisher actually published.
    registry = assets_from_registry(read_registry(REPO_ROOT / REGISTRY_RELPATH))
    legacy_paths = {asset.legacyPath for asset in registry}

    assert legacy_paths, "the registry is empty — nothing was published"
    assert PERSONAL_DATA <= legacy_paths


def test_the_flag_is_set_for_the_assets_that_carry_an_account() -> None:
    # The list is the result of a per-file review (ADR-068). A wrong entry
    # here is a privacy defect, so the reviewed set is pinned.
    assert PERSONAL_DATA == {
        ".media/store/device_en/1_hero.png",
        ".media/store/device/1_hero.png",
        ".media/store/en/1_hero.png",
        ".media/store/en/7_phrase.png",
        ".media/store/ru/1_hero.png",
        ".media/store/ru/7_phrase.png",
        ".media/screenshots/ipad/home.png",
        ".media/banners/feature_graphics/feature-graphic-en-all-in-one-1024x500.png",
        ".media/banners/feature_graphics/feature-graphic-ru-all-in-one-1024x500.png",
        ".media/banners/en.all_in_one.png",
        ".media/banners/ru.all_in_one.png",
    }


# --- the provenance ----------------------------------------------------------


def test_the_git_history_gives_the_date_the_asset_entered_the_repository() -> None:
    added_at, commit = publish_media.added_at_and_commit(
        REPO_ROOT, ".media/store/device_en/1_hero.png"
    )

    assert commit.startswith("61b0b3c2")
    assert added_at.startswith("2026-08-17")


def test_the_drift_label_follows_the_distance() -> None:
    assert (
        publish_media.drift_label(
            "2026-08-17T00:00:00+00:00", "2026-08-20T00:00:00+00:00"
        )
        == "fresh"
    )
    assert (
        publish_media.drift_label(
            "2026-08-17T00:00:00+00:00", "2026-09-20T00:00:00+00:00"
        )
        == "1-month"
    )
    assert (
        publish_media.drift_label(
            "2026-08-17T00:00:00+00:00", "2026-10-22T00:00:00+00:00"
        )
        == "2-months"
    )
    assert publish_media.drift_label("", "") == "unknown"


def test_the_locale_comes_from_the_group_or_the_file_name() -> None:
    fixed = AssetGroup(
        source=".",
        prefix="media/app-store-iphone-ru",
        kind="app-store-screenshot",
        channel="app-store-iphone-ru",
        locale="ru",
    )
    derived = AssetGroup(
        source=".",
        prefix="media/marketing-visual",
        kind="marketing-visual",
        channel="marketing",
    )

    assert locale_for(fixed, "1_hero.png") == "ru"
    assert locale_for(derived, "en.all_in_one.png") == "en"
    assert locale_for(derived, "ru.your_content.png") == "ru"
    assert locale_for(derived, "cleaned_logo.png") is None


# --- the png reader ----------------------------------------------------------


def test_the_png_size_is_read_from_the_header(tmp_path: Path) -> None:
    path = tmp_path / "sample.png"
    path.write_bytes(_png_bytes(width=7, height=5))

    assert png_size(path) == (7, 5)


def test_a_non_png_file_is_refused(tmp_path: Path) -> None:
    path = tmp_path / "sample.png"
    path.write_bytes(b"not a png")

    with pytest.raises(ValueError):
        png_size(path)
