#!/usr/bin/env bash
# Extract the "What's New" section for a release version from CHANGELOG.md.
#
# Used by .github/workflows/_upload-rustore.yml to fill the RuStore `whatsNew`
# field. The workflow runs this BEFORE any RuStore API call, so a contract
# violation fails the job without creating a stub draft (the failure mode that
# got 0.7.28+ rejected by moderation).
#
# Usage: extract-whatsnew.sh <version> <changelog-path> <output-path>
#
# Contract (format documented in the CHANGELOG.md header):
#   - section header is the line "## <version>" (trailing whitespace allowed,
#     no prefix matching — "## 0.7.3" must not match "## 0.7.32")
#   - body: lines until the next "##" section header or EOF
#   - duplicate sections for one version are an authoring error → fail
#   - bullets start with "- " or "* "; links / inline code / nested headers
#     are forbidden by the format contract (nested headers FAIL the job)
#   - limit: 500 characters of the final text including newlines — the
#     RuStore whatsNew field cap
set -euo pipefail

if [[ $# -ne 3 ]]; then
    echo "usage: $0 <version> <changelog-path> <output-path>" >&2
    exit 1
fi

VERSION="$1"
CHANGELOG="$2"
OUTPUT="$3"

if [[ ! -f "$CHANGELOG" ]]; then
    echo "error: changelog not found: $CHANGELOG" >&2
    exit 1
fi

# Count section headers with the same semantics the extractor uses below:
# "## <version>" with trailing whitespace allowed, no prefix matching.
# All ERE metacharacters are escaped so an odd version string cannot widen
# the match ("## 0.7.3" must not match "## 0.7.32").
# Guarded grep: under set -e an unguarded failing grep (0 matches) would
# exit 1 and swallow the actionable message below.
VERSION_RE=$(printf '%s' "$VERSION" | sed -E 's/[][\\.^$*+?(){}|]/\\&/g')
matches=$(grep -cE "^## ${VERSION_RE}[[:space:]]*$" "$CHANGELOG" || true)
if [[ "$matches" -eq 0 ]]; then
    echo "error: no '## ${VERSION}' section in ${CHANGELOG} — add it before tagging (format: see the CHANGELOG.md header)" >&2
    exit 1
fi
if [[ "$matches" -gt 1 ]]; then
    echo "error: found ${matches} '## ${VERSION}' sections in ${CHANGELOG} — keep exactly one" >&2
    exit 1
fi

# Body extraction: exact header match (trailing whitespace trimmed), stop at
# the next "##" section header or EOF (section-last-in-file is valid).
# A "##" header INSIDE the section is a format violation (nested headers are
# forbidden) — fail loudly instead of silently truncating the tail.
# \r is stripped defensively in case the file was edited outside the git
# cycle (.gitattributes enforces LF, but that does not cover stray editors).
header="## ${VERSION}"
body=$(awk -v header="$header" '
    BEGIN { in_section = 0; body_lines = 0 }
    {
        line = $0
        sub(/\r$/, "", line)
        trimmed = line
        sub(/[[:space:]]+$/, "", trimmed)
        if (!in_section) {
            if (trimmed == header) in_section = 1
            next
        }
        if (trimmed ~ /^##/) {
            # A version-formatted header ("## X.Y.Z") is the start of the next
            # section — a clean end of the current one (duplicates are already
            # caught by the grep counter; an empty body is caught below).
            # Any OTHER "##"-line is a nested header, forbidden by the format:
            # after content lines it FAILS loudly instead of silently
            # truncating the tail; before any content it is just the end of
            # an empty section, reported by the dedicated "is empty" check.
            if (trimmed !~ /^## [0-9]+\.[0-9]+\.[0-9]+[[:space:]]*$/ && body_lines > 0) {
                printf "error: nested header [%s] inside the section — nested headers are forbidden (CHANGELOG.md header)\n", trimmed > "/dev/stderr"
                exit 3
            }
            exit 0
        }
        print line
        if (line !~ /^[[:space:]]*$/) body_lines++
    }' "$CHANGELOG") || {
    # awk already printed the diagnostic to stderr (exit 3 = nested header)
    exit 1
}

# Strip bullet markers, trailing whitespace, and drop empty lines.
cleaned=$(printf '%s\n' "$body" \
    | sed -E 's/^[[:space:]]*[-*][[:space:]]+//' \
    | sed -E 's/[[:space:]]+$//' \
    | grep -v '^[[:space:]]*$' || true)

if [[ -z "$cleaned" ]]; then
    echo "error: '## ${VERSION}' section in ${CHANGELOG} is empty — add at least one bullet before tagging" >&2
    exit 1
fi

# Length in Unicode codepoints (jq counts codepoints, not bytes — the naive
# byte count would fail valid Cyrillic text at ~250 chars). This matches the
# RuStore 500-character limit for BMP text; astral-plane characters (rare
# emoji) would be counted 2x by the store — the format contract forbids them.
length=$(printf '%s' "$cleaned" | jq -sR 'length')
if [[ "$length" -gt 500 ]]; then
    echo "error: '## ${VERSION}' section is ${length} characters (limit 500) — shorten it to fit the RuStore whatsNew field" >&2
    exit 1
fi

printf '%s\n' "$cleaned" > "$OUTPUT"
echo "whatsNew for ${VERSION}: ${length} chars extracted from ${CHANGELOG}"
