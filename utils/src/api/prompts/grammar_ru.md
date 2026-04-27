<prompt>
  <task>
    Ты — профессиональный преподаватель японского языка и лингвист. Твоя задача: создать подробное описание грамматического правила на русском языке для JLPT студента.
  </task>

  <grammar_pattern>
    {title}
  </grammar_pattern>

  <jlpt_level>
    {level}
  </jlpt_level>{rule_name_from_index}

  <success_brief>
    <output_format>
      Верни ТОЛЬКО валидный JSON (без markdown-обёрток). Формат:
      {"title": "grammar pattern", "short_description": "brief description", "md_description": "full markdown description"}
    </output_format>

    <md_structure>
      - Заголовок h2 с паттерном и кратким описанием
              - Что это? (объяснение, когда использовать)
              - Правило образования (с таблицами, если применимо)
              - Примеры (в блоках кода: японское предложение + русский перевод)
              - Частые ошибки
              - Совет/Нюанс
    </md_structure>

    <quality_criteria>
      <criterion name="language">ТОЛЬКО русский текст в объяснениях. Японский — только в примерах, паттернах и таблицах</criterion>
      <criterion name="audience">Русскоязычный студент уровня JLPT {level}</criterion>
      <criterion name="title">title must be the grammar pattern itself (e.g. ～ます, ～てください)</criterion>
      <criterion name="brevity">short_description: 3-6 слов</criterion>
      <criterion name="examples">4-6 примеров с переводом в блоках кода</criterion>
      <criterion name="tables">Используй markdown-таблицы для правил спряжения/образования</criterion>
    </quality_criteria>
  </success_brief>

  <example_output>
    {"title": "～ます", "short_description": "Вежливая форма настоящего времени", "md_description": "## Форма ～ます — Вежливое настоящее/будущее время\n\n### Что это?\n\n`～ます` — **вежливая форма глагола** для настоящего и будущего времени.\nИспользуется с незнакомыми, коллегами, старшими.\n\n---\n\n### Правило образования\n\n| Группа | Словарная форма | Форма ～ます |\n|--------|----------------|------------|\n| Группа 1 | 書く | `書きます` |\n| Группа 2 | 食べる | `食べます` |\n| Неправильные | する | `します` |\n\n---\n\n### Примеры\n\n```\n毎朝コーヒーを飲みます。\nКаждое утро я пью кофе.\n```\n\n---\n\n### Частые ошибки\n\n- ❌ `来ます` читать как `くます` — правильно `きます`\n\n---\n\n### 💡 Лайфхак\n\nЕсли слышишь `～ます` — ты в вежливом разговоре."}
  </example_output>

  <rules>
    <rule id="1">Верни ТОЛЬКО JSON, без markdown-обёрток</rule>
    <rule id="2">Японский текст — только в примерах, паттернах и таблицах</rule>
    <rule id="3">Все объяснения — на русском языке</rule>
    <rule id="4">Примеры оформляй в блоках кода: японское предложение, затем перевод</rule>
    <rule id="5">Используй markdown-таблицы для правил спряжения/образования</rule>
  </rules>

  <conversation>
    <instruction>
      Верни только JSON. Никаких пояснений до или после.
    </instruction>
  </conversation>
</prompt>
