# Origa Translator

[English](README.md) | [Русский](README.ru.md) | [한국어](README.ko.md) | [Tiếng Việt](README.vi.md)

Bộ stack dịch cho pipeline nội dung của Origa: một bộ dịch JA→{EN,RU,KO,VI} và một judge chấm chất lượng.

Đây là code và bộ đo lường. Trọng số mô hình nằm trên Hugging Face; kho dữ liệu huấn luyện là nội bộ, không phát hành.

## Thành phần

**Translator — `ep1`** ([HF: origa-translator-ep1](https://huggingface.co/yurvon-screamo/origa-translator-ep1))
Adapter LoRA (r=16 α=32, 24.16M tham số huấn luyện được, 205 mô-đun trên text stack) đặt trên `google/gemma-4-E2B-it-qat-w4a16-ct`. Huấn luyện 1 epoch trên 600k cặp câu (150k × 4 locale), val_loss 0.6720.

**Judge** ([HF: origa-judge](https://huggingface.co/yurvon-screamo/origa-judge))
`intfloat/multilingual-e5-base` (mean-pooling) + các đầu InfoNCE. Huấn luyện trên 59,973 cặp. Đánh giá nội bộ: 81.9% trên golden_v4c (662 cặp, 7 vòng audit). Judge xếp hạng ứng viên (Best-of-N); điểm tuyệt đối chưa được hiệu chỉnh theo từng locale — xem [issue #713](https://github.com/yurvon-screamo/origa/issues/713).

## Các con số chất lượng

Mọi con số dưới đây là *đánh giá nội bộ*: tác giả dự án tự soát tay, không phải benchmark độc lập. Coi chúng là số định hướng.

| Hạng mục | Kết quả |
| --- | --- |
| Soát tay 200 câu (50 × 4 locale) | 83% dịch trọn vẹn / 99.5% dùng được |
| Định tuyến ngôn ngữ, 200 câu | 200/200 ngay lần đầu |
| Locale zero-shot không có trong huấn luyện (my/bm/th), 24 câu | 24/24 dùng được (mô hình gốc không adapter: 0/24) |
| Judge trên golden_v4c | 542/662 (81.9%) |

Phân tách theo locale và phân loại lỗi: [docs/REPORT_AB_6MODELS.md](docs/REPORT_AB_6MODELS.md). So sánh chrF giữa 6 mô hình ứng cử nằm trong cùng báo cáo; bộ stack cuối được chọn vượt tất cả.

## Quickstart

Chạy adapter bằng vLLM:

```bash
vllm serve google/gemma-4-E2B-it-qat-w4a16-ct \
  --enable-lora --lora-modules tr600k=<path-to-ep1> \
  --max-lora-rank 16
```

Hoặc nạp bằng transformers (trên nền QAT từng chi tiết đều có thể quyết định, xem phần gotcha):

```python
import torch
from transformers import AutoModelForCausalLM, AutoTokenizer, CompressedTensorsConfig
from peft import PeftModel

BASE = "google/gemma-4-E2B-it-qat-w4a16-ct"
model = AutoModelForCausalLM.from_pretrained(
    BASE, dtype=torch.bfloat16, low_cpu_mem_usage=False,
    quantization_config=CompressedTensorsConfig(run_compressed=False),
).to("xpu")
model = PeftModel.from_pretrained(model, "yurvon-screamo/origa-translator-ep1").eval()
tok = AutoTokenizer.from_pretrained(BASE)
```

Không có `low_cpu_mem_usage=False` + `CompressedTensorsConfig(run_compressed=False)`, các layer sẽ nạp thành bộ khung rỗng không mang trọng số đóng gói và PEFT sẽ lỗi.

Inference của judge: serve `intfloat/multilingual-e5-base` qua vLLM pooling với `--pooler-config '{"pooling_type":"MEAN"}'` (pooling mặc định LAST làm vỡ tương thích đầu) và `--attention-backend TRITON_ATTN`. Gọi qua `scripts/judge_api.py` — API câu hỏi có kiểu (noul/choice/score) đặt trên các đầu.

## Scripts

| Script | Việc của nó |
| --- | --- |
| `build_datasets.py` | Lắp các pool SFT từ dữ liệu nguồn |
| `train_sft_qat.py` | Trainer LoRA QAT-native (công thức ở dưới) |
| `run_600k.sh` | Chạy trọn 600k với checkpoint và auto-resume |
| `eval_wrapper.py` | Sinh single-shot + dò ngôn ngữ + retry |
| `eval_model.py` / `bench_train_qat.py` | Đánh giá mô hình và benchmark huấn luyện |
| `judge_api.py` | API câu hỏi có kiểu đặt trên các đầu judge |
| `train_judge_heads.py` | Trainer các đầu judge (ăn embedding từ vLLM) |
| `build_judge_v3.py` | Lắp dataset judge (v3) |

Các script đọc đường dẫn từ biến môi trường (tất cả tùy chọn, mặc định tính theo repo):

| Biến | Mục đích |
| --- | --- |
| `ORIGA_DATA_DIR` | Thư mục gốc dữ liệu huấn luyện |
| `ORIGA_RUNS_DIR`, `ORIGA_WORK_DIR` | Checkpoint và không gian làm việc |
| `ORIGA_JUDGE_HEADS` | Đường dẫn tới `judge_heads_v3.pt` |
| `ORIGA_JUDGE_DATA` | Dataset của judge |

## Công thức huấn luyện (nền QAT)

1. Nạp nền QAT với `CompressedTensorsConfig(run_compressed=False)` (bí danh `dequantize=True`) + `low_cpu_mem_usage=False`. Bước này dequantize int4 đóng gói thành các layer `nn.Linear` bf16 thật, giữ nguyên giá trị trọng số QAT gốc. Đừng chồng bitsandbytes lên trên.
2. Chọn đích LoRA theo cây mô-đun thực tế (`Linear` + `weight` khác None, chỉ text stack — bỏ qua vision/audio/multimodal): 205 mô-đun trên kiến trúc này. `target_modules="all-linear"` văng lỗi trên các layer clippable tùy biến.
3. Huấn luyện QAT-native nhanh hơn 2.4× so với nạp nền qua bitsandbytes-NF4, và đạt chất lượng tương đương trong các lần chạy của chúng tôi.
4. Sinh theo batch với left-padding trên Intel XPU làm méo đầu ra; mọi eval chốt đều chạy single-shot (batch=1). Batching qua vLLM an toàn.

Post-mortem đầy đủ, gồm các bản sửa hạ tầng (lồng cgroup chống OOM, masked CE cho vocab 262k, phân mảnh allocator của XPU), nằm trong [docs/REPORT_AB_6MODELS.md](docs/REPORT_AB_6MODELS.md).

## Hạn chế đã biết

- Judge xếp hạng, không chấm điểm tuyệt đối. Ngưỡng tuyệt đối chưa hiệu chỉnh theo từng locale (bằng chứng cho KO và VI tại [#713](https://github.com/yurvon-screamo/origa/issues/713)). Best-of-N là cách dùng được hỗ trợ.
- Judge cho false-negative trên từ mượn phiên âm; vì vậy scan production luôn cần quy tắc xác nhận kép (phân vị × cờ thuật toán), không cắt theo điểm thô.
- Translator còn sót lỗi 🟡 trên VI (trôi register, trôi lịch sự) và hiếm chỗ đảo cấu trúc trên EN. Chúng tôi đánh giá đây là trần năng lực của nền 2B; thêm dữ liệu không làm mất đi.
- Huấn luyện chỉ chạy trên Intel XPU; inference chạy ở bất cứ đâu vLLM hỗ trợ nền.

## Giấy phép

- Code trong thư mục này: BSL 1.1, cùng giấy phép với cả repo. Xem [LICENSE](../LICENSE) ở gốc.
- Mô hình nền `google/gemma-4-E2B-it-qat-w4a16-ct`: Apache 2.0 (theo model card trên Hugging Face).
- Kho dữ liệu huấn luyện: phái sinh từ nội dung sản phẩm Origa, nội bộ và không phát hành.
