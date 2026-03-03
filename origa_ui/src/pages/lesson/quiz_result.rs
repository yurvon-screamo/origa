#[derive(Clone, Copy, PartialEq, Default, Debug)]
pub enum QuizResult {
    #[default]
    None,
    Correct,
    Incorrect,
}

impl QuizResult {
    pub fn option_class(&self, is_correct: bool) -> &'static str {
        match (self, is_correct) {
            (QuizResult::None, _) => {
                "bg-[var(--bg-paper)] hover:bg-[var(--bg-aged)] border-[var(--border-dark)]"
            }
            (QuizResult::Correct, true) | (QuizResult::Incorrect, true) => {
                "bg-[var(--bg-warm)] border-[var(--success)] text-[var(--success)]"
            }
            (QuizResult::Correct, false) => {
                "bg-[var(--bg-paper)] border-[var(--border-light)] opacity-50"
            }
            (QuizResult::Incorrect, false) => {
                "bg-[var(--bg-warm)] border-[var(--error)] text-[var(--error)]"
            }
        }
    }
}
