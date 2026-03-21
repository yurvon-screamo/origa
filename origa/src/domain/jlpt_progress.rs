use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::domain::JapaneseLevel;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CategoryProgress {
    pub learned: usize,
    pub total: usize,
}

impl CategoryProgress {
    pub fn new() -> Self {
        Self {
            learned: 0,
            total: 0,
        }
    }

    pub fn percentage(&self) -> f64 {
        if self.total == 0 {
            return 0.0;
        }
        (self.learned as f64 / self.total as f64) * 100.0
    }

    pub fn is_complete(&self, threshold: f64) -> bool {
        self.percentage() >= threshold
    }
}

impl Default for CategoryProgress {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LevelProgressDetail {
    pub kanji: CategoryProgress,
    pub words: CategoryProgress,
    pub grammar: CategoryProgress,
}

impl LevelProgressDetail {
    pub fn new() -> Self {
        Self {
            kanji: CategoryProgress::new(),
            words: CategoryProgress::new(),
            grammar: CategoryProgress::new(),
        }
    }

    pub fn overall_percentage(&self) -> f64 {
        (self.kanji.percentage() + self.words.percentage() + self.grammar.percentage()) / 3.0
    }

    pub fn is_complete(&self, threshold: f64) -> bool {
        self.overall_percentage() >= threshold
    }
}

impl Default for LevelProgressDetail {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct JlptProgress {
    levels: HashMap<JapaneseLevel, LevelProgressDetail>,
}

impl JlptProgress {
    pub fn new() -> Self {
        let mut levels = HashMap::new();
        for level in JapaneseLevel::ALL {
            levels.insert(level, LevelProgressDetail::new());
        }
        Self { levels }
    }

    pub fn current_level(&self) -> JapaneseLevel {
        let all_empty = JapaneseLevel::ALL.iter().all(|&level| {
            self.levels
                .get(&level)
                .map(|d| d.overall_percentage() == 0.0)
                .unwrap_or(true)
        });

        if all_empty {
            return JapaneseLevel::N5;
        }

        let completed_levels: Vec<_> = JapaneseLevel::ALL
            .iter()
            .filter(|&&level| {
                self.levels
                    .get(&level)
                    .map(|d| d.is_complete(90.0))
                    .unwrap_or(false)
            })
            .collect();

        if completed_levels.is_empty() {
            return JapaneseLevel::N5;
        }

        let max_completed = completed_levels
            .into_iter()
            .min_by_key(|&&level| level.as_number())
            .unwrap();

        match max_completed {
            JapaneseLevel::N1 => JapaneseLevel::N1,
            JapaneseLevel::N2 => JapaneseLevel::N1,
            JapaneseLevel::N3 => JapaneseLevel::N2,
            JapaneseLevel::N4 => JapaneseLevel::N3,
            JapaneseLevel::N5 => JapaneseLevel::N4,
        }
    }

    pub fn level_progress(&self, level: JapaneseLevel) -> Option<&LevelProgressDetail> {
        self.levels.get(&level)
    }

    pub fn update_level(&mut self, level: JapaneseLevel, detail: LevelProgressDetail) {
        self.levels.insert(level, detail);
    }

    pub fn recalculate(
        &mut self,
        learned_kanji: &HashMap<JapaneseLevel, usize>,
        learned_words: &HashMap<JapaneseLevel, usize>,
        learned_grammar: &HashMap<JapaneseLevel, usize>,
        total_kanji: &HashMap<JapaneseLevel, usize>,
        total_words: &HashMap<JapaneseLevel, usize>,
        total_grammar: &HashMap<JapaneseLevel, usize>,
    ) {
        for level in JapaneseLevel::ALL {
            let detail = LevelProgressDetail {
                kanji: CategoryProgress {
                    learned: *learned_kanji.get(&level).unwrap_or(&0),
                    total: *total_kanji.get(&level).unwrap_or(&0),
                },
                words: CategoryProgress {
                    learned: *learned_words.get(&level).unwrap_or(&0),
                    total: *total_words.get(&level).unwrap_or(&0),
                },
                grammar: CategoryProgress {
                    learned: *learned_grammar.get(&level).unwrap_or(&0),
                    total: *total_grammar.get(&level).unwrap_or(&0),
                },
            };
            self.levels.insert(level, detail);
        }
    }
}

impl Default for JlptProgress {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn category_progress_percentage_calculation() {
        let progress = CategoryProgress {
            learned: 50,
            total: 100,
        };
        assert!((progress.percentage() - 50.0).abs() < 0.001);

        let progress_zero_total = CategoryProgress {
            learned: 50,
            total: 0,
        };
        assert!((progress_zero_total.percentage() - 0.0).abs() < 0.001);

        let progress_full = CategoryProgress {
            learned: 100,
            total: 100,
        };
        assert!((progress_full.percentage() - 100.0).abs() < 0.001);
    }

    #[test]
    fn category_progress_is_complete_threshold() {
        let progress = CategoryProgress {
            learned: 90,
            total: 100,
        };
        assert!(progress.is_complete(90.0));
        assert!(!progress.is_complete(95.0));

        let progress_below = CategoryProgress {
            learned: 89,
            total: 100,
        };
        assert!(!progress_below.is_complete(90.0));
    }

    #[test]
    fn level_progress_detail_overall_percentage() {
        let detail = LevelProgressDetail {
            kanji: CategoryProgress {
                learned: 100,
                total: 100,
            },
            words: CategoryProgress {
                learned: 50,
                total: 100,
            },
            grammar: CategoryProgress {
                learned: 0,
                total: 100,
            },
        };
        assert!((detail.overall_percentage() - 50.0).abs() < 0.001);

        let detail_empty = LevelProgressDetail::new();
        assert!((detail_empty.overall_percentage() - 0.0).abs() < 0.001);
    }

    #[test]
    fn jlpt_progress_current_level_empty_returns_n5() {
        let progress = JlptProgress::new();
        assert_eq!(progress.current_level(), JapaneseLevel::N5);
    }

    #[test]
    fn jlpt_progress_current_level_n5_completed_returns_n4() {
        let mut progress = JlptProgress::new();
        let n5_complete = LevelProgressDetail {
            kanji: CategoryProgress {
                learned: 100,
                total: 100,
            },
            words: CategoryProgress {
                learned: 100,
                total: 100,
            },
            grammar: CategoryProgress {
                learned: 90,
                total: 100,
            },
        };
        progress.update_level(JapaneseLevel::N5, n5_complete);
        assert_eq!(progress.current_level(), JapaneseLevel::N4);
    }

    #[test]
    fn jlpt_progress_current_level_n5_n4_completed_returns_n3() {
        let mut progress = JlptProgress::new();

        let complete = LevelProgressDetail {
            kanji: CategoryProgress {
                learned: 100,
                total: 100,
            },
            words: CategoryProgress {
                learned: 100,
                total: 100,
            },
            grammar: CategoryProgress {
                learned: 100,
                total: 100,
            },
        };

        progress.update_level(JapaneseLevel::N5, complete.clone());
        progress.update_level(JapaneseLevel::N4, complete);

        assert_eq!(progress.current_level(), JapaneseLevel::N3);
    }

    #[test]
    fn jlpt_progress_current_level_n1_completed_stays_n1() {
        let mut progress = JlptProgress::new();

        let complete = LevelProgressDetail {
            kanji: CategoryProgress {
                learned: 100,
                total: 100,
            },
            words: CategoryProgress {
                learned: 100,
                total: 100,
            },
            grammar: CategoryProgress {
                learned: 100,
                total: 100,
            },
        };

        for level in JapaneseLevel::ALL {
            progress.update_level(level, complete.clone());
        }

        assert_eq!(progress.current_level(), JapaneseLevel::N1);
    }

    #[test]
    fn jlpt_progress_recalculate_updates_all_levels() {
        let mut progress = JlptProgress::new();

        let mut learned_kanji = HashMap::new();
        learned_kanji.insert(JapaneseLevel::N5, 50);
        learned_kanji.insert(JapaneseLevel::N4, 30);

        let mut learned_words = HashMap::new();
        learned_words.insert(JapaneseLevel::N5, 100);

        let mut learned_grammar = HashMap::new();
        learned_grammar.insert(JapaneseLevel::N5, 25);

        let mut total_kanji = HashMap::new();
        total_kanji.insert(JapaneseLevel::N5, 100);
        total_kanji.insert(JapaneseLevel::N4, 150);

        let mut total_words = HashMap::new();
        total_words.insert(JapaneseLevel::N5, 200);

        let mut total_grammar = HashMap::new();
        total_grammar.insert(JapaneseLevel::N5, 50);

        progress.recalculate(
            &learned_kanji,
            &learned_words,
            &learned_grammar,
            &total_kanji,
            &total_words,
            &total_grammar,
        );

        let n5_progress = progress.level_progress(JapaneseLevel::N5).unwrap();
        assert_eq!(n5_progress.kanji.learned, 50);
        assert_eq!(n5_progress.kanji.total, 100);
        assert_eq!(n5_progress.words.learned, 100);
        assert_eq!(n5_progress.words.total, 200);
        assert_eq!(n5_progress.grammar.learned, 25);
        assert_eq!(n5_progress.grammar.total, 50);

        let n4_progress = progress.level_progress(JapaneseLevel::N4).unwrap();
        assert_eq!(n4_progress.kanji.learned, 30);
        assert_eq!(n4_progress.kanji.total, 150);
        assert_eq!(n4_progress.words.learned, 0);
        assert_eq!(n4_progress.grammar.learned, 0);
    }
}
