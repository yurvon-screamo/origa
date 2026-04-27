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
      Return ONLY valid JSON (no markdown wrappers). Format:
      {
        "en": {
          "title": "grammar pattern",
          "short_description": "brief description",
          "md_description": "full markdown description in English"
        },
        "ru": {
          "title": "grammar pattern",
          "short_description": "краткое описание",
          "md_description": "полное описание на русском"
        }
      }
    </output_format>

    <md_structure_en>
      ## [Pattern/Construction] — [Short Description/Translation]

      ### 📖 What is it?
      [1-2 sentences: clear explanation of meaning, function, and usage context. Mention politeness, register, or restrictions if applicable.]
      > ⚠️ **Important:** [Key warning or commonly missed detail.]

      ---

      ### 🔧 How to form it?
      [Rule/formula. Use a table or list for different word/verb types.]
      | Word type | Rule | Example |
      |-----------|------|---------|
      | ... | ... | ... |

      ---

      ### 🗣️ Examples in Context
      ```
      [Japanese sentence]
      [English translation]
      ```
      [2-3 more examples with varied contexts or politeness levels]

      ---

      ### 💡 Nuances & Common Mistakes
      - ❌ [Common error/incorrect usage]
      - ✅ [Correct form/explanation]
      - 🔄 [Comparison with a similar pattern, if applicable]

      ---

      ### 🌸 Pro Tip / Cultural Note (optional)
      [Extra info on speech style, usage situations, or a memory trick. Delete if not needed.]
    </md_structure_en>

    <md_structure_ru>
      ## [Конструкция] — [Краткое описание/Перевод]

      ### 📖 Что это и зачем?
      [1-2 предложения: чёткое объяснение значения, функции и контекста использования. Укажите уровень вежливости, стиль или ограничения, если есть.]
      > ⚠️ **Важно:** [Ключевое предупреждение или особенность, которую часто упускают.]

      ---

      ### 🔧 Как образуется?
      [Правило/формула. Используйте таблицу или список для разных типов слов/глаголов.]
      | Тип слова | Правило | Пример |
      |-----------|---------|--------|
      | ... | ... | ... |

      ---

      ### 🗣️ Примеры в контексте
      ```
      [Японское предложение]
      [Перевод на русский]
      ```
      [Ещё 2-3 примера с разными контекстами или уровнями вежливости]

      ---

      ### 💡 Нюансы и типичные ошибки
      - ❌ [Частая ошибка/неправильное использование]
      - ✅ [Правильный вариант/объяснение]
      - 🔄 [Сравнение с похожей конструкцией, если применимо]

      ---

      ### 🌸 Лайфхак / Культурный контекст (опционально)
      [Дополнительная информация о стиле речи, ситуациях использования или мнемоника для запоминания. Если не нужно, раздел можно удалить.]
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
        Use ONLY ## for main title and ### for sections. No #### subsections.
        First 3 sections (What is it? / Что это?, How to form / Как образуется, Examples / Примеры) are REQUIRED.
        Last 2 sections (Nuances / Нюансы, Pro Tip / Лайфхак) are optional but recommended.
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
        "md_description": "## ～ます — Polite Present/Future Tense\n\n### 📖 What is it?\n\n`～ます` is the **polite verb form** for present and future tense. Used with strangers, coworkers, and superiors in any situation requiring politeness.\n> ⚠️ **Important:** Japanese has no separate future tense. Context determines if the action happens now or later.\n\n---\n\n### 🔧 How to form it?\n\n| Verb Group | Rule | Example |\n|------------|------|--------|\n| Group 1 (う-verbs) | Last う-row sound → い-row + ます | 書く → 書きます |\n| Group 2 (る-verbs) | Remove る + ます | 食べる → 食べます |\n| Group 3 (irregular) | する → します / くる → 来ます | する → します |\n\n---\n\n### 🗣️ Examples in Context\n\n```\n毎朝コーヒーを飲みます。\nI drink coffee every morning.\n```\n\n```\n明日東京へ行きます。\nI will go to Tokyo tomorrow.\n```\n\n---\n\n### 💡 Nuances & Common Mistakes\n\n- ❌ Reading `来ます` as `くます` → ✅ Correct: `きます`\n- ❌ Confusing verb groups (not all る-verbs are Group 2) → ✅ Remember exceptions: `知る`, `帰る`, `切る` are Group 1\n- 🔄 With friends, use plain form (`飲む`, `食べる`), but in office/school always use `～ます`\n\n---\n\n### 🌸 Pro Tip\n\nWhen you hear `～ます` at the end of a sentence, you're in a polite or formal context. The more `ます`-forms, the more respectful the speech sounds."
      },
      "ru": {
        "title": "～ます",
        "short_description": "Вежливая форма настоящего времени",
        "md_description": "## ～ます — Вежливая форма настоящего/будущего времени\n\n### 📖 Что это и зачем?\n\n`～ます` — стандартная вежливая форма глагола для настоящего и будущего времени. Используется с незнакомыми, коллегами, старшими и в любой ситуации, где нужна вежливость.\n> ⚠️ **Важно:** В японском нет отдельного будущего времени. Контекст определяет, происходит действие сейчас или позже.\n\n---\n\n### 🔧 Как образуется?\n\n| Группа глаголов | Правило | Пример |\n|-----------------|---------|--------|\n| Группа 1 (う-глаголы) | Последний звук у-ряда → и-ряд + ます | 書く → 書きます |\n| Группа 2 (る-глаголы) | Убрать る + ます | 食べる → 食べます |\n| Группа 3 (исключения) | する → します / くる → 来ます | する → します |\n\n---\n\n### 🗣️ Примеры в контексте\n\n```\n毎朝コーヒーを飲みます。\nКаждое утро я пью кофе.\n```\n\n```\n明日東京へ行きます。\nЗавтра я поеду в Токио.\n```\n\n---\n\n### 💡 Нюансы и типичные ошибки\n\n- ❌ Читать `来ます` как `くます` → ✅ Правильно: `きます`\n- ❌ Путать группы глаголов (не все る-глаголы относятся ко 2-й группе) → ✅ Запомните исключения: `知る`, `帰る`, `切る` — это Группа 1\n- 🔄 С друзьями можно использовать простую форму (`飲む`, `食べる`), но в офисе/учебе всегда `～ます`\n\n---\n\n### 🌸 Лайфхак\n\nЕсли слышите `～ます` в конце предложения — вы находитесь в вежливом или официальном контексте. Чем больше `ます`-форм, тем уважительнее звучит речь."
      }
    }
  </example_output>

  <rules>
    <rule id="1">Return ONLY JSON, no markdown wrappers</rule>
    <rule id="2">Japanese text — only in examples, patterns and tables (for both EN and RU sections)</rule>
    <rule id="3">EN explanations — in English only; RU explanations — in Russian only</rule>
    <rule id="4">Format examples in code blocks: Japanese sentence, then translation</rule>
    <rule id="5">Use markdown tables for conjugation/formation rules</rule>
    <rule id="6">Both language versions must have the same structure (same sections)</rule>
    <rule id="7">The title field must be identical for both EN and RU (the grammar pattern itself)</rule>
    <rule id="8">If a section is not applicable (e.g., particles have no conjugation), omit it — but first 3 sections are mandatory</rule>
  </rules>

  <conversation>
    <instruction>
      Return only JSON with both "en" and "ru" keys. No explanations before or after.
    </instruction>
  </conversation>
</prompt>
````
<!-- markdownlint-enable MD013 -->