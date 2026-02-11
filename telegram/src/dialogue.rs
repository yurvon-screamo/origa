use origa::domain::JapaneseLevel;
use ulid::Ulid;

#[derive(Clone, Default)]
pub struct SessionData {
    pub user_id: Ulid,
    pub username: String,
}

#[derive(Clone, Default)]
pub enum DialogueState {
    #[default]
    Idle,
    Lesson {
        mode: LessonMode,
        card_ids: Vec<Ulid>,
        current_index: usize,
        showing_answer: bool,
        new_count: usize,
        review_count: usize,
    },
    VocabularyList {
        page: usize,
        items_per_page: usize,
        filter: String,
    },
    VocabularySearch {
        page: usize,
        items_per_page: usize,
        query: String,
    },
    KanjiList {
        level: Option<JapaneseLevel>,
        page: usize,
        items_per_page: usize,
    },
    GrammarList {
        page: usize,
        items_per_page: usize,
    },
    Profile {
        current_view: ProfileView,
    },
    DuolingoConnect,
    AddFromText {
        pending_words: Vec<String>,
    },
}

#[derive(Clone, Copy, Default, PartialEq)]
pub enum ProfileView {
    #[default]
    Main,
    Settings,
    JlptSelect,
}

#[derive(Clone, Copy, PartialEq)]
pub enum LessonMode {
    Lesson,
    Fixation,
}
