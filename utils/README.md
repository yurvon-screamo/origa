# Utils CLI

Unified command-line interface for Japanese tokenization, OCR, and
content/CDN tooling.

## Installation

```bash
cargo build --release -p utils
```

The binary will be available at `target/release/utils`.

## Usage

The CLI provides the following commands, grouped by purpose:

- **Tokenization & OCR**: `tokenize`, `ndlocr`, `tokenize-well-known`
- **Vocabulary QA**: `find-missing`, `validate-dictionary`, `regenerate-invalid`
- **Grammar**: `generate-grammar`
- **Phrases**: `build-phrase-dataset`, `retokenize-phrases`, `enrich-phrases-with-grammar`
- **Kanji dictionary**: `dedup-kanji-readings`, `patch-kanji-readings`
- **CDN precompute**: `build-cdn-rkyv`, `build-grammar-precompute`, `build-phrase-precompute`

### `tokenize` - Tokenize Japanese Text

Tokenizes Japanese text and extracts vocabulary words.

```bash
# Tokenize a text string
utils tokenize "日本語のテキスト"

# Tokenize from a file
utils tokenize -f path/to/file.txt
utils tokenize path/to/file.txt  # Automatically detects if path exists
```

**Options:**

- `TEXT` - Text to tokenize or path to file
- `-f, --file` - Read text from file

### `ndlocr` - Japanese OCR

Performs OCR on Japanese text images using NDLOCR-Lite models.

```bash
# Basic usage
utils ndlocr -i image.png

# With custom model paths
utils ndlocr \
  -i image.png \
  --detector path/to/detector.onnx \
  --rec30 path/to/rec30.onnx \
  --rec50 path/to/rec50.onnx \
  --rec100 path/to/rec100.onnx \
  --vocab path/to/vocab.txt
```

**Options:**

- `-i, --input <INPUT>` - Input image path (required)
- `--detector <DETECTOR>` - Detector model path
- `--rec30 <REC30>` - Parseq 30 model path
- `--rec50 <REC50>` - Parseq 50 model path
- `--rec100 <REC100>` - Parseq 100 model path
- `--vocab <VOCAB>` - Vocabulary file path

Model paths have built-in defaults pointing at the NDLOCR-Lite checkout
(`../ndlocr-lite/...`); see `utils ndlocr --help` for the exact values.

### `tokenize-well-known` - Batch Process JSON Files

Batch processes JSON files in well_known_set format, updating
the words arrays with tokenized vocabulary.

```bash
# Process a single file
utils tokenize-well-known path/to/file.json

# Process all JSON files in a directory
utils tokenize-well-known path/to/directory/
```

**Arguments:**

- `PATH` - Path to directory or JSON file

### `find-missing` - Find Missing Vocabulary

Finds vocabulary words from well-known sets that are missing from
the dictionary and optionally generates translations using an
OpenAI-compatible API.

```bash
# Generate a report of missing vocabulary
utils find-missing

# Generate a report with custom output path
utils find-missing -o custom_report.md

# Auto-generate translations for missing words
utils find-missing --generate

# Generate only Russian translations
utils find-missing --generate --russian-only

# Generate only English translations
utils find-missing --generate --english-only

# With custom API settings
utils find-missing \
  --generate \
  --api-base http://localhost:8000/v1 \
  --api-key your-api-key \
  --workers 16
```

**Options:**

- `-o, --output <OUTPUT>` - Output path for the markdown report
  (default: `missing_vocabulary.md` in project root)
- `-g, --generate` - Auto-generate missing words with translations
- `--api-base <API_BASE>` - OpenAI API base URL
  (default: `http://localhost:8001/v1`, env: `ORIGA_API_URL`)
- `--api-key <API_KEY>` - OpenAI API key (default: `none`)
- `--model <MODEL>` - Model name for the endpoint (default: `llm`)
- `-w, --workers <WORKERS>` - Number of concurrent translation
  requests (default: `32`)
- `--chunk-size <CHUNK_SIZE>` - Chunk size for processing (default: `512`)
- `--russian-only` - Only translate to Russian
- `--english-only` - Only translate to English

### `validate-dictionary` - Validate Vocabulary Translations

Validates vocabulary dictionary translations by sending each word +
translation to an LLM for Y/N correctness check (OpenRouter by default).

```bash
# Dry run — see how many words would be checked
utils validate-dictionary --api-key YOUR_KEY --dry-run

# Validate all vocabulary with 8 workers
utils validate-dictionary --api-key YOUR_KEY

# Validate with custom model and limit
utils validate-dictionary --api-key YOUR_KEY \
  --model google/gemini-2.0-flash-001 --limit 100

# Custom output path
utils validate-dictionary --api-key YOUR_KEY -o results/invalid.jsonl
```

**Options:**

- `--api-key <API_KEY>` - OpenRouter API key (required, or set
  `OPENROUTER_API_KEY` env var)
- `--api-base <API_BASE>` - API base URL
  (default: `https://openrouter.ai/api/v1`)
- `--model <MODEL>` - LLM model
  (default: `google/gemini-2.0-flash-001`)
- `-w, --workers <WORKERS>` - Concurrent requests (default: `8`)
- `-o, --output <OUTPUT>` - Output JSONL progress file path
- `--dry-run` - Estimate without API calls
- `--limit <LIMIT>` - Max words to validate

**Output:**

- `.jsonl` file: append-only progress (crash-safe)

### `regenerate-invalid` - Regenerate Invalid Translations

Regenerates translations for the words marked invalid by
`validate-dictionary`, reading its `.jsonl` progress file.

```bash
utils regenerate-invalid -i invalid_vocabulary.jsonl --api-key YOUR_KEY

# Only Russian translations
utils regenerate-invalid -i invalid.jsonl --api-key YOUR_KEY --russian-only

# Dry run
utils regenerate-invalid -i invalid.jsonl --api-key YOUR_KEY --dry-run
```

**Options:**

- `-i, --input <INPUT>` - Path to the JSONL progress file from
  `validate-dictionary` (required)
- `--api-base <API_BASE>` - API base URL
  (default: `http://localhost:8001/v1`, env: `ORIGA_API_URL`)
- `--api-key <API_KEY>` - API key (default: `none`)
- `-w, --workers <WORKERS>` - Concurrent requests (default: `8`)
- `--dry-run` - Show what would be done without making changes
- `--russian-only` - Only translate to Russian
- `--english-only` - Only translate to English

### `generate-grammar` - Generate Grammar Rule Descriptions

Generates markdown descriptions for Japanese grammar rules using
an LLM (OpenAI-compatible API).
Russian and English descriptions are generated in **separate API calls**
to ensure native-quality output for each language.

```bash
# Generate description for a single rule
utils generate-grammar 01KJ9AVWBGC2BT0DMFPDYYFEWB

# Dry run: see what rules would be processed
utils generate-grammar --all --dry-run

# Regenerate selected rules by index
utils generate-grammar --indices "153,176,202,193"

# Regenerate all rules
utils generate-grammar --all

# With reasoning and a specific model
utils generate-grammar --all --reasoning --model minimax/minimax-m2.5:free

# Custom grammar.json path
utils generate-grammar --all --grammar-path path/to/grammar.json
```

**Options:**

- `rule_id` - Rule ID to generate (omit with `--all` for batch mode)
- `--all` - Generate descriptions for all rules
- `--indices <INDICES>` - Rule indices to regenerate (comma-separated)
- `--level <LEVEL>` - Filter by JLPT level (N5, N4, N3), use with `--all`
- `--api-base <API_BASE>` - OpenAI API base URL
  (default: `http://localhost:8001/v1`, env: `ORIGA_API_URL`)
- `--api-key <API_KEY>` - API key (default: `none`)
- `--model <MODEL>` - LLM model to use
- `--reasoning` - Enable reasoning with high effort
- `-w, --workers <WORKERS>` - Number of concurrent workers
  (default: `1`, sequential; `0` = sequential)
- `--dry-run` - Show what would be done without making changes
- `--grammar-path <PATH>` - Custom path to grammar.json

**Notes:**

- The command saves after **each rule** (crash-safe)
- 1-second delay between rules (rate limiting)
- RU and EN descriptions are generated in separate LLM calls
- Existing `rule_id`, `level`, and `format_map` are preserved

### `build-phrase-dataset` - Build Phrase Dataset

Builds the phrase dataset from transcription JSON, filtering by
minimum vocabulary token count.

```bash
utils build-phrase-dataset -i transcriptions.json -o out_dir/
utils build-phrase-dataset -i transcriptions.json -o out_dir/ --min-tokens 3
```

**Options:**

- `-i, --input <INPUT>` - Input JSON file with transcriptions (required)
- `-o, --output <OUTPUT>` - Output directory for results (required)
- `--min-tokens <MIN_TOKENS>` - Minimum vocabulary tokens per phrase
  (default: `2`)

### `retokenize-phrases` - Re-tokenize Phrase Index

Re-tokenizes `phrase_index.json` tokens with the current tokenizer,
using phrase texts from the data bundles.

```bash
utils retokenize-phrases \
  --index cdn/phrases/phrase_index.json \
  --data-dir cdn/phrases/data/
```

**Options:**

- `--index <INDEX>` - Path to `phrase_index.json` (required)
- `--data-dir <DATA_DIR>` - Directory containing `data_bundle_*.json`
  phrase texts (required)

### `enrich-phrases-with-grammar` - Enrich Phrase Index with Grammar

Detects grammar rules in phrases and writes an enriched phrase index.

```bash
utils enrich-phrases-with-grammar \
  -i cdn/phrases/phrase_index.json \
  --chunks-dir cdn/phrases/data/ \
  -g cdn/grammar/grammar_v2.json \
  -o cdn/phrases/phrase_index_enriched.json
```

**Options:**

- `-i, --input <INPUT>` - Input `phrase_index.json` path (required)
- `--chunks-dir <CHUNKS_DIR>` - Directory containing chunk data files
  (`p*.json`, required)
- `-g, --grammar <GRAMMAR>` - Path to grammar.json (required)
- `-o, --output <OUTPUT>` - Output enriched `phrase_index.json` path
  (required)
- `--dictionary-dir <DIR>` - Dictionary directory for the tokenizer
  (optional, uses default path)

### `dedup-kanji-readings` - Remove Duplicate Kanji Readings

Removes duplicate readings from the kanji dictionary.

```bash
utils dedup-kanji-readings                    # default path
utils dedup-kanji-readings -i kanji.json --dry-run
```

**Options:**

- `-i, --input <INPUT>` - Path to `kanji.json`
  (default: `cdn/dictionary/kanji.json`)
- `--dry-run` - Show what would be changed without writing

### `patch-kanji-readings` - Apply Reading Patches

Applies targeted reading-removal patches to the kanji dictionary.

```bash
utils patch-kanji-readings                    # default paths
utils patch-kanji-readings --dry-run
```

**Options:**

- `-i, --input <INPUT>` - Path to `kanji.json`
  (default: `cdn/dictionary/kanji.json`)
- `--patches <PATCHES>` - Path to patches JSON file
  (default: `cdn/dictionary/kanji_patches.json`)
- `--dry-run` - Show what would be changed without writing

### `build-cdn-rkyv` - Build CDN rkyv Blobs

Builds the pre-parsed rkyv blobs (`JmdictFurigana.rkyv`,
`vocabulary.rkyv`) for CDN deploy. Regenerates blobs whose sources
changed (skips when the existing blob header is fresh). Blobs are
deterministic: identical sources produce identical bytes, keeping the
CDN manifest hash stable. Invoked automatically by
`scripts/deploy_cdn.py` (Step 0.5).

```bash
cargo run -p utils -- build-cdn-rkyv
```

**Options:**

- `--cdn-dir <DIR>` - CDN root directory (defaults to `<repo>/cdn`
  next to the crate)

### `build-grammar-precompute` - Build Grammar Tokenization Precompute

Builds the grammar tokenization precompute (#521): one deflated rkyv
blob with furigana spans for every grammar markdown text node and whole
string. Freshness binds to both grammar sources plus the tokenizer
inputs.

```bash
cargo run -p utils -- build-grammar-precompute
```

**Options:**

- `--cdn-dir <DIR>` - CDN root directory (defaults to `<repo>/cdn`
  next to the crate)

**Requires:** the tokenizer dictionary (lindera) and JmdictFurigana in
the CDN checkout.

### `build-phrase-precompute` - Build Phrase Tokenization Precompute

Builds the phrase tokenization precompute (#521): per-chunk deflated
rkyv blobs keyed by the full phrase text and each of its render
sentences. Freshness binds the chunk bytes plus the dictionary inputs,
so a dictionary bump regenerates all chunks.

```bash
cargo run -p utils -- build-phrase-precompute
```

**Options:**

- `--cdn-dir <DIR>` - CDN root directory (defaults to `<repo>/cdn`
  next to the crate)

**Requires:** the tokenizer dictionary (lindera) and JmdictFurigana in
the CDN checkout.

## Getting Help

To see all available commands:

```bash
utils --help
```

To see help for a specific command:

```bash
utils tokenize --help
utils ndlocr --help
utils find-missing --help
```

## Dictionary Requirements

The tokenizer commands require the SudachiDict dictionary (lindera dictionary
format, built by `scripts/build_sudachidict.py`; readable by the lindera 6.x
runtime). The CLI resolves it in the
following locations:

1. the directory passed via `--dictionary-dir` (if it exists)
2. `cdn/dictionaries/sudachidict-20260723/` (versioned)
3. `target/cdn-cache/` (downloaded CDN copy)

The dictionary files include:

- `char_def.bin` (compressed)
- `matrix.mtx` (compressed)
- `dict.trie` (compressed)
- `dict.valsidx` (compressed)
- `dict.vals` (compressed)
- `unk.bin` (compressed)
- `dict.wordsidx` (compressed)
- `dict.words` (compressed)
- `metadata.json`

## Examples

### Extract vocabulary from Japanese text

```bash
utils tokenize "私は日本語を勉強しています"
# Output: 勉強 日本語 私
```

### OCR a Japanese document

```bash
utils ndlocr -i document.png > output.txt
```

### Update well-known sets with tokenized vocabulary

```bash
utils tokenize-well-known ../origa_ui/public/well_known/
```

### Find and translate missing vocabulary

```bash
utils find-missing --generate --api-key sk-xxx
```
