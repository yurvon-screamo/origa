#!/usr/bin/env bash
# Convert every generated wav (all shards) to opus 24 kbps mono, then verify
# the output count against the corpus manifest — an incomplete corpus must
# NEVER reach the immutable examples/audio/ CDN prefix (a missing file would
# 404 forever; the pitch/audio incident class, see tts/README.md).
#
# Usage (from the tts work dir holding full_corpus.json and out/batch/):
#   bash /path/to/repo/tts/convert_opus.sh
set -euo pipefail

CORPUS="${1:-full_corpus.json}"
SRC_DIR="${2:-out/batch}"
OUT_DIR="${3:-opus_out}"

mkdir -p "$OUT_DIR"

find "$SRC_DIR" -name '*.wav' -print0 | while IFS= read -r -d '' f; do
    fid=$(basename "$f" .wav)
    out="$OUT_DIR/${fid}.opus"
    if [ ! -f "$out" ]; then
        ffmpeg -nostdin -y -loglevel error -i "$f" \
            -c:a libopus -b:a 24k -ac 1 -application audio "$out"
    fi
done

# Completeness gate: every corpus id must have an opus file.
expected=$(python3 -c "import json,sys; print(len(json.load(open(sys.argv[1]))))" "$CORPUS")
actual=$(find "$OUT_DIR" -name '*.opus' | wc -l)
echo "converted: $actual / expected: $expected"
if [ "$actual" -ne "$expected" ]; then
    echo "ERROR: incomplete corpus — do NOT deploy (immutable CDN keys)" >&2
    exit 1
fi
echo "CONVERT DONE: $actual files, corpus complete"
