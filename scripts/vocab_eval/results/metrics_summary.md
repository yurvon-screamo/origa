# Vocab eval metrics summary

| model | lang | json | purity | copy | cross | sem | sec/word | tok/s | barriers |
|---|---|---|---|---|---|---|---|---|---|
| gemma4:e4b-it-q8_0 | vi | 1.00 | 0.99 | 0.01 | 0.00 | 0.631 | 5.5 | 18.3 | PASS |
| gemma4:e4b-it-q8_0 | ko | 1.00 | 1.00 | 0.01 | 0.00 | 0.603 | 5.5 | 18.3 | PASS |
| gemma4:e2b-it-q8_0 | vi | 1.00 | 0.99 | 0.05 | 0.00 | 0.628 | 3.5 | 31.4 | PASS |
| gemma4:e2b-it-q8_0 | ko | 1.00 | 0.99 | 0.01 | 0.00 | 0.610 | 3.5 | 31.4 | PASS |
| SparkLLM/Spark-X2.5-4B | vi | 0.98 | 0.98 | 0.03 | 0.00 | 0.629 | 10.1 | 9.4 | PASS |
| SparkLLM/Spark-X2.5-4B | ko | 0.98 | 0.99 | 0.01 | 0.02 | 0.610 | 10.1 | 9.4 | PASS |
| hf.co/InternScience/Agents-A1-4B-Q8_0-GGUF:Q8_0 | vi | 1.00 | 1.00 | 0.03 | 0.00 | 0.626 | 4.8 | 22.2 | PASS |
| hf.co/InternScience/Agents-A1-4B-Q8_0-GGUF:Q8_0 | ko | 1.00 | 1.00 | 0.03 | 0.00 | 0.610 | 4.8 | 22.2 | PASS |
| hf.co/bartowski/Nanbeige_Nanbeige4.2-3B-GGUF:Q8_0 | vi | 0.96 | 0.99 | 0.02 | 0.00 | 0.586 | 12.5 | 12.6 | PASS |
| hf.co/bartowski/Nanbeige_Nanbeige4.2-3B-GGUF:Q8_0 | ko | 0.96 | 0.99 | 0.01 | 0.01 | 0.587 | 12.5 | 12.6 | PASS |
- determinism gemma4:e4b-it-q8_0: 1.000
- structure gemma4:e4b-it-q8_0 vi: senses_ok=0.98 empty_fields=0.00
- structure gemma4:e4b-it-q8_0 ko: senses_ok=0.98 empty_fields=0.00
- determinism gemma4:e2b-it-q8_0: 1.000
- structure gemma4:e2b-it-q8_0 vi: senses_ok=0.96 empty_fields=0.00
- structure gemma4:e2b-it-q8_0 ko: senses_ok=0.94 empty_fields=0.02
- determinism SparkLLM/Spark-X2.5-4B: 0.950
- structure SparkLLM/Spark-X2.5-4B vi: senses_ok=0.94 empty_fields=0.00
- structure SparkLLM/Spark-X2.5-4B ko: senses_ok=0.94 empty_fields=0.00
- determinism hf.co/InternScience/Agents-A1-4B-Q8_0-GGUF:Q8_0: 1.000
- structure hf.co/InternScience/Agents-A1-4B-Q8_0-GGUF:Q8_0 vi: senses_ok=0.97 empty_fields=0.00
- structure hf.co/InternScience/Agents-A1-4B-Q8_0-GGUF:Q8_0 ko: senses_ok=0.96 empty_fields=0.01
- determinism hf.co/bartowski/Nanbeige_Nanbeige4.2-3B-GGUF:Q8_0: 1.000
- structure hf.co/bartowski/Nanbeige_Nanbeige4.2-3B-GGUF:Q8_0 vi: senses_ok=0.94 empty_fields=0.00
- structure hf.co/bartowski/Nanbeige_Nanbeige4.2-3B-GGUF:Q8_0 ko: senses_ok=0.93 empty_fields=0.00
