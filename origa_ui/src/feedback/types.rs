//! Data model of a user feedback report.
//!
//! A report is assembled at the moment of frustration: the entry point
//! (translator token popup, lesson header) captures the *subject* — the
//! exact content the user is looking at — and the modal adds the user's
//! message. Auto-context (version, platform, page, UI language) is snapshotted
//! on `open()` so it matches what the user was seeing, never a stale value.

use crate::core::version::VERSION;

/// What kind of problem the report describes. Derived from the entry point,
/// never chosen by the user — the entry point already knows.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FeedbackCategory {
    /// Wrong translation/reading/gloss of a token in the translator popup.
    Translation,
    /// Wrong card data (typo, audio mismatch, wrong reading) on a lesson card.
    Content,
}

impl FeedbackCategory {
    /// Stable tag value sent to Sentry (filterable in the feedback stream).
    pub fn as_str(self) -> &'static str {
        match self {
            FeedbackCategory::Translation => "translation",
            FeedbackCategory::Content => "content",
        }
    }
}

/// Which UI surface opened the feedback form.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FeedbackSource {
    /// The token popup of the translator component (`TranslatorText`).
    TranslatorPopup,
    /// The alert-triangle button in the lesson header.
    LessonHeader,
}

impl FeedbackSource {
    /// Stable tag value sent to Sentry (filterable in the feedback stream).
    pub fn as_str(self) -> &'static str {
        match self {
            FeedbackSource::TranslatorPopup => "translator_popup",
            FeedbackSource::LessonHeader => "lesson_header",
        }
    }
}

/// The content the user is reporting — captured verbatim from the screen.
#[derive(Clone, Debug, Default)]
pub struct FeedbackSubject {
    /// The Japanese surface the user sees (token text, card front, …).
    pub surface: String,
    /// Its reading, when the surface shows one.
    pub reading: Option<String>,
    /// The line the subject appeared in (phrase, sentence), if applicable.
    pub context_line: Option<String>,
}

/// Runtime environment snapshot appended to every report.
#[derive(Clone, Debug)]
pub struct FeedbackEnvironment {
    pub app_version: &'static str,
    pub platform: String,
    pub page: String,
    pub ui_language: String,
}

impl FeedbackEnvironment {
    /// Snapshot the current environment. Called on modal open, not on submit —
    /// the report must describe the moment the user decided to report.
    ///
    /// `ui_language` is passed by the caller (an entry point inside a
    /// component scope): reading the i18n context here would panic when
    /// `open()` is driven from outside a component (tests).
    pub fn capture(ui_language: &str) -> Self {
        Self {
            app_version: VERSION,
            platform: crate::core::platform::platform_name(),
            page: crate::feedback::current_page_path(),
            ui_language: ui_language.to_string(),
        }
    }
}

/// A complete report ready for submission.
#[derive(Clone, Debug)]
pub struct FeedbackReport {
    pub category: FeedbackCategory,
    pub source: FeedbackSource,
    pub subject: FeedbackSubject,
    /// The user's free-form description (trimmed, non-empty, <= 2000 chars).
    pub message: String,
    pub environment: FeedbackEnvironment,
}

/// Hard limit on the user message length (anti-spam, feedback API quota).
pub const FEEDBACK_MESSAGE_MAX_CHARS: usize = 2000;

impl FeedbackReport {
    /// Build the display message: user text first, subject lines appended so
    /// the report is self-contained in the Sentry feedback stream (where the
    /// structured tags/extra live alongside, not instead of, the message).
    pub fn compose_message(&self) -> String {
        let mut out = self.message.clone();
        if !self.subject.surface.is_empty() {
            out.push_str("\n\n— subject —\n");
            out.push_str(&self.subject.surface);
            if let Some(reading) = self.subject.reading.as_deref()
                && !reading.is_empty()
            {
                out.push_str(" [");
                out.push_str(reading);
                out.push(']');
            }
            if let Some(line) = self.subject.context_line.as_deref()
                && !line.is_empty()
            {
                out.push_str("\ncontext: ");
                out.push_str(line);
            }
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn subject() -> FeedbackSubject {
        FeedbackSubject {
            surface: "食べる".to_string(),
            reading: Some("たべる".to_string()),
            context_line: Some("毎日おいしいものを食べる".to_string()),
        }
    }

    #[test]
    fn compose_message_appends_subject_reading_and_context() {
        let report = FeedbackReport {
            category: FeedbackCategory::Translation,
            source: FeedbackSource::TranslatorPopup,
            subject: subject(),
            message: "wrong gloss".to_string(),
            environment: FeedbackEnvironment {
                app_version: "0.7.0",
                platform: "android".to_string(),
                page: "/lesson".to_string(),
                ui_language: "ru".to_string(),
            },
        };

        let composed = report.compose_message();

        assert!(composed.starts_with("wrong gloss"));
        assert!(composed.contains("食べる [たべる]"));
        assert!(composed.contains("context: 毎日おいしいものを食べる"));
    }

    #[test]
    fn compose_message_omits_empty_subject_parts() {
        let report = FeedbackReport {
            category: FeedbackCategory::Content,
            source: FeedbackSource::LessonHeader,
            subject: FeedbackSubject {
                surface: String::new(),
                reading: None,
                context_line: None,
            },
            message: "typo on card".to_string(),
            environment: FeedbackEnvironment {
                app_version: "0.7.0",
                platform: "web".to_string(),
                page: "/lesson".to_string(),
                ui_language: "en".to_string(),
            },
        };

        assert_eq!(report.compose_message(), "typo on card");
    }

    #[test]
    fn category_tags_are_stable() {
        assert_eq!(FeedbackCategory::Translation.as_str(), "translation");
        assert_eq!(FeedbackCategory::Content.as_str(), "content");
    }

    #[test]
    fn source_tags_are_stable() {
        assert_eq!(FeedbackSource::TranslatorPopup.as_str(), "translator_popup");
        assert_eq!(FeedbackSource::LessonHeader.as_str(), "lesson_header");
    }
}
