use origa::domain::{Answer, Card, JapaneseLevel, NativeLanguage, Question, User};

pub fn create_demo_user() -> User {
    let mut user = User::new(
        "demo".to_string(),
        JapaneseLevel::N5,
        NativeLanguage::Russian,
        None,
    );

    let demo_cards = vec![
        create_vocabulary_card("猫", "Кошка"),
        create_vocabulary_card("犬", "Собака"),
        create_vocabulary_card("水", "Вода"),
        create_vocabulary_card("火", "Огонь"),
        create_vocabulary_card("山", "Гора"),
        create_vocabulary_card("本", "Книга"),
        create_vocabulary_card("食べる", "Есть"),
        create_vocabulary_card("学校", "Школа"),
        create_vocabulary_card("先生", "Учитель"),
        create_vocabulary_card("勉強", "Учиться"),
        create_vocabulary_card("日本語", "Японский язык"),
        create_kanji_card("日"),
        create_kanji_card("月"),
        create_kanji_card("木"),
        create_kanji_card("人"),
        create_kanji_card("大"),
        create_kanji_card("小"),
    ];

    for card in demo_cards {
        if let Err(e) = user.create_card(card) {
            eprintln!("Error creating demo card: {}", e);
        }
    }

    user
}

fn create_vocabulary_card(word: &str, meaning: &str) -> Card {
    Card::Vocabulary(origa::domain::VocabularyCard::new(
        Question::new(word.to_string()).unwrap(),
        Answer::new(meaning.to_string()).unwrap(),
    ))
}

fn create_kanji_card(kanji: &str) -> Card {
    match origa::domain::KanjiCard::new(kanji.to_string(), &NativeLanguage::Russian) {
        Ok(card) => Card::Kanji(card),
        Err(_) => Card::Kanji(
            origa::domain::KanjiCard::new(kanji.to_string(), &NativeLanguage::Russian)
                .unwrap_or_else(|_| panic!("Failed to create kanji card for: {}", kanji)),
        ),
    }
}
