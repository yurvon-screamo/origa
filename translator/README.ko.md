# Origa Translator

[English](README.md) | [Русский](README.ru.md) | [한국어](README.ko.md) | [Tiếng Việt](README.vi.md)

Origa 콘텐츠 파이프라인용 번역 스택: JA→{EN,RU,KO,VI} 번역기와 품질 판별기(judge).

여기는 코드와 평가 하네스입니다. 모델 가중치는 Hugging Face에 있고, 학습 코퍼스는 내부 자산으로 공개하지 않습니다.

## 구성 요소

**번역기 — `ep1`** ([HF: origa-translator-ep1](https://huggingface.co/yurvon-screamo/origa-translator-ep1))
`google/gemma-4-E2B-it-qat-w4a16-ct` 위의 LoRA 어댑터(r=16 α=32, 학습 파라미터 24.16M, 텍스트 스택 205개 모듈). 600k 문장 쌍(150k × 4 로캘)으로 1 에포크 학습, val_loss 0.6720.

**판별기** ([HF: origa-judge](https://huggingface.co/yurvon-screamo/origa-judge))
`intfloat/multilingual-e5-base`(mean-pooling) + InfoNCE 헤드. 60,004쌍으로 학습(v3 + 라벨링된 프로덕션 오류 32건). 내부 평가: golden_v4c에서 82.8%(662쌍, 7차 감사). 판별기는 후보를 랭킹합니다(Best-of-N). 로캘별 절대 임계값은 캘리브레이션 레이어로 제공됩니다: `judge_api.Judge.verdict()`는 라벨링된 쌍으로 피팅한 임계값으로 합격/불합격 판정을 냅니다(`calibrate_judge.py`). 임계값은 감사 기준으로 캘리브레이션되었습니다(golden_v4c 저자 라벨); LLM 라벨링 프로덕션-플로 웨이브(로캘당 200쌍, #713)가 잔여 맹점을 정량화했습니다: 미세한 오류가 good 분포 안에 스코어링됩니다(로캘별 `wave_v1` 블록), 따라서 스캔은 이중 확인 규칙을 유지합니다 — [#713](https://github.com/yurvon-screamo/origa/issues/713).

## 품질 수치

아래 수치는 모두 *내부 평가(internal eval)*입니다: 프로젝트 작성자의 수동 검토이지 독립 벤치마크가 아닙니다. 방향성 참고로만 취급하세요.

| 점검 | 결과 |
| --- | --- |
| 수동 검토, 200문장 (50 × 4 로캘) | 83% 완전 / 99.5% 사용 가능 |
| 언어 라우팅, 200문장 | 200/200 첫 시도 |
| 학습에 없던 제로샷 로캘 (my/bm/th), 24문장 | 24/24 사용 가능 (어댑터 없는 베이스: 0/24) |
| golden_v4c에서의 judge | 542/662 (81.9%) |

로캘별 세부와 오류 유형 분류: [docs/REPORT_AB_6MODELS.md](docs/REPORT_AB_6MODELS.md). 6개 후보 모델의 chrF 비교도 같은 보고서에 있습니다; 최종 스택은 이들 모두를 제치고 선택되었습니다.

## 퀵스타트

vLLM으로 어댑터 서빙:

```bash
vllm serve google/gemma-4-E2B-it-qat-w4a16-ct \
  --enable-lora --lora-modules tr600k=<path-to-ep1> \
  --max-lora-rank 16
```

또는 transformers로 로드 (QAT 베이스에서는 세부 사항이 중요합니다, gotchas 참조):

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

`low_cpu_mem_usage=False` + `CompressedTensorsConfig(run_compressed=False)` 없이는 레이어가 packed weight 없는 스켈레톤으로 로드되며 PEFT가 실패합니다.

judge 추론: `intfloat/multilingual-e5-base`를 vLLM pooling으로 서빙하되 `--pooler-config '{"pooling_type":"MEAN"}'`(기본값인 LAST pooling은 헤드 호환성을 깨뜨립니다)와 `--attention-backend TRITON_ATTN`을 사용합니다. 호출은 헤드 위의 타입 질의 API인 `scripts/judge_api.py`(noul/choice/score)로 합니다; 임계값이 피팅되면 `verdict()`가 캘리브레이션된 로캘별 합격/불합격 판정을 추가합니다.

## 스크립트

| 스크립트 | 하는 일 |
| --- | --- |
| `build_datasets.py` | 소스 데이터에서 SFT 풀 조립 |
| `train_sft_qat.py` | QAT 네이티브 LoRA 트레이너 (아래 레시피 참조) |
| `run_600k.sh` | 체크포인팅과 auto-resume이 있는 전체 600k 실행 |
| `eval_wrapper.py` | 단일 샷 생성 + 언어 감지 + 재시도 |
| `eval_model.py` / `bench_train_qat.py` | 모델 평가와 학습 벤치마크 |
| `judge_api.py` | judge 헤드 위의 타입 질의 API + 캘리브레이션된 `verdict()` |
| `calibrate_judge.py` | 라벨링 태스크 export/import + 로캘별 임계값 피팅 (issue #713) |
| `train_judge_heads.py` | judge 헤드 트레이너 (vLLM 임베딩 소비) |
| `build_judge_v3.py` | judge 데이터셋 조립 (v3) |

스크립트는 위치를 환경 변수에서 읽습니다 (모두 선택 사항, 기본값은 레포 상대 경로):

| 변수 | 용도 |
| --- | --- |
| `ORIGA_DATA_DIR` | 학습 데이터 루트 |
| `ORIGA_RUNS_DIR`, `ORIGA_WORK_DIR` | 체크포인트와 작업 공간 |
| `ORIGA_JUDGE_HEADS` | `origa-judge.pt` 경로 |
| `ORIGA_JUDGE_CALIBRATION` | `judge_calibration.json` 경로 (피팅된 로캘별 임계값) |
| `ORIGA_JUDGE_DATA` | judge 데이터셋 |

## 학습 레시피 (QAT 베이스)

1. QAT 베이스를 `CompressedTensorsConfig(run_compressed=False)` (별칭 `dequantize=True`) + `low_cpu_mem_usage=False`로 로드합니다. 이렇게 하면 packed int4가 원래 QAT 가중치 값을 가진 실제 bf16 `nn.Linear` 레이어로 양자화 해제됩니다. 그 위에 bitsandbytes를 쌓지 마세요.
2. 실제 모듈 트리에서 LoRA 타깃을 수집합니다 (`Linear` + `weight`가 non-None, 텍스트 스택만 — vision/audio/멀티모달 제외): 이 아키텍처에서 205개 모듈. `target_modules="all-linear"`은 커스텀 clippable 레이어에서 크래시합니다.
3. QAT 네이티브 학습은 베이스를 bitsandbytes-NF4로 로드하는 것보다 2.4배 빠르고, 우리 실행에서는 동일한 품질에 도달했습니다.
4. Intel XPU에서 left-padding을 사용한 배치 생성은 출력을 왜곡합니다; 모든 최종 평가는 single-shot(batch=1)으로 실행했습니다. vLLM 배칭은 안전합니다.

OOM cgroup 케이지, 262k 어휘용 masked CE, XPU 얼로케이터 파편화를 포함한 전체 포스트모템은 [docs/REPORT_AB_6MODELS.md](docs/REPORT_AB_6MODELS.md)에 있습니다.

## 알려진 제한

- 합격/불합격 절대 판정(`judge_api` `verdict()`)은 커밋된 로캘별 임계값(`calibration/judge_calibration.json`)을 사용합니다. 피팅은 golden_v4c 저자 감사 라벨 기준입니다(로캘별 CV F1 0.68–0.76), 즉 '감사급 오류 없음'. 프로덕션-플로 웨이브(로캘당 200쌍, LLM 라벨, [#713](https://github.com/yurvon-screamo/origa/issues/713))의 결과: 미세한 오류(KO 이름 음차, 레지스터 시프트, 관용구 반전)는 good 분포 안에 스코어링됩니다 — 절대 판정이 이를 잡지 못하며, 웨이브의 퇴화 임계값은 커밋하지 않았습니다. 스캔은 이중 확인을 유지하고 레이어는 피팅되지 않은 로캘에서 크게 실패합니다.
- judge는 음차 외래어에서 거짓 음성(false negative)을 냅니다. 캘리브레이션된 임계값도 이 클래스를 없애지 못합니다: 프로덕션 스캔은 이중 확인 규칙(백분위 × 알고리즘 플래그)을 유지하며, 날 점수 컷오프는 쓰지 않습니다. `verdict()`는 새 번역에 대한 파이프라인 자동 합격 판정용입니다.
- 번역기는 VI에서 🟡 잔여 문제(레지스터/존칭 시프트)와 드문 EN 역전을 유지합니다. 우리는 이를 2B 베이스의 capability 한계로 평가했으며, 데이터를 더해도 사라지지 않았습니다.
- 학습은 Intel XPU 전용입니다; 추론은 베이스가 지원하는 환경이면 어디서든 vLLM으로 실행됩니다.

## 라이선스

- 이 디렉터리의 코드: BSL 1.1, 레포 전체와 동일합니다. 루트 [LICENSE](../LICENSE) 참조.
- 베이스 모델 `google/gemma-4-E2B-it-qat-w4a16-ct`: Apache 2.0 (HF 모델 카드 기준).
- 학습 코퍼스: Origa 제품 콘텐츠에서 파생되었으며 내부 자산으로 공개하지 않습니다.
