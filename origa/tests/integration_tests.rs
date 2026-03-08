use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, Mutex, Once};

use origa::domain::{
    init_grammar_rules, init_kanji_dictionary, init_vocabulary_dictionary, iter_grammar_rules,
    GrammarData, JapaneseLevel, KanjiData, NativeLanguage, OrigaError, User, VocabularyChunkData,
};
use origa::traits::{UserRepository, WellKnownSet, WellKnownSetLoader, WellKnownSetMeta};
use origa::use_cases::{
    CreateGrammarCardUseCase, GrammarRuleInfoUseCase, ImportWellKnownSetUseCase,
    KanjiInfoListUseCase, ListWellKnownSetsUseCase,
};
use origa::domain::Card;
use ulid::Ulid;

static INIT: Once = Once::new();

fn get_public_dir() -> PathBuf {
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR not set");
    PathBuf::from(manifest_dir)
        .parent()
        .expect("parent dir")
        .join("origa_ui")
        .join("public")
}

fn init_real_dictionaries() {
    INIT.call_once(|| {
        let public_dir = get_public_dir();

        let vocab_dir = public_dir.join("domain").join("dictionary").join("vocabulary");
        let vocab_data = VocabularyChunkData {
            chunk_01: std::fs::read_to_string(vocab_dir.join("chunk_01.json")).expect("chunk_01"),
            chunk_02: std::fs::read_to_string(vocab_dir.join("chunk_02.json")).expect("chunk_02"),
            chunk_03: std::fs::read_to_string(vocab_dir.join("chunk_03.json")).expect("chunk_03"),
            chunk_04: std::fs::read_to_string(vocab_dir.join("chunk_04.json")).expect("chunk_04"),
            chunk_05: std::fs::read_to_string(vocab_dir.join("chunk_05.json")).expect("chunk_05"),
            chunk_06: std::fs::read_to_string(vocab_dir.join("chunk_06.json")).expect("chunk_06"),
            chunk_07: std::fs::read_to_string(vocab_dir.join("chunk_07.json")).expect("chunk_07"),
            chunk_08: std::fs::read_to_string(vocab_dir.join("chunk_08.json")).expect("chunk_08"),
            chunk_09: std::fs::read_to_string(vocab_dir.join("chunk_09.json")).expect("chunk_09"),
            chunk_10: std::fs::read_to_string(vocab_dir.join("chunk_10.json")).expect("chunk_10"),
            chunk_11: std::fs::read_to_string(vocab_dir.join("chunk_11.json")).expect("chunk_11"),
        };
        init_vocabulary_dictionary(vocab_data).expect("Failed to init vocabulary");

        let kanji_path = public_dir
            .join("domain")
            .join("dictionary")
            .join("kanji.json");
        let kanji_data = KanjiData {
            kanji_json: std::fs::read_to_string(kanji_path).expect("kanji.json"),
        };
        init_kanji_dictionary(kanji_data).expect("Failed to init kanji");

        let grammar_path = public_dir.join("domain").join("grammar").join("grammar.json");
        let grammar_data = GrammarData {
            grammar_json: std::fs::read_to_string(grammar_path).expect("grammar.json"),
        };
        init_grammar_rules(grammar_data).expect("Failed to init grammar");
    });
}

#[derive(Clone)]
struct InMemoryUserRepository {
    users: Arc<Mutex<HashMap<Ulid, User>>>,
}

impl InMemoryUserRepository {
    fn new() -> Self {
        Self {
            users: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    fn with_user(user: User) -> Self {
        let repo = Self::new();
        let id = user.id();
        repo.users.lock().unwrap().insert(id, user);
        repo
    }
}

impl UserRepository for InMemoryUserRepository {
    async fn find_by_id(&self, user_id: Ulid) -> Result<Option<User>, OrigaError> {
        Ok(self.users.lock().unwrap().get(&user_id).cloned())
    }

    async fn find_by_email(&self, email: &str) -> Result<Option<User>, OrigaError> {
        Ok(self
            .users
            .lock()
            .unwrap()
            .values()
            .find(|u| u.email() == email)
            .cloned())
    }

    async fn find_by_telegram_id(&self, telegram_id: &u64) -> Result<Option<User>, OrigaError> {
        Ok(self
            .users
            .lock()
            .unwrap()
            .values()
            .find(|u| u.telegram_user_id() == Some(telegram_id))
            .cloned())
    }

    async fn save(&self, user: &User) -> Result<(), OrigaError> {
        self.users.lock().unwrap().insert(user.id(), user.clone());
        Ok(())
    }

    async fn save_sync(&self, user: &User) -> Result<(), OrigaError> {
        self.save(user).await
    }

    async fn delete(&self, user_id: Ulid) -> Result<(), OrigaError> {
        self.users.lock().unwrap().remove(&user_id);
        Ok(())
    }
}

#[tokio::test]
async fn grammar_rules_loads_from_real_file() {
    init_real_dictionaries();

    let rules: Vec<_> = iter_grammar_rules().collect();
    assert!(!rules.is_empty(), "Grammar rules should not be empty");

    let n5_rules: Vec<_> = rules.iter().filter(|r| r.level() == &JapaneseLevel::N5).collect();
    assert!(!n5_rules.is_empty(), "Should have N5 grammar rules");

    let first_n5 = n5_rules.first().unwrap();
    assert!(!first_n5.content(&NativeLanguage::Russian).title().is_empty());
}

#[tokio::test]
async fn grammar_rule_info_returns_rules_for_level() {
    init_real_dictionaries();

    let user = User::new("test@example.com".to_string(), NativeLanguage::Russian, None);
    let repo = InMemoryUserRepository::with_user(user);
    let use_case = GrammarRuleInfoUseCase::new(&repo);

    let user_id = repo.users.lock().unwrap().keys().next().copied().unwrap();
    let result = use_case
        .execute(user_id, &JapaneseLevel::N5, &std::collections::HashSet::new())
        .await
        .unwrap();

    assert!(!result.is_empty(), "Should return N5 grammar rules");

    let first = result.first().unwrap();
    assert!(!first.title.is_empty());
    assert!(!first.short_description.is_empty());
    assert!(!first.md_description.is_empty());
}

#[tokio::test]
async fn create_grammar_card_creates_from_real_rule() {
    init_real_dictionaries();

    let user = User::new("test@example.com".to_string(), NativeLanguage::Russian, None);
    let user_id = user.id();
    let repo = InMemoryUserRepository::with_user(user);
    let use_case = CreateGrammarCardUseCase::new(&repo);

    let first_rule = iter_grammar_rules()
        .find(|r| r.level() == &JapaneseLevel::N5)
        .expect("Should have N5 rule");
    let rule_id = *first_rule.rule_id();

    let cards = use_case.execute(user_id, vec![rule_id]).await.unwrap();

    assert_eq!(cards.len(), 1);

    let saved_user = repo.find_by_id(user_id).await.unwrap().unwrap();
    assert!(saved_user.knowledge_set().study_cards().contains_key(&cards[0].card_id()));
}

#[tokio::test]
async fn kanji_list_returns_real_kanji_for_level() {
    init_real_dictionaries();

    let user = User::new("test@example.com".to_string(), NativeLanguage::Russian, None);
    let repo = InMemoryUserRepository::with_user(user);
    let use_case = KanjiInfoListUseCase::new(&repo);

    let user_id = repo.users.lock().unwrap().keys().next().copied().unwrap();
    let kanji_list = use_case.execute(user_id, &JapaneseLevel::N5).await.unwrap();

    assert!(!kanji_list.is_empty(), "Should return N5 kanji");
}

#[tokio::test]
async fn kanji_list_excludes_learned_kanji() {
    init_real_dictionaries();

    let mut user = User::new("test@example.com".to_string(), NativeLanguage::Russian, None);
    let user_id = user.id();

    let kanji_card = origa::domain::KanjiCard::new("一".to_string(), &NativeLanguage::Russian).unwrap();
    user.create_card(Card::Kanji(kanji_card)).unwrap();

    let repo = InMemoryUserRepository::with_user(user);
    let use_case = KanjiInfoListUseCase::new(&repo);

    let kanji_list = use_case.execute(user_id, &JapaneseLevel::N5).await.unwrap();

    let learned_kanji: Vec<_> = kanji_list.iter().filter(|k| k.kanji == '一').collect();
    assert!(learned_kanji.is_empty(), "Should exclude learned kanji");
}

struct FileWellKnownSetLoader {
    public_dir: PathBuf,
}

impl FileWellKnownSetLoader {
    fn new() -> Self {
        Self {
            public_dir: get_public_dir(),
        }
    }
}

impl WellKnownSetLoader for FileWellKnownSetLoader {
    async fn load_meta_list(&self) -> Result<Vec<WellKnownSetMeta>, OrigaError> {
        let path = self.public_dir.join("domain").join("well_known_set").join("well_known_sets_meta.json");
        let json = std::fs::read_to_string(&path).map_err(|e| OrigaError::WellKnownSetParseError {
            reason: format!("Failed to read meta list: {}", e),
        })?;
        serde_json::from_str(&json).map_err(|e| OrigaError::WellKnownSetParseError {
            reason: format!("Failed to parse meta list: {}", e),
        })
    }

    async fn load_set(&self, id: String) -> Result<WellKnownSet, OrigaError> {
        #[derive(serde::Deserialize)]
        struct SetData {
            level: JapaneseLevel,
            words: Vec<String>,
        }

        let path = self.id_to_path(&id);
        let json = std::fs::read_to_string(&path).map_err(|e| OrigaError::WellKnownSetParseError {
            reason: format!("Failed to read set {}: {}", id, e),
        })?;
        
        let data: SetData = serde_json::from_str(&json).map_err(|e| OrigaError::WellKnownSetParseError {
            reason: format!("Failed to parse set {}: {}", id, e),
        })?;

        Ok(WellKnownSet::new(data.level, data.words))
    }
}

impl FileWellKnownSetLoader {
    fn id_to_path(&self, id: &str) -> PathBuf {
        if let Some(level) = id.strip_prefix("jlpt_") {
            self.public_dir
                .join("domain")
                .join("well_known_set")
                .join(format!("jltp_{}.json", level))
        } else {
            self.public_dir
                .join("domain")
                .join("well_known_set")
                .join(format!("{}.json", id))
        }
    }
}

#[tokio::test]
async fn list_well_known_sets_returns_jlpt_sets() {
    let loader = FileWellKnownSetLoader::new();
    let use_case = ListWellKnownSetsUseCase::new(&loader);

    let result = use_case.execute().await.unwrap();

    let jlpt_sets: Vec<_> = result
        .iter()
        .filter(|s| s.meta.set_type == origa::traits::SetType::Jlpt)
        .collect();

    assert!(!jlpt_sets.is_empty(), "Should have JLPT sets");

    let n5_set = jlpt_sets.iter().find(|s| s.meta.level == JapaneseLevel::N5);
    assert!(n5_set.is_some(), "Should have N5 set");
}

#[tokio::test]
async fn load_well_known_set_n5_returns_words() {
    let loader = FileWellKnownSetLoader::new();

    let set = loader.load_set("jlpt_n5".to_string()).await.unwrap();

    assert_eq!(set.level(), &JapaneseLevel::N5);
    assert!(!set.words().is_empty(), "N5 set should have words");
}

#[tokio::test]
async fn well_known_set_preview_shows_known_words() {
    init_real_dictionaries();

    let mut user = User::new("test@example.com".to_string(), NativeLanguage::Russian, None);
    let user_id = user.id();

    let loader = FileWellKnownSetLoader::new();
    let repo = InMemoryUserRepository::with_user(user);
    let use_case = ImportWellKnownSetUseCase::new(&repo, &loader);

    let preview = use_case
        .preview_set(user_id, "jlpt_n5".to_string())
        .await
        .unwrap();

    assert!(!preview.words.is_empty(), "Preview should have words");
    assert_eq!(preview.known_count, 0, "New user should have no known words");
}
