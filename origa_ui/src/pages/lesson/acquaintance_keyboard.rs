//! Клавиатура режима знакомства: те же хендлы, что в обычном уроке
//! (спека §8.3): Space = показать/дальше, [1]/[2] = оценка. Аудио-фронт
//! Reverse-подфазы: Space = повтор аудио, Enter = показать ответ.

use super::acquaintance_state::{AcquaintanceContext, AcquaintanceStage};
use leptos::ev::KeyboardEvent;
use leptos::prelude::*;

/// Действие, разрешённое клавиатурой на текущей стадии и состоянии слайда.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AcquaintanceKeyAction {
    /// Space в показе — следующий слайд («Дальше»).
    Advance,
    /// Space в тренировке до раскрытия — «Показать ответ»
    /// (Enter — на аудио-фронте).
    Reveal,
    /// Space на аудио-фронтe тренировки — повтор аудио.
    ReplayAudio,
    /// [1] после раскрытия — «Не помню».
    RateDontRemember,
    /// [2] после раскрытия — «Помню».
    RateRemember,
}

/// Чистая функция разрешения клавиши — покрывается host-тестами без
/// браузерного окружения. `audio_front` — текущий фронт аудио
/// (Reverse-подфаза, слово озвучивается вместо текста перевода).
pub fn resolve_key_action(
    stage: AcquaintanceStage,
    showing_answer: bool,
    audio_front: bool,
    key: &str,
) -> Option<AcquaintanceKeyAction> {
    match stage {
        AcquaintanceStage::Presentation => (key == " ").then_some(AcquaintanceKeyAction::Advance),
        AcquaintanceStage::Training => match (showing_answer, key) {
            (false, " ") if audio_front => Some(AcquaintanceKeyAction::ReplayAudio),
            (false, " ") => Some(AcquaintanceKeyAction::Reveal),
            (false, "Enter") if audio_front => Some(AcquaintanceKeyAction::Reveal),
            (true, "1") => Some(AcquaintanceKeyAction::RateDontRemember),
            (true, "2") => Some(AcquaintanceKeyAction::RateRemember),
            _ => None,
        },
        AcquaintanceStage::Completed => (key == " ").then_some(AcquaintanceKeyAction::Advance),
        AcquaintanceStage::Inactive => None,
    }
}

/// Колбэки, которые клавиатура дёргает вместо кнопок.
pub struct AcquaintanceKeyboardActions {
    pub on_advance: Box<dyn Fn()>,
    pub on_reveal: Box<dyn Fn()>,
    pub on_rate: Box<dyn Fn(bool)>,
    pub on_replay_audio: Box<dyn Fn()>,
}

/// Обработчик keydown: резолвит действие и исполняет колбэк.
/// Guard на поля ввода — на стороне слушателя (`is_typing_target`);
/// `is_audio_front` сообщает, озвучивается ли текущий фронт (Reverse).
pub fn create_acquaintance_keyboard_handler(
    ctx: AcquaintanceContext,
    showing_answer: RwSignal<bool>,
    actions: AcquaintanceKeyboardActions,
    is_audio_front: Box<dyn Fn() -> bool>,
) -> impl Fn(KeyboardEvent) {
    move |ev: KeyboardEvent| {
        // Автоповтор удержания игнорируем: иначе удержание Space на
        // последнем слайде показа «продавливает» Reveal уже в тренировке.
        if ev.repeat() {
            return;
        }
        let stage = ctx.state.get().stage;
        let Some(action) = resolve_key_action(
            stage,
            showing_answer.get_untracked(),
            is_audio_front(),
            &ev.key(),
        ) else {
            return;
        };
        ev.prevent_default();
        match action {
            AcquaintanceKeyAction::Advance => (actions.on_advance)(),
            AcquaintanceKeyAction::Reveal => (actions.on_reveal)(),
            AcquaintanceKeyAction::ReplayAudio => (actions.on_replay_audio)(),
            AcquaintanceKeyAction::RateDontRemember => (actions.on_rate)(false),
            AcquaintanceKeyAction::RateRemember => (actions.on_rate)(true),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn presentation_space_advances() {
        assert_eq!(
            resolve_key_action(AcquaintanceStage::Presentation, false, false, " "),
            Some(AcquaintanceKeyAction::Advance)
        );
    }

    #[test]
    fn presentation_digits_do_nothing() {
        assert_eq!(
            resolve_key_action(AcquaintanceStage::Presentation, false, false, "1"),
            None
        );
    }

    #[test]
    fn training_space_before_reveal_reveals() {
        assert_eq!(
            resolve_key_action(AcquaintanceStage::Training, false, false, " "),
            Some(AcquaintanceKeyAction::Reveal)
        );
    }

    #[test]
    fn training_space_on_audio_front_replays_audio() {
        assert_eq!(
            resolve_key_action(AcquaintanceStage::Training, false, true, " "),
            Some(AcquaintanceKeyAction::ReplayAudio)
        );
    }

    #[test]
    fn training_enter_on_audio_front_reveals() {
        assert_eq!(
            resolve_key_action(AcquaintanceStage::Training, false, true, "Enter"),
            Some(AcquaintanceKeyAction::Reveal)
        );
    }

    #[test]
    fn training_enter_on_text_front_does_nothing() {
        // Enter не входит в текстовый контракт (Space = показать): лишняя
        // клавиша не должна менять поведение текстового фронта.
        assert_eq!(
            resolve_key_action(AcquaintanceStage::Training, false, false, "Enter"),
            None
        );
    }

    #[test]
    fn training_after_reveal_one_is_dont_remember() {
        assert_eq!(
            resolve_key_action(AcquaintanceStage::Training, true, false, "1"),
            Some(AcquaintanceKeyAction::RateDontRemember)
        );
    }

    #[test]
    fn training_after_reveal_two_is_remember() {
        assert_eq!(
            resolve_key_action(AcquaintanceStage::Training, true, false, "2"),
            Some(AcquaintanceKeyAction::RateRemember)
        );
    }

    #[test]
    fn training_space_after_reveal_does_nothing() {
        // После раскрытия оценивание только [1]/[2]: Space не должен
        // случайно скрыть ответ или двинуть ротацию — ни на текстовом,
        // ни на аудио-фронте.
        assert_eq!(
            resolve_key_action(AcquaintanceStage::Training, true, false, " "),
            None
        );
        assert_eq!(
            resolve_key_action(AcquaintanceStage::Training, true, true, " "),
            None
        );
    }

    #[test]
    fn rating_keys_do_nothing_before_reveal() {
        assert_eq!(
            resolve_key_action(AcquaintanceStage::Training, false, false, "1"),
            None
        );
        assert_eq!(
            resolve_key_action(AcquaintanceStage::Training, false, false, "2"),
            None
        );
    }

    #[test]
    fn inactive_ignores_all_keys() {
        for key in [" ", "1", "2"] {
            assert_eq!(
                resolve_key_action(AcquaintanceStage::Inactive, false, false, key),
                None
            );
        }
    }
}
