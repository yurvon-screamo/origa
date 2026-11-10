"""Unit tests for ``subset_fonts.sync_preload_links``.

The function keeps ``origa_ui/index.html`` font links aligned with freshly
produced subset filenames — the drift source behind the font-404 incidents
(June 2026 CI, Nov 2026 prod). Tests are pure: tmp_path fixtures, no ``cdn/``
dependency, no network.
"""

from __future__ import annotations

from pathlib import Path

from subset_fonts import sync_preload_links

SAMPLE_HTML = """<!doctype html>
<html>
  <head>
    <link
      rel="preload"
      href="https://cdn.example/fonts/noto-sans-jp-400-aaaaaaaa.woff2"
      as="font"
    />
    <link
      rel="preload"
      href="https://cdn.example/fonts/cormorant-garamond-87e70f35.woff2"
      as="font"
    />
    <link
      rel="preload"
      href="https://cdn.example/fonts/cormorant-garamond-italic-bbbbbbbb.woff2"
      as="font"
    />
    <link rel="stylesheet" href="/style.css" />
    <p>Plain mention: noto-sans-jp-400-cccccccc.woff2</p>
  </head>
</html>
"""


def _write_sample(tmp_path: Path) -> Path:
    page = tmp_path / "index.html"
    page.write_text(SAMPLE_HTML, encoding="utf-8")
    return page


def test_stale_hash_of_the_same_logical_is_rewritten(tmp_path):
    # Arrange
    page = _write_sample(tmp_path)

    # Act
    changed = sync_preload_links(["fonts/noto-sans-jp-400-41a4227e.woff2"], page)

    # Assert
    assert changed == ["noto-sans-jp-400-41a4227e.woff2"]
    html = page.read_text(encoding="utf-8")
    assert 'fonts/noto-sans-jp-400-41a4227e.woff2"' in html
    assert "noto-sans-jp-400-aaaaaaaa.woff2" not in html


def test_sibling_logical_name_keeps_its_own_hash(tmp_path):
    # Arrange
    page = _write_sample(tmp_path)

    # Act
    sync_preload_links(["fonts/cormorant-garamond-188e64d1.woff2"], page)

    # Assert
    html = page.read_text(encoding="utf-8")
    assert "cormorant-garamond-188e64d1.woff2" in html
    assert "cormorant-garamond-87e70f35.woff2" not in html
    # `-italic` shares the logical prefix but is a different face: untouched.
    assert "cormorant-garamond-italic-bbbbbbbb.woff2" in html
    assert "noto-sans-jp-400-aaaaaaaa.woff2" in html


def test_plain_text_mentions_are_not_rewritten(tmp_path):
    # Arrange
    page = _write_sample(tmp_path)

    # Act
    sync_preload_links(["fonts/noto-sans-jp-400-41a4227e.woff2"], page)

    # Assert
    html = page.read_text(encoding="utf-8")
    # Only href-attribute values are synced; prose stays as written.
    assert "noto-sans-jp-400-cccccccc.woff2" in html


def test_second_run_with_the_same_names_is_a_noop(tmp_path):
    # Arrange
    page = _write_sample(tmp_path)
    fonts = [
        "fonts/noto-sans-jp-400-41a4227e.woff2",
        "fonts/cormorant-garamond-188e64d1.woff2",
    ]
    sync_preload_links(fonts, page)
    before = page.read_text(encoding="utf-8")

    # Act
    changed = sync_preload_links(fonts, page)

    # Assert
    assert changed == []
    assert page.read_text(encoding="utf-8") == before


def test_html_without_font_links_is_untouched(tmp_path):
    # Arrange
    page = tmp_path / "index.html"
    page.write_text('<html><link rel="stylesheet" href="/style.css" /></html>', "utf-8")

    # Act
    changed = sync_preload_links(["fonts/noto-sans-jp-400-41a4227e.woff2"], page)

    # Assert
    assert changed == []
    assert "noto-sans-jp-400" not in page.read_text(encoding="utf-8")
