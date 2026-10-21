"""Диагностика утечки: 12 инференсов с замером RSS/GPU/gc после каждого."""
import os, gc, time
os.environ.setdefault("HF_HOME", os.path.expanduser("~/.cache/huggingface"))

import soundfile as sf_lib
import torch, torchaudio

def _sf_load(filepath, *a, **kw):
    data, sr = sf_lib.read(filepath, dtype="float32", always_2d=True)
    return torch.from_numpy(data).T, sr

def _sf_save(filepath, src, sample_rate, *a, **kw):
    arr = src.detach().cpu().numpy()
    if arr.ndim == 2: arr = arr.T
    sf_lib.write(filepath, arr, sample_rate)

torchaudio.load = _sf_load
torchaudio.save = _sf_save

def rss_mb():
    with open("/proc/self/status") as f:
        for line in f:
            if line.startswith("VmRSS"):
                return int(line.split()[1]) // 1024
    return 0

import json
corpus = json.load(open("full_corpus.json"))

from indextts.infer_v2_5 import IndexTTS2
tts = IndexTTS2(cfg_path="index-tts-src/checkpoints/config.yaml",
                model_dir="index-tts-src/checkpoints", use_bf16=True, device="xpu")

print(f"{'iter':>4} {'RSS_MB':>8} {'gpu_alloc':>10} {'gpu_res':>9} {'gc_objs':>8} {'t_sec':>6}", flush=True)
print(f"{'load':>4} {rss_mb():>8} {torch.xpu.memory_allocated()/1e9:>9.2f}G {torch.xpu.memory_reserved()/1e9:>8.2f}G {len(gc.get_objects()):>8}", flush=True)

ref_pool = sorted(__import__("glob").glob("ref_pool/*.wav"))
os.makedirs("/tmp/leak_test", exist_ok=True)

for i in range(12):
    t0 = time.perf_counter()
    item = corpus[i * 100 % len(corpus)]  # guard: small corpora
    tts.infer(spk_audio_prompt=ref_pool[item["id"] % len(ref_pool)],
              text=item["text"], lang="JA",
              output_path=f"/tmp/leak_test/{i:03d}.wav")
    dt = time.perf_counter() - t0
    gc.collect()
    print(f"{i:>4} {rss_mb():>8} {torch.xpu.memory_allocated()/1e9:>9.2f}G {torch.xpu.memory_reserved()/1e9:>8.2f}G {len(gc.get_objects()):>8} {dt:>6.1f}", flush=True)

# Дельта за 12 итераций
print("\nDIAG DONE", flush=True)
