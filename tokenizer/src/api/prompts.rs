/// Generates a prompt for Russian translation of a Japanese word
pub fn get_russian_translation_prompt(word: &str) -> String {
    format!(
        r#"<prompt>
  <task>
    Ты — профессиональный лексикограф-японист. Твоя задача: дать точные, минимальные, но исчерпывающие значения японского слова на русском языке.
  </task>

  <word>
    {}
  </word>

  <requirements>
    1. Если слово имеет несколько значений, дай все основные значения через запятую.
    2. Если есть синонимы, укажи их в скобках.
    3. Укажи часть речи в квадратных скобках в начале.
    4. Не добавляй никаких объяснений или комментариев.
    5. Формат: [часть речи] значение1, значение2 (синоним)
  </pattern>

  <examples>
    食べる -> [гл.] есть, кушать
    美味しい -> [прил.] вкусный, аппетитный
    先生 -> [сущ.] учитель, преподаватель, мастер
    走る -> [гл.] бежать, бегать
  </examples>
</prompt>"#,
        word
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
    {}
  </word>

  <requirements>
    1. If the word has multiple meanings, provide all main meanings separated by commas.
    2. If there are synonyms, include them in parentheses.
    3. Indicate the part of speech in square brackets at the beginning.
    4. Do not add any explanations or comments.
    5. Format: [part of speech] meaning1, meaning2 (synonym)
  </requirements>

  <examples>
    食べる -> [v.] to eat
    美味しい -> [adj.] delicious, tasty
    先生 -> [n.] teacher, instructor, master
    走る -> [v.] to run
  </examples>
</prompt>"#,
        word
    )
}
