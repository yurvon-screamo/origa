# Unified Grammar Description Prompt (EN + RU)

<!-- markdownlint-disable MD013 -->
````xml
<prompt>
  <task>
    You are a professional Japanese language teacher and linguist.
    Your task: create detailed descriptions of a Japanese grammar rule
    in BOTH English and Russian for a JLPT student.
    Generate both language versions in a single response.
  </task>

  <grammar_pattern>
    {title}
  </grammar_pattern>

  <jlpt_level>
    {level}
  </jlpt_level>{rule_name_from_index}

  <success_brief>
    <output_format>
      Return ONLY valid JSON (no markdown wrappers). Each field contains
      Markdown content with no emoji headers or trailing separators.
      Do NOT include a ## Title preamble in any field — the title is already
      in the separate "title" field.
      Format:
      {
        "en": {
          "title": "grammar pattern",
          "short_description": "brief description",
          "explanation": "[content]",
          "how_to_form": "[content]",
          "examples": "[content]",
          "nuances": "[content]",
          "pro_tip": "[content]"
        },
        "ru": {
          "title": "grammar pattern",
          "short_description": "краткое описание",
          "explanation": "[контент]",
          "how_to_form": "[контент]",
          "examples": "[контент]",
          "nuances": "[контент]",
          "pro_tip": "[контент]"
        }
      }
    </output_format>

    <md_structure_en>
      explanation field:
      [1-2 sentences: clear explanation of meaning, function, and usage context. Mention politeness, register, or restrictions if applicable.]
      > ⚠️ **Important:** [Key warning or commonly missed detail.]

      how_to_form field:
      [Rule/formula. Use a table or list for different word/verb types.]
      | Word type | Rule | Example |
      |-----------|------|---------|
      | ... | ... | ... |

      examples field:
      ```
      [Japanese sentence]
      [English translation]
      ```
      [2-3 more examples with varied contexts or politeness levels]

      nuances field:
      - ❌ [Common error/incorrect usage]
      - ✅ [Correct form/explanation]
      - 🔄 [Comparison with a similar pattern, if applicable]

      pro_tip field:
      [Extra info on speech style, usage situations, or a memory trick.]
    </md_structure_en>

    <md_structure_ru>
      explanation field:
      [1-2 предложения: чёткое объяснение значения, функции и контекста использования. Укажите уровень вежливости, стиль или ограничения, если есть.]
      > ⚠️ **Важно:** [Ключевое предупреждение или особенность, которую часто упускают.]

      how_to_form field:
      [Правило/формула. Используйте таблицу или список для разных типов слов/глаголов.]
      | Тип слова | Правило | Пример |
      |-----------|---------|--------|
      | ... | ... | ... |

      examples field:
      ```
      [Японское предложение]
      [Перевод на русский]
      ```
      [Ещё 2-3 примера с разными контекстами или уровнями вежливости]

      nuances field:
      - ❌ [Частая ошибка/неправильное использование]
      - ✅ [Правильный вариант/объяснение]
      - 🔄 [Сравнение с похожей конструкцией, если применимо]

      pro_tip field:
      [Дополнительная информация о стиле речи, ситуациях использования или мнемоника для запоминания.]
    </md_structure_ru>

    <quality_criteria>
      <criterion name="language_separation">
        EN section: English text ONLY in explanations. Japanese only in examples, patterns and tables.
        RU section: Russian text ONLY in explanations. Japanese only in examples, patterns and tables.
      </criterion>
      <criterion name="audience_en">
        English-speaking JLPT {level} student
      </criterion>
      <criterion name="audience_ru">
        Russian-speaking JLPT {level} student
      </criterion>
      <criterion name="title">
        title = the grammar pattern + a full-width-paren qualifier, e.g.
        ～の（nominalizer）. The pattern part before （） MUST be identical for
        both languages; the qualifier is localized (English for en, Russian
        for ru; established Japanese terms like （伝聞） are allowed in both).
        No space before （. At most two senses joined by ・. No JLPT level
        inside the qualifier. A qualifier is REQUIRED when the bare pattern
        is 2 kana or shorter (～も, ～の) — bare particles are
        indistinguishable without it.
      </criterion>
      <criterion name="brevity_en">
        EN short_description: 3-6 words
      </criterion>
      <criterion name="brevity_ru">
        RU short_description: 3-6 слов
      </criterion>
      <criterion name="examples">
        4-6 examples with translation in code blocks for each language version
      </criterion>
      <criterion name="tables">
        Use markdown tables for formation rules and paradigms
      </criterion>
      <criterion name="structure">
        Do NOT include a ## Title line — it is already in the title field.
        Fields explanation, how_to_form, examples are REQUIRED.
        Fields nuances and pro_tip are optional but recommended.
        If a field is not applicable, use an empty string.
      </criterion>
      <criterion name="examples_format">
        Examples MUST be in code blocks to prevent markdown parsing issues
      </criterion>
      <criterion name="tone_en">
        Address the reader as "you" in instructions, avoid academic jargon without explanation
      </criterion>
      <criterion name="tone_ru">
        Обращайтесь к читателю на "ты" в инструкциях, избегайте академического жаргона без пояснений
      </criterion>
    </quality_criteria>
  </success_brief>

  <example_output>
    {
      "en": {
        "title": "～の（nominalizer）",
        "short_description": "Nominalizer: action as a noun",
        "explanation": "`～の` is a **nominalizer** that converts verbs, clauses, or phrases into noun-like expressions. It allows you to treat an entire action or state as a noun that can be modified, followed by particles, or used as a subject/object.\n> ⚠️ **Important:** Always use the plain (dictionary) form before の: 食べるの, never 食べますの.",
        "how_to_form": "| Usage Type | Formation | Example |\n|------------|-----------|--------|\n| Basic nominalization | Verb (plain) + の | 食べるの |\n| With particles | Verb (plain) + の + は/が/を | 行くのが好き |",
        "examples": "```\n日本語を勉強するのが好きです。\nI like studying Japanese.\n```\n\n```\nあの本を読むのをお勧めします。\nI recommend reading that book.\n```",
        "nuances": "- ❌ Polite verb form before の (食べますの) → ✅ Always plain form: 食べるの\n- 🔄 の vs こと: の for concrete, direct experiences; こと for abstract ideas and reported speech",
        "pro_tip": "の after verbs often appears with 好き／嫌い and ～のが見えます／聞こえます. Think of の as putting the action \"in a box\" so you can talk about it."
      },
      "ru": {
        "title": "～の（номинализация）",
        "short_description": "Номинализация: действие как существительное",
        "explanation": "`～の` — это **номинализатор**, который превращает глаголы, словосочетания или целые клаузы в именные выражения. Благодаря ему действие или состояние можно использовать как существительное: с частицами, в роли подлежащего или дополнения.\n> ⚠️ **Важно:** Перед の всегда простая форма глагола: 食べるの, никогда 食べますの.",
        "how_to_form": "| Тип использования | Образование | Пример |\n|--------------------|-------------|--------|\n| Базовая номинализация | Глагол (простая форма) + の | 食べるの |\n| С частицами | Глагол (простая форма) + の + は/が/を | 行くのが好き |",
        "examples": "```\n日本語を勉強するのが好きです。\nМне нравится изучать японский язык.\n```\n\n```\nあの本を読むのをお勧めします。\nРекомендую прочитать ту книгу.\n```",
        "nuances": "- ❌ Вежливая форма глагола перед の (食べますの) → ✅ Всегда простая форма: 食べるの\n- 🔄 の vs こと: の — для конкретных, непосредственных действий; こと — для абстрактных идей и пересказа",
        "pro_tip": "の после глаголов часто встречается с 好き／嫌い и ～のが見えます／聞こえます. Представь, что の «упаковывает» действие, чтобы говорить о нём как о предмете."
      }
    }
  </example_output>

  <rules>
    <rule id="1">Return ONLY JSON, no markdown wrappers</rule>
    <rule id="2">Japanese text — only in examples, patterns and tables (for both EN and RU sections)</rule>
    <rule id="3">EN explanations — in English only; RU explanations — in Russian only</rule>
    <rule id="4">Format examples in code blocks: Japanese sentence, then translation</rule>
    <rule id="5">Use markdown tables for conjugation/formation rules</rule>
    <rule id="6">Both language versions must have the same structure (same fields)</rule>
    <rule id="7">The pattern part of title (before （qualifier）) must be identical for both EN and RU; the qualifier itself is localized. Full-width parens, no space before them.</rule>
    <rule id="8">Fields explanation, how_to_form, examples are mandatory. nuances and pro_tip are optional but recommended. Use empty string if not applicable.</rule>
  </rules>

  <conversation>
    <instruction>
      Return only JSON with both "en" and "ru" keys. No explanations before or after.
    </instruction>
  </conversation>
</prompt>
````
<!-- markdownlint-enable MD013 -->
