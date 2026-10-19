#!/bin/bash
# Конвертация wav → opus 24kbps mono (параллельно на все ядра)
set -e
mkdir -p opus_out
export LANG=C
find out/batch/shard_0 out/batch/shard_1 -name '*.wav' | while read f; do
    fid=$(basename "$f" .wav)
    out="opus_out/${fid}.opus"
    [ -f "$out" ] && continue
    echo "$f|$out"
done | xargs -P 10 -I {} bash -c '
    IFS="|" read -r src dst <<< "{}"
    ffmpeg -y -loglevel error -i "$src" -c:a libopus -b:a 24k -ar 24000 -ac 1 -application audio "$dst"
'
echo "CONVERT DONE: $(ls opus_out | wc -l) files"
