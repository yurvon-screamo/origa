#!/usr/bin/env python3
"""Post-install patches for the vLLM XPU stack (idempotent).

Run once after `uv sync` (or any vllm/vllm-xpu-kernels reinstall):
    uv run scripts/patch_vllm_xpu.py

What and why — see docs/RUNBOOK.md, section 'Окружение':
1. dist_backend xccl -> gloo: oneccl 2022 (the only pip runtime) is incompatible
   with torch>=2.14-xpu XCCL; single-GPU serving works on gloo.
"""
import re
import sys
from pathlib import Path

import vllm  # noqa: F401 — locate site-packages
VLLM_DIR = Path(vllm.__file__).parent
patched = []

xpu_plat = VLLM_DIR / "platforms" / "xpu.py"
s = xpu_plat.read_text(encoding="utf-8")
s2 = s.replace('    dist_backend: str = "xccl"  # xccl only',
               '    dist_backend: str = "gloo"  # patched single-gpu (oneccl2022 vs torch2.16-xcu broken)')
if s2 != s:
    xpu_plat.write_text(s2, encoding="utf-8")
    patched.append("platforms/xpu.py: xccl -> gloo")

init = VLLM_DIR / "platforms" / "__init__.py"
s = init.read_text(encoding="utf-8")
s2 = s.replace('            dist_backend = "xccl"',
               '            dist_backend = "gloo"  # patched: builtin xccl broken on this stack')
if s2 != s:
    init.write_text(s2, encoding="utf-8")
    patched.append("platforms/__init__.py: xccl -> gloo")

if patched:
    print("patched:", "; ".join(patched))
else:
    print("already patched — nothing to do")
