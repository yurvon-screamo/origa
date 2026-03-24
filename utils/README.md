# Utils CLI

Unified command-line interface for Japanese tokenization and OCR tools.

## Installation

```bash
cargo build --release
```

The binary will be available at `target/release/utils`.

## Usage

The CLI provides four main commands:

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
- `--detector <DETECTOR>` - Detector model path (default: `../ndlocr-lite/src/model/deim-s-1024x1024.onnx`)
- `--rec30 <REC30>` - Parseq 30 model path
- `--rec50 <REC50>` - Parseq 50 model path
- `--rec100 <REC100>` - Parseq
 100 model path
- `--vocab <VOCAB>` - Vocabulary file path (default: `config/NDLmoji.txt`)

### `tokenize-well-known` - Batch Process JSON Files

Batch processes JSON files in well_known_set format, updating the words arrays with tokenized vocabulary.

```bash
# Process a single file
utils tokenize-well-known path/to/file.json

# Process all JSON files in a directory
utils tokenize-well-known path/to/directory/
```

**Arguments:**
- `PATH` - Path to directory or JSON file

### `find-missing` - Find Missing Vocabulary

Finds vocabulary words from well-known sets that are missing from the dictionary and optionally generates translations using an OpenAI-compatible API.

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
- `-o, --output <OUTPUT>` - Output path for the markdown report (default: `missing_vocabulary.md` in project root)
- `-g, --generate` - Auto-generate missing words with translations
- `--api-base <API_BASE>` - OpenAI API base URL (default: `http://10.2.11.6:8001/v1`)
- `--api-key <API_KEY>` - OpenAI API key (default: `none`)
- `-w, --workers <WORKERS>` - Number of concurrent translation requests (default: `32`)
- `--chunk-size <CHUNK_SIZE>` - Chunk size for processing (default: `512`)
- `--russian-only` - Only translate to Russian
- `--english-only` - Only translate to English

## Getting Help

To see all available commands:

```bash
utils --help
```

To see help for a specific command:

```bash
utils tokenize --help
utils ndlocr --help
utils tokenize-well-known --help
utils find-missing --help
```

## Dictionary Requirements

The `tokenize
`, `tokenize-well-known`, and `find-missing` commands require a Japanese dictionary (UniDic) to be available. The CLI will search for the dictionary in the following locations:

1. `origa_ui/public/dictionaries/unidic/` (relative to project root)
2. `CARGO_MANIFEST_DIR/../origa_ui/public/dictionaries/unidic/`

The dictionary files should include:
- `char_def.bin` (compressed)
- `matrix.mtx` (compressed)
- `dict.da` (compressed)
- `dict.vals` (compressed)
- `unk.bin` (compressed)
- `dict.wordsidx` (compressed)
- `dict.words` (compressed)
- `metadata.json`

## Examples

### Extract vocabulary from Japanese text

```bash
tokenizer tokenize "私は日本語を勉強しています"
# Output: 勉強 日本語 私
```

### OCR a Japanese document

```bash
tokenizer ndlocr -i document.png > output.txt
```

### Update well-known sets with tokenized vocabulary

```bash
tokenizer tokenize-well-known ../origa_ui/public/well_known/
```

### Find and translate missing vocabulary

```bash
tokenizer find-missing --generate --api-key sk-xxx
```
