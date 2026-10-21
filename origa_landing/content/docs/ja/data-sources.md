---
title: "Origaのデータソース"
slug: /docs/data-sources
locale: ja
meta_title: "Origaのデータソース — 辞書、漢字、モデル"
meta_description: "Origaを支えるオープンデータとモデル: JMdict辞書、KanjiVGアニメーション、NDLOCR-Lite OCR、Whisper、SudachiDict、いろどり語彙セット、それぞれのライセンス。"
target_keywords: ["jmdict ライセンス", "kanjivg ライセンス", "ndlocr lite", "whisper 日本語", "sudachidict"]
lastmod: 2026-09-23
status: ready
---

<!-- markdownlint-disable-file MD025 — frontmatter `title` is metadata, not a rendered H1; the body has a single H1 by design. -->

# データソース

Origaはオープンデータとモデルの上に構築されています。このページでは、アプリが使用するものと、各派生物の配布条件を一覧します。

## 辞書と読み

辞書エントリー、翻訳、ふりがなは[JMdict / EDRDG](https://www.edrdg.org/jmdict/edrdg_license.html)からCC BY-SA 4.0の下で提供されています。辞書プロジェクトはElectronic Dictionary Research and Development Groupが保守しています。

## 助数詞

助数詞データセット（読み、数字の結びつき、JLPTレベル）は、プロジェクトCDNからアプリアセットと共に配布されるデータファイルとして保守され、オープンな[josuushi](https://github.com/naclsn/josuushi)リファレンスデータと公開されているTofugu・Wikipediaの助数詞リストとクロスチェックされています。訳語は英語・ロシア語・韓国語・ベトナム語で保守されています。

## 漢字アニメーション

筆順データとアニメーションは[KanjiVG](https://kanjivg.tagaini.es/)からCC BY-SA 3.0の下で提供されています。

## 形態素解析

テキストの分割には、Apache-2.0ライセンスの[SudachiDict](https://github.com/WorksApplications/SudachiDict)を使用しています。

## OCR

画像テキスト認識には、国立国会図書館のNDLOCR-LiteをCC BY 4.0の下で使用しています。

## 音声認識

音声の文字起こしには、MITライセンスの[Whisper](https://github.com/openai/whisper)を使用しています。

## フレーズ音声

聴解トレーニングの録音は、NHKの素材を含むネイティブ話者コーパスから来ています。

## 単語セット

インポート用の既製語彙セットは、国際交流基金の[Irodori](https://www.irodori.jpf.go.jp/)から提供されています。

## 例文

頻出単語の教科書例文 — [Tanaka Corpus / WWWJDIC](https://www.edrdg.org/wiki/Tanaka_Corpus.html)と[Tatoeba](https://tatoeba.org)、CC BY 2.0 FR。ロシア語・韓国語・ベトナム語への翻訳は機械生成で、コンテンツパイプラインでレビューされています。

## フォント

- Cormorant Garamond: SIL Open Font License
- IBM Plex Mono: SIL Open Font License
- Noto Sans JP: SIL Open Font License
