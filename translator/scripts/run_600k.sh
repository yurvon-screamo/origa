#!/usr/bin/env bash
# 600k full run with auto-resume: restarts from the last checkpoint after any crash.
# bs6 + accum8 (eff.batch 48) + towers-cpu + pretokenize + sdpa.
# Checkpoints every 50k rows (station ladder: 100k/200k/.../600k).
set -u
cd "${WORK_DIR:-.}" || exit 1
PY="${PY:-python}"
CAP="systemd-run --user --scope -q -p MemoryMax=26G -p MemorySwapMax=0"
OUT=qat-scale/600k
PROG=runs/$OUT/progress.json
TOTAL_ROWS=600000

while true; do
  RESUME_ARGS=()
  if [ -f "$PROG" ]; then
    STEP=$("$PY" -c "import json;print(json.load(open('$PROG'))['step'])" 2>/dev/null || echo 0)
    ROWS=$("$PY" -c "import json;print(json.load(open('$PROG'))['rows'])" 2>/dev/null || echo 0)
    ADAPTER=$("$PY" -c "import json;print(json.load(open('$PROG'))['adapter'])" 2>/dev/null || echo "")
    if [ "$ROWS" -ge "$TOTAL_ROWS" ]; then
      echo "[600k] already complete (rows=$ROWS)" >> logs/ab-runner.out
      break
    fi
    if [ -n "$ADAPTER" ] && [ "$ROWS" -gt 0 ]; then
      RESUME_ARGS+=(--resume "$ADAPTER" --skip-rows "$ROWS" --start-step "$STEP")
      echo "[600k] RESUME from rows=$ROWS step=$STEP" >> logs/ab-runner.out
    fi
  else
    echo "[600k] fresh start $(date)" >> logs/ab-runner.out
  fi

  $CAP "$PY" train_sft_qat.py \
    --data data/sft_multi_c150k.jsonl \
    --out $OUT \
    --bs 4 --accum 8 \
    --save-every-rows 50000 \
    --pretokenize --towers-cpu \
    ${RESUME_ARGS[@]+"${RESUME_ARGS[@]}"} >> logs/qat-600k.out 2>&1
  RC=$?
  echo "[600k] train rc=$RC $(date)" >> logs/ab-runner.out
  # rc=0 -> finished the epoch; verify progress says complete
  if [ "$RC" -eq 0 ]; then
    ROWS=$("$PY" -c "import json;print(json.load(open('$PROG'))['rows'])" 2>/dev/null || echo 0)
    if [ "$ROWS" -ge "$TOTAL_ROWS" ]; then break; fi
    # rc=0 but not complete (edge: crashed between save and exit) -> loop and resume
  fi
  sleep 30
done

# final eval: wrapper 20x4 with retry logic
$CAP "$PY" eval_wrapper.py \
  --base "${BASE_MODEL:-google/gemma-4-E2B-it-qat-w4a16-ct}" \
  --adapter runs/$OUT/ep1 --name wrap-qat-600k > logs/wrap-qat-600k.out 2>&1
echo "[600k] eval rc=$? $(date)" >> logs/ab-runner.out
touch logs/qat-600k-all-done
