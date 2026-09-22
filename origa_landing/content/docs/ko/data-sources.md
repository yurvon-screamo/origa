---
title: "Origa 데이터 출처"
slug: /docs/data-sources
locale: ko
meta_title: "Origa 데이터 출처 — 사전, 한자, 모델"
meta_description: "Origa를 지탱하는 공개 데이터와 모델: JMdict 사전, KanjiVG 애니메이션, NDLOCR-Lite OCR, Whisper, SudachiDict, Irodori 단어 세트와 각 라이선스."
target_keywords: ["jmdict 라이선스", "kanjivg 라이선스", "ndlocr lite", "whisper mit 일본어", "sudachidict apache"]
lastmod: 2026-09-23
status: ready
---

<!-- markdownlint-disable-file MD025 — frontmatter `title` is metadata, not a rendered H1; the body has a single H1 by design. -->

# 데이터 출처

Origa는 공개 데이터와 모델 위에 세워져 있습니다. 이 문서는 앱이 무엇을 쓰는지, 파생물이 어떤 조건으로 배포되는지를 적습니다.

## 사전과 읽기

사전 항목, 번역, 후리가나는 CC BY-SA 4.0 아래 [JMdict / EDRDG](https://www.edrdg.org/jmdict/edrdg_license.html)에서 옵니다. 사전 프로젝트는 Electronic Dictionary Research and Development Group이 관리합니다.

## 조수사

조수사 데이터셋(읽기와 숫자 결합형)은 저장소에서 직접 관리하는 원본 파일이며, 공개 참조 데이터 [josuushi](https://github.com/naclsn/josuushi)와 Tofugu·위키피디아 조수사 목록과 대조 검증합니다. 뜻풀이는 영어, 러시아어, 한국어, 베트남어로 유지됩니다.

## 한자 애니메이션

획순 데이터와 애니메이션은 CC BY-SA 3.0 아래 [KanjiVG](https://kanjivg.tagaini.es/)에서 옵니다.

## 토큰화

텍스트 분절은 Apache-2.0 라이선스의 [SudachiDict](https://github.com/WorksApplications/SudachiDict)를 씁니다.

## OCR

이미지 문자 인식은 일본 국립국회도서관의 NDLOCR-Lite를 CC BY 4.0 아래 씁니다.

## 음성 인식

오디오 전사는 MIT 라이선스의 [Whisper](https://github.com/openai/whisper)를 씁니다.

## 문장 오디오

듣기 연습 녹음은 NHK 자료를 포함해 원어민 코퍼스에서 옵니다.

## 단어 세트

가져오기용 미리 구성된 단어 세트는 국제교류기금의 [Irodori](https://www.irodori.jpf.go.jp/)에서 옵니다.

## 글꼴

- Cormorant Garamond: SIL Open Font License
- IBM Plex Mono: SIL Open Font License
- Noto Sans JP: SIL Open Font License
