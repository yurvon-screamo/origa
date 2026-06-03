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
        title must be the grammar pattern itself (e.g. ～ます, ～てください) — same for both languages
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
        "title": "～ます",
        "short_description": "Polite present tense form",
        "explanation": "`～ます` is the **polite verb form** for present and future tense. Used with strangers, coworkers, and superiors in any situation requiring politness.\n> ⚠️ **Important:** Japanese has no separate future tense. Context determines if the action happens now or later.",
        "how_to_form": "| Verb Group | Rule | Example |\n|------------|------|--------|\n| Group 1 (う-verbs) | Last う-row sound → い-row + ます | 書く → 書きます |\n| Group 2 (る-verbs) | Remove る + ます | 食べる → 食べます |\n| Group 3 (irregular) | する → します / くる → 来ます | する → します |",
        "examples": "```\n毎朝コーヒーを飲みます。\nI drink coffee every morning.\n```\n\n```\n明日東京へ行きます。\nI will go to Tokyo tomorrow.\n```",
        "nuances": "- ❌ Reading `来ます` as `くます` → ✅ Correct: `きます`\n- ❌ Confusing verb groups (not all る-verbs are Group 2) → ✅ Remember exceptions: `知る`, `帰る`, `切る` are Group 1\n- 🔄 With friends, use plain form (`飲む`, `食べる`), but in office/school always use `～ます`",
        "pro_tip": "When you hear `～ます` at the end of a sentence, you're in a polite or formal context. The more `ます`-forms, the more respectful the speech sounds."
      },
      "ru": {
        "title": "～ます",
        "short_description": "Вежливая форма настоящего времени",
        "explanation": "`～ます` — стандартная вежливая форма глагола для настоящего и будущего времени. Используется с незнакомыми, коллегами, старшими и в любой ситуации, где нужна вежливость.\n> ⚠️ **Важно:** В японском нет отдельного будущего времени. Контекст определяет, происходит действие сейчас или позже.",
        "how_to_form": "| Группа глаголов | Правило | Пример |\n|-----------------|---------|--------|\n| Группа 1 (う-глаголы) | Последний звук у-ряда → и-ряд + ます | 書く → 書きます |\n| Группа 2 (る-глаголы) | Убрать る + ます | 食べる → 食べます |\n| Группа 3 (исключения) | する → します / くる → 来ます | する → します |",
        "examples": "```\n毎朝コーヒーを飲みます。\nКаждое утро я пью кофе.\n```\n\n```\n明日東京へ行きます。\nЗавтра я поеду в Токио.\n```",
        "nuances": "- ❌ Читать `来ます` как `くます` → ✅ Правильно: `きます`\n- ❌ Путать группы глаголов (не все る-глаголы относятся ко 2-й группе) → ✅ Запомните исключения: `知る`, `帰る`, `切る` — это Группа 1\n- 🔄 С друзьями можно использовать простую форму (`飲む`, `食べる`), но в офисе/учебе всегда `～ます`",
        "pro_tip": "Если слышите `～ます` в конце предложения — вы находитесь в вежливом или официальном контексте. Чем больше `ます`-форм, тем уважительнее звучит речь."
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
    <rule id="7">The title field must be identical for both EN and RU (the grammar pattern itself)</rule>
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
