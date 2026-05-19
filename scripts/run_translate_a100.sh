#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

# --- Defaults (override via env vars) ---
INPUT_DIR="${TRANSLATE_INPUT_DIR:-./phrases/data}"
OUTPUT_DIR="${TRANSLATE_OUTPUT_DIR:-./phrases/data}"
BATCH_SIZE="${TRANSLATE_BATCH_SIZE:-64}"
CHECKPOINT="${TRANSLATE_CHECKPOINT:-translate_checkpoint.json}"
LANGUAGES="${TRANSLATE_LANGUAGES:-en,ru}"
SCRIPT="${TRANSLATE_SCRIPT:-${SCRIPT_DIR}/translate_madlad400.py}"

# --- Parse CLI args (override env vars) ---
while [[ $# -gt 0 ]]; do
    case "$1" in
        --input)       INPUT_DIR="$2";    shift 2 ;;
        --output)      OUTPUT_DIR="$2";   shift 2 ;;
        --batch-size)  BATCH_SIZE="$2";   shift 2 ;;
        --checkpoint)  CHECKPOINT="$2";   shift 2 ;;
        --languages)   LANGUAGES="$2";    shift 2 ;;
        --script)      SCRIPT="$2";       shift 2 ;;
        -h|--help)
            echo "Usage: $0 [OPTIONS]"
            echo ""
            echo "Options:"
            echo "  --input DIR       Input directory   (default: ./phrases/data)"
            echo "  --output DIR      Output directory   (default: ./phrases/data)"
            echo "  --batch-size N    Batch size          (default: 64)"
            echo "  --checkpoint FILE Checkpoint file     (default: translate_checkpoint.json)"
            echo "  --languages LANGS Comma-separated     (default: en,ru)"
            echo "  --script FILE     Python script       (default: translate_madlad400.py)"
            echo ""
            echo "Env vars: TRANSLATE_INPUT_DIR, TRANSLATE_OUTPUT_DIR,"
            echo "          TRANSLATE_BATCH_SIZE, TRANSLATE_CHECKPOINT,"
            echo "          TRANSLATE_LANGUAGES, TRANSLATE_SCRIPT"
            exit 0
            ;;
        *)
            echo "Unknown option: $1" >&2
            exit 1
            ;;
    esac
done

echo "=== A100 Translation Runner ==="
echo ""

# --- 1. GPU check ---
echo "--- GPU ---"
if ! command -v nvidia-smi &>/dev/null; then
    echo "ERROR: nvidia-smi not found. No GPU drivers?" >&2
    exit 1
fi

if ! nvidia-smi &>/dev/null; then
    echo "ERROR: nvidia-smi failed. GPU not available." >&2
    exit 1
fi

nvidia-smi --query-gpu=name,memory.total,memory.free --format=csv,noheader
echo ""

# --- 2. Python + packages check ---
echo "--- Python ---"
if ! command -v python3 &>/dev/null; then
    echo "ERROR: python3 not found." >&2
    exit 1
fi

python3 --version

MISSING=""
for pkg in torch transformers tqdm; do
    if ! python3 -c "import ${pkg}" 2>/dev/null; then
        MISSING="${MISSING} ${pkg}"
    fi
done

if [[ -n "${MISSING}" ]]; then
    echo "ERROR: Missing packages:${MISSING}" >&2
    echo "Install with:" >&2
    echo "  pip install${MISSING}" >&2
    exit 1
fi

echo "All required packages found."
echo ""

# --- 3. Input directory check ---
echo "--- Data ---"
if [[ ! -d "${INPUT_DIR}" ]]; then
    echo "ERROR: Input directory not found: ${INPUT_DIR}" >&2
    exit 1
fi

FILE_COUNT=$(find "${INPUT_DIR}" -maxdepth 1 -name 'p*.json' -type f | wc -l)
echo "Input:  ${INPUT_DIR} (${FILE_COUNT} files)"
echo "Output: ${OUTPUT_DIR}"
echo ""

# --- 4. Run ---
echo "--- Running ${SCRIPT} ---"
echo "  --input ${INPUT_DIR}"
echo "  --output ${OUTPUT_DIR}"
echo "  --batch-size ${BATCH_SIZE}"
echo "  --checkpoint ${CHECKPOINT}"
echo "  --languages ${LANGUAGES}"
echo ""

python3 "${SCRIPT}" \
    --input "${INPUT_DIR}" \
    --output "${OUTPUT_DIR}" \
    --batch-size "${BATCH_SIZE}" \
    --checkpoint "${CHECKPOINT}" \
    --languages "${LANGUAGES}"

echo ""
echo "=== Done ==="
