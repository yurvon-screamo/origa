#!/usr/bin/env python3
"""Freeze symbolic ONNX dimensions to static values.

ort-web WebGPU execution provider cannot resolve symbolic dimensions
(dim_param) at run time and crashes inside ORT's TensorShape::SizeToDimension
with "Invalid dimension of 4294967295" (-1 reinterpreted as u32). The ort-web
Rust API exposes SessionBuilder::with_dimension_override, but the ort-web
backend raises NotImplemented for it, and JS freeDimensionOverrides is wasm-only.

The only path that works for the WebGPU EP is to bake the static dimension
values directly into the ONNX model — replace dim_param with dim_value.

This is needed for any model where ort-web WebGPU is the active EP and the
graph input has a symbolic batch (or other free) dimension. Models with fully
static input shapes (e.g. our PARSeq recognisers) do not need this.

Usage:
    uv run --with onnx scripts/freeze_onnx_dims.py cdn/ndlocr/deim.onnx
    uv run --with onnx scripts/freeze_onnx_dims.py cdn/ndlocr/deim.onnx --dim N=1
    uv run --with onnx scripts/freeze_onnx_dims.py cdn/ndlocr/deim.onnx --in-place

Default behaviour: replace every symbolic input dimension with 1 (batch=1
covers the inference-time case where we never batch). Use --dim NAME=VALUE to
override specific symbolic dims. Outputs and value_info are left untouched —
ORT computes their shapes at run time and the WebGPU EP handles dynamic
output shapes correctly.
"""

from __future__ import annotations

import argparse
import sys
from pathlib import Path

import onnx
from onnx import ModelProto


def describe_dim(d) -> str:
    if d.dim_param:
        return d.dim_param
    return str(d.dim_value)


def freeze_model(
    model: ModelProto,
    overrides: dict[str, int],
    default_value: int,
) -> list[tuple[str, str, int]]:
    """Replace symbolic dims in graph inputs with static values.

    Returns the list of (input_name, dim_param, new_value) replacements.
    """
    replaced: list[tuple[str, str, int]] = []
    for inp in model.graph.input:
        for d in inp.type.tensor_type.shape.dim:
            if not d.dim_param:
                continue
            new_value = overrides.get(d.dim_param, default_value)
            old_param = d.dim_param
            d.Clear()
            d.dim_value = new_value
            replaced.append((inp.name, old_param, new_value))
    return replaced


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    parser.add_argument("input", type=Path, help="Path to the source .onnx model")
    parser.add_argument(
        "--output",
        type=Path,
        help="Output path (default: <input_stem>_frozen.onnx next to input)",
    )
    parser.add_argument(
        "--in-place",
        action="store_true",
        help="Overwrite the input file (creates a .bak first)",
    )
    parser.add_argument(
        "--dim",
        action="append",
        default=[],
        metavar="NAME=VALUE",
        help="Override a specific symbolic dim, e.g. --dim N=1. Repeatable.",
    )
    parser.add_argument(
        "--default",
        type=int,
        default=1,
        help="Value for symbolic dims not listed in --dim (default: 1)",
    )
    args = parser.parse_args()

    overrides: dict[str, int] = {}
    for spec in args.dim:
        if "=" not in spec:
            print(f"Invalid --dim spec: {spec!r}. Expected NAME=VALUE", file=sys.stderr)
            return 2
        name, value = spec.split("=", 1)
        try:
            overrides[name.strip()] = int(value)
        except ValueError:
            print(f"Invalid --dim value: {value!r}. Expected integer", file=sys.stderr)
            return 2

    if not args.input.exists():
        print(f"Input not found: {args.input}", file=sys.stderr)
        return 1

    model = onnx.load(str(args.input))

    print(f"=== {args.input} ===")
    for inp in model.graph.input:
        dims = [describe_dim(d) for d in inp.type.tensor_type.shape.dim]
        print(f"  input {inp.name!r}: {dims}")

    replaced = freeze_model(model, overrides, args.default)
    if not replaced:
        print("  no symbolic input dims found — model is already static")
        return 0

    print("\nReplacements:")
    for name, old_param, new_value in replaced:
        print(f"  {name!r}.{old_param} -> {new_value}")

    if args.in_place:
        backup = args.input.with_suffix(args.input.suffix + ".bak")
        backup.write_bytes(args.input.read_bytes())
        out_path = args.input
        print(f"\nBackup saved: {backup}")
    else:
        out_path = args.output or args.input.with_name(f"{args.input.stem}_frozen{args.input.suffix}")

    onnx.save(model, str(out_path))
    print(f"Frozen model saved: {out_path}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
