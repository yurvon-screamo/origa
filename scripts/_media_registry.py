"""Registry of the store assets published to the CDN (ADR-068).

The store assets — App Store screenshots, Play feature graphics, marketing
creatives — used to live as binaries in ``.media/`` and were consumed by hand:
a person downloaded them from the repository and uploaded them into the store
console. The registry keeps the metadata of every published asset (the CDN
link, the hash, the provenance) so the binary itself can leave the repository
without losing its address.

The registry is the successor of the hand-written ``.media/README.md`` map:
every field a consumer needs is machine-readable, and
``scripts/tests/test_media_map.py`` fails the build when the map and the
repository drift apart.

Two vocabularies meet here and must not be mixed up:

- ``capturedAt`` — when the picture was taken. For the store assets the date
  of the capture is UNKNOWN (the files arrived as binaries), so the field is
  ``null`` and ``capturedAtSource`` says so. The date the file entered the
  repository lives in ``addedAt`` (the git commit that added it).
- ``capturedAt`` in ``docs/ui-screenshots/shots/*.json`` (ADR-067) means the
  Playwright capture timestamp — a different thing under the same name. The
  two registries are deliberately separate; this one never claims a capture
  date it cannot prove.
"""

from __future__ import annotations

import hashlib
import json
import struct
from dataclasses import asdict, dataclass
from pathlib import Path
from typing import Final

SCHEMA: Final[int] = 1
REGISTRY_RELPATH: Final[str] = "docs/media/store-assets.json"

# --- vocabularies -----------------------------------------------------------

KIND_APP_STORE_SCREENSHOT: Final[str] = "app-store-screenshot"
KIND_STORE_CREATIVE: Final[str] = "store-creative"
KIND_PLAY_FEATURE_GRAPHIC: Final[str] = "play-feature-graphic"
KIND_SOCIAL_POSTER: Final[str] = "social-poster"
KIND_MARKETING_VISUAL: Final[str] = "marketing-visual"

KINDS: Final[frozenset[str]] = frozenset(
    {
        KIND_APP_STORE_SCREENSHOT,
        KIND_STORE_CREATIVE,
        KIND_PLAY_FEATURE_GRAPHIC,
        KIND_SOCIAL_POSTER,
        KIND_MARKETING_VISUAL,
    }
)

CHANNEL_APP_STORE_IPHONE_EN: Final[str] = "app-store-iphone-en"
CHANNEL_APP_STORE_IPHONE_RU: Final[str] = "app-store-iphone-ru"
CHANNEL_APP_STORE_IPAD: Final[str] = "app-store-ipad"
CHANNEL_PLAY: Final[str] = "play"
CHANNEL_SOCIAL: Final[str] = "social"
CHANNEL_MARKETING: Final[str] = "marketing"

CHANNELS: Final[frozenset[str]] = frozenset(
    {
        CHANNEL_APP_STORE_IPHONE_EN,
        CHANNEL_APP_STORE_IPHONE_RU,
        CHANNEL_APP_STORE_IPAD,
        CHANNEL_PLAY,
        CHANNEL_SOCIAL,
        CHANNEL_MARKETING,
    }
)

LOCALES: Final[tuple[str, ...]] = ("en", "ru")

# --- asset groups ------------------------------------------------------------


@dataclass(frozen=True)
class AssetGroup:
    """A set of files that share a CDN prefix, a kind and a channel.

    ``locale`` is the locale of the whole group; when it is ``None`` the
    locale is derived from the file name (``en.all_in_one.png`` → ``en``).

    ``legacy`` marks the migration groups: their files sit in the repository
    and the registry keeps their path as a historical pointer. A group
    published from the staging directory has ``legacy=False`` — its files
    were never in git, so there is nothing to point at.
    """

    source: str
    prefix: str
    kind: str
    channel: str
    locale: str | None = None
    pattern: str = "*.png"
    legacy: bool = True


GROUPS: Final[tuple[AssetGroup, ...]] = (
    AssetGroup(
        source=".media/store/device_en",
        prefix="media/app-store-iphone-en",
        kind=KIND_APP_STORE_SCREENSHOT,
        channel=CHANNEL_APP_STORE_IPHONE_EN,
        locale="en",
    ),
    AssetGroup(
        source=".media/store/device",
        prefix="media/app-store-iphone-ru",
        kind=KIND_APP_STORE_SCREENSHOT,
        channel=CHANNEL_APP_STORE_IPHONE_RU,
        locale="ru",
    ),
    AssetGroup(
        source=".media/screenshots/ipad",
        prefix="media/app-store-ipad",
        kind=KIND_APP_STORE_SCREENSHOT,
        channel=CHANNEL_APP_STORE_IPAD,
        locale="en",
    ),
    AssetGroup(
        source=".media/store/en",
        prefix="media/store-creative-en",
        kind=KIND_STORE_CREATIVE,
        channel=CHANNEL_MARKETING,
        locale="en",
    ),
    AssetGroup(
        source=".media/store/ru",
        prefix="media/store-creative-ru",
        kind=KIND_STORE_CREATIVE,
        channel=CHANNEL_MARKETING,
        locale="ru",
    ),
    AssetGroup(
        source=".media/banners/feature_graphics",
        prefix="media/play-feature-graphic",
        kind=KIND_PLAY_FEATURE_GRAPHIC,
        channel=CHANNEL_PLAY,
    ),
    AssetGroup(
        source=".media/store",
        prefix="media/social-poster",
        kind=KIND_SOCIAL_POSTER,
        channel=CHANNEL_SOCIAL,
        pattern="poster-*.png",
    ),
    AssetGroup(
        source=".media/banners",
        prefix="media/marketing-visual",
        kind=KIND_MARKETING_VISUAL,
        channel=CHANNEL_MARKETING,
    ),
)

# --- personal data -----------------------------------------------------------

# Assets whose picture carries the name of an account or the learning
# statistics of one. The list is the result of a per-file review of every
# published asset (2026-10-22) — not of the folder they live in: the
# ``store-creative`` set mixes captures of the dashboard (name + counters)
# with captures of a single card (neither).
#
# Marking an asset is safe; failing to mark one is not. The registry keeps the
# flag so the next store listing can be prepared from a neutral account.
PERSONAL_DATA: Final[frozenset[str]] = frozenset(
    {
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
)


# --- asset record ------------------------------------------------------------


@dataclass(frozen=True)
class Asset:
    """One published asset, exactly as the registry stores it."""

    id: str
    kind: str
    channel: str
    locale: str | None
    slug: str
    width: int
    height: int
    bytes: int
    sha256: str
    key: str
    url: str
    capturedAt: None = None
    capturedAtSource: str = "unknown"
    addedAt: str = ""
    sourceCommit: str = ""
    uiLastChangeAt: str = ""
    uiDriftCommits: int = 0
    uiDrift: str = "unknown"
    personalData: bool = False
    uploadedAt: str = ""
    uploadedSha256: str = ""
    legacyPath: str = ""

    def to_json(self) -> dict[str, object]:
        return asdict(self)


# --- helpers -----------------------------------------------------------------


def sha256_of(path: Path) -> str:
    """SHA256 of a file, streamed so a large asset does not land in memory."""
    digest = hashlib.sha256()
    with path.open("rb") as handle:
        for chunk in iter(lambda: handle.read(1 << 20), b""):
            digest.update(chunk)
    return digest.hexdigest()


def png_size(path: Path) -> tuple[int, int]:
    """Width and height of a PNG, read from the IHDR chunk.

    The size is a registry fact (a store console rejects an asset whose
    dimensions do not fit), so it is measured, not remembered.
    """
    with path.open("rb") as handle:
        header = handle.read(33)
    if header[:8] != b"\x89PNG\r\n\x1a\n":
        raise ValueError(f"not a PNG file: {path}")
    width, height = struct.unpack(">II", header[16:24])
    return width, height


def build_key(prefix: str, slug: str, sha256: str, suffix: str = ".png") -> str:
    """Content-addressed CDN key: the hash in the name keeps the URL stable.

    The same bytes keep the same key (and therefore the same URL), a changed
    asset gets a new one. That is the doctrine of ``scripts/_cdn_cache.py``:
    ``immutable`` is only correct for a content-addressed object.
    """
    return f"{prefix}/{slug}-{sha256[:8]}{suffix}"


def locale_for(group: AssetGroup, file_name: str) -> str | None:
    """The locale of a file: the group's own, or the one in its name."""
    if group.locale is not None:
        return group.locale
    tokens = {
        token
        for token in file_name.replace(".", "-").replace("_", "-").split("-")
        if token
    }
    for candidate in LOCALES:
        if candidate in tokens:
            return candidate
    return None


class RegistrySchemaError(Exception):
    """The registry on disk does not match the schema this script speaks."""


def read_registry(path: Path) -> dict[str, object]:
    """Read the registry file; an absent file is an empty registry.

    A registry written by another schema version is refused loudly: writing
    it back with this script would silently drop the fields it does not know.
    """
    if not path.exists():
        return {"schema": SCHEMA, "generatedAt": "", "assets": []}
    data = json.loads(path.read_text(encoding="utf-8"))
    if not isinstance(data, dict):
        raise RegistrySchemaError(f"registry is not an object: {path}")
    schema = data.get("schema")
    if schema != SCHEMA:
        raise RegistrySchemaError(
            f"registry schema is {schema!r}, this script speaks {SCHEMA!r}: {path}"
        )
    return data


def write_registry(path: Path, assets: list[Asset], generated_at: str) -> None:
    """Write the registry, sorted by id so the diff is stable."""
    payload = {
        "schema": SCHEMA,
        "generatedAt": generated_at,
        "assets": [asset.to_json() for asset in sorted(assets, key=lambda a: a.id)],
    }
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(
        json.dumps(payload, ensure_ascii=False, indent=2) + "\n", encoding="utf-8"
    )


def merge_assets(existing: list[Asset], incoming: list[Asset]) -> list[Asset]:
    """Merge the published assets into the registry, keyed by ``id``.

    An asset whose key did not change keeps its ``uploadedAt`` — the picture
    in the CDN is the same one, only the metadata around it was refreshed.
    """
    by_id = {asset.id: asset for asset in existing}
    for asset in incoming:
        previous = by_id.get(asset.id)
        if previous is not None and previous.key == asset.key:
            asset = Asset(
                **{
                    **asset.to_json(),
                    "uploadedAt": previous.uploadedAt,
                    "uploadedSha256": previous.uploadedSha256,
                }
            )
        by_id[asset.id] = asset
    return list(by_id.values())


def assets_from_registry(data: dict[str, object]) -> list[Asset]:
    """Rebuild the asset records from a raw registry payload.

    A field this version does not know is an error, not something to drop:
    a silent drop would erase it on the next write.
    """
    raw = data.get("assets")
    if not isinstance(raw, list):
        return []
    known = set(Asset.__dataclass_fields__)
    assets: list[Asset] = []
    for entry in raw:
        if not isinstance(entry, dict):
            continue
        unknown = sorted(set(entry) - known)
        if unknown:
            raise RegistrySchemaError(
                f"registry entry {entry.get('id', '?')} has unknown fields "
                f"{unknown}: update the script before touching this registry"
            )
        assets.append(Asset(**entry))
    return assets


__all__ = [
    "Asset",
    "AssetGroup",
    "CHANNELS",
    "GROUPS",
    "KINDS",
    "LOCALES",
    "PERSONAL_DATA",
    "REGISTRY_RELPATH",
    "RegistrySchemaError",
    "SCHEMA",
    "assets_from_registry",
    "build_key",
    "locale_for",
    "merge_assets",
    "png_size",
    "read_registry",
    "sha256_of",
    "write_registry",
]
