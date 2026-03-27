/// Generates a prompt for Russian translation of a Japanese word
pub fn get_russian_translation_prompt(word: &str) -> String {
    format!(
        r#"<prompt>
          <task>
            Ты — профессиональный лексикограф-японист. Твоя задача: дать точные, минимальные, но исчерпывающие значения японского слова на русском языке.
          </task>

          <word>
            {word}
          </word>

          <success_brief>
            <format>
              <description>Строго соблюдай следующий формат вывода (markdown):</description>
              <template>
        <![CDATA[
        - Перевод 1
        - Перевод 2
        - Перевод 3

        > Комментарий (только при острой необходимости)
        ]]>
              </template>
            </format>

            <quality_criteria>
              <criterion name="минимализм">Не дублируй смыслы (избегай: "вечер/вечером/вечерний", "стирать/стирка")</criterion>
              <criterion name="разные значения">Каждое значение должно быть семантически отличным от других</criterion>
              <criterion name="язык">ТОЛЬКО русский текст (никакого японского: кандзи, каны, ромадзи, прочтений)</criterion>
              <criterion name="структура">Маркированный список + опционально блок цитаты для комментария</criterion>
              <criterion name="объём">1-5 значений максимум (для многозначных слов), 1-2 для однозначных</criterion>
            </quality_criteria>
          </success_brief>

          <rules>
            <rule id="1">НЕ дублируй грамматические формы одного слова (существительное ≠ глагол от того же корня, если смысл тот же)</rule>
            <rule id="2">НЕ добавляй японский текст в ответ (никаких прочтений, кандзи, примеров на японском)</rule>
            <rule id="3">НЕ пиши вступление, заключение, пояснения перед списком (сразу начинай со списка)</rule>
            <rule id="4">Комментарий в блоке цитаты — только если значение неочевидно или требуется уточнение контекста</rule>
            <rule id="5">Группируй близкие семантически смыслы в одно значение</rule>
            <rule id="6">Если слово имеет омонимы — указывай только основные значения, не все возможные</rule>
            <rule id="7">Приоритет: частотные значения → редкие значения</rule>
          </rules>

          <examples>
            <good>
              <word>かける (kakeru)</word>
              <output>
        <![CDATA[
        - Вешать
        - Тратить (время, деньги)
        - Звонить (по телефону)
        - Надевать (очки, страховку)
        ]]>
              </output>
              <reason>4 разных семантических значения одного глагола</reason>
            </good>

            <good>
              <word>重い (omoi)</word>
              <output>
        <![CDATA[
        - Тяжёлый
        - Серьёзный (об ошибке, болезни)

        > В зависимости от контекста: физический вес или степень важности
        ]]>
              </output>
              <reason>2 значения + комментарий для уточнения</reason>
            </good>

            <good>
              <word>冷蔵庫 (reizouko)</word>
              <output>
        <![CDATA[
        - Холодильник
        ]]>
              </output>
              <reason>Одно значение, не нужно дублировать</reason>
            </good>

            <bad>
              <word>冷蔵庫 (reizouko)</word>
              <output>
        <![CDATA[
        - Холодильник
        - Морозильная камера
        - Холод
        - Охлаждение
        ]]>
              </output>
              <reason>Избыточно: по сути одно устройство, остальные — не значения слова</reason>
            </bad>

            <bad>
              <word>夕べ (yuube)</word>
              <output>
        <![CDATA[
        - Вечер
        - Вечером
        - Вечерний
        ]]>
              </output>
              <reason>Грамматические формы одного смысла, нужно одно значение</reason>
            </bad>
          </examples>

          <conversation>
            <instruction>
              Отвечай только markdown по заданному формату
            </instruction>
          </conversation>
        </prompt>"#,
    )
}

/// Generates a prompt for English translation of a Japanese word
pub fn get_english_translation_prompt(word: &str) -> String {
    format!(
        r#"<prompt>
          <task>
            You are a professional Japanese lexicographer. Your task: provide accurate, minimal yet comprehensive meanings of the Japanese word in English.
          </task>

          <word>
            {word}
          </word>

          <success_brief>
            <format>
              <description>Strictly follow this output format (markdown):</description>
              <template>
        <![CDATA[
        - Meaning 1
        - Meaning 2
        - Meaning 3

        > Comment (only when absolutely necessary)
        ]]>
              </template>
            </format>

            <quality_criteria>
              <criterion name="minimalism">Do not duplicate senses (avoid: "evening/evening time/in the evening", "to wash/washing")</criterion>
              <criterion name="distinct meanings">Each meaning must be semantically distinct from others</criterion>
              <criterion name="language">ENGLISH TEXT ONLY (no Japanese: kanji, kana, romaji, readings)</criterion>
              <criterion name="structure">Bulleted list + optional blockquote for comments</criterion>
              <criterion name="volume">1-5 meanings max (for polysemous words), 1-2 for unambiguous words</criterion>
            </quality_criteria>
          </success_brief>

          <rules>
            <rule id="1">Do NOT duplicate grammatical forms of the same word (noun ≠ verb from the same root if the meaning is identical)</rule>
            <rule id="2">Do NOT include Japanese text in your response (no readings, kanji, Japanese examples)</rule>
            <rule id="3">Do NOT write introductions, conclusions, or explanations before the list (start immediately with the list)</rule>
            <rule id="4">Comment in blockquote — only if the meaning is not obvious or requires context clarification</rule>
            <rule id="5">Group semantically similar senses into one meaning</rule>
            <rule id="6">If the word has homonyms — indicate only the main meanings, not all possible ones</rule>
            <rule id="7">Priority: frequent meanings → rare meanings</rule>
          </rules>

          <examples>
            <good>
              <word>かける (kakeru)</word>
              <output>
        <![CDATA[
        - To hang (up)
        - To spend (time, money)
        - To call (on the phone)
        - To put on (glasses, seatbelt)
        ]]>
              </output>
              <reason>4 different semantic meanings of one verb</reason>
            </good>

            <good>
              <word>重い (omoi)</word>
              <output>
        <![CDATA[
        - Heavy
        - Serious (about a mistake, illness)

        > Context-dependent: physical weight or degree of importance
        ]]>
              </output>
              <reason>2 meanings + comment for clarification</reason>
            </good>

            <good>
              <word>冷蔵庫 (reizouko)</word>
              <output>
        <![CDATA[
        - Refrigerator
        ]]>
              </output>
              <reason>Single meaning, no need to duplicate</reason>
            </good>

            <bad>
              <word>冷蔵庫 (reizouko)</word>
              <output>
        <![CDATA[
        - Refrigerator
        - Freezer
        - Cold
        - Cooling
        ]]>
              </output>
              <reason>Excessive: essentially one device, others are not meanings of the word</reason>
            </bad>

            <bad>
              <word>夕べ (yuube)</word>
              <output>
        <![CDATA[
        - Evening
        - In the evening
        - Evening (adjective)
        ]]>
              </output>
              <reason>Grammatical forms of the same sense, need one meaning</reason>
            </bad>
          </examples>

          <conversation>
            <instruction>
              Respond only in markdown using the specified format
            </instruction>
          </conversation>
        </prompt>"#,
    )
}
