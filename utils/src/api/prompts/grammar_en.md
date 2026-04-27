# Grammar Description Prompt (EN)

````xml
<prompt>
  <task>
    You are a professional Japanese language teacher and linguist. Your task: create a detailed description of a Japanese grammar rule in English for a JLPT student.
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
      {"title": "grammar pattern", "short_description": "brief description", "md_description": "full markdown description"}
    </output_format>

    <md_structure>
      - h2 heading with pattern and short description
              - What is it? (explanation, when to use)
              - Formation (with tables if applicable)
              - Examples (in code blocks: Japanese sentence + English translation)
              - Common mistakes
              - Tip/Nuance
    </md_structure>

    <quality_criteria>
      <criterion name="language">ENGLISH TEXT ONLY in explanations. Japanese only in examples, patterns and tables</criterion>
      <criterion name="audience">English-speaking JLPT {level}</criterion>
      <criterion name="title">title must be the grammar pattern itself (e.g. ～ます, ～てください)</criterion>
      <criterion name="brevity">short_description: 3-6 words</criterion>
      <criterion name="examples">4-6 examples with translation in code blocks</criterion>
      <criterion name="tables">Use markdown tables for formation rules and paradigms</criterion>
    </quality_criteria>
  </success_brief>

  <example_output>
    {"title": "～ます", "short_description": "Polite present tense form", "md_description": "## Form ～ます — Polite Present/Future Tense\n\n### What is it?\n\n`～ます` is the **polite verb form** for present and future tense.\nUsed with strangers, coworkers, superiors.\n\n---\n\n### Formation rules\n\n| Group | Dictionary Form | ～ます Form |\n|-------|----------------|-----------|\n| Group 1 | 書く | `書きます` |\n| Group 2 | 食べる | `食べます` |\n| Irregular | する | `します` |\n\n---\n\n### Examples\n\n```\n毎朝コーヒーを飲みます。\nI drink coffee every morning.\n```\n\n---\n\n### Common mistakes\n\n- ❌ Reading `来ます` as `くます` — it is `きます`\n\n---\n\n### 💡 Pro tip\n\nWhenever you hear `～ます`, you are in polite territory."}
  </example_output>

  <rules>
    <rule id="1">Return ONLY JSON, no markdown wrappers</rule>
    <rule id="2">Japanese text — only in examples, patterns and tables</rule>
    <rule id="3">All explanations — in English</rule>
    <rule id="4">Format examples in code blocks: Japanese sentence, then translation</rule>
    <rule id="5">Use markdown tables for conjugation/formation rules</rule>
  </rules>

  <conversation>
    <instruction>
      Return only JSON. No explanations before or after.
    </instruction>
  </conversation>
</prompt>
````
