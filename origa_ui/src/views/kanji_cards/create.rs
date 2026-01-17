use crate::components::button::{Button, ButtonVariant};
use crate::components::input::Input;
use crate::components::sheet::{
    Sheet, SheetContent, SheetFooter, SheetHeader, SheetSide, SheetTitle,
};
use crate::{DEFAULT_USERNAME, ensure_user, to_error};
use dioxus::prelude::*;
use origa::application::CreateKanjiCardUseCase;
use origa::settings::ApplicationEnvironment;

#[component]
pub fn KanjiCreateModal(
    on_close: EventHandler<()>,
    on_success: EventHandler<String>,
    on_error: EventHandler<String>,
    loading: bool,
) -> Element {
    let kanji_input = use_signal(String::new);
    let mut error_msg = use_signal(String::new);

    rsx! {
                Sheet {
            open: true,
            on_open_change: move |v: bool| {
                if !v {
                    on_close.call(());
                    error_msg.set(String::new());
                }
            },
            SheetContent { side: SheetSide::Right,
                SheetHeader {
                    SheetTitle { "Создать карточки кандзи" }
                }

                div { class: "space-y-4",
                    div { class: "space-y-2",
                        label { class: "text-sm font-medium", "Кандзи (через запятую или пробел)" }
                        Input {
                            placeholder: "例: 語, 学, 日本",
                            value: kanji_input(),
                            oninput: {
                                let mut kanji_input = kanji_input;
                                move |e: FormEvent| {
                                    kanji_input.set(e.value());
                                    error_msg.set(String::new());
                                }
                            },
                        }
                        if !error_msg().is_empty() {
                            div { class: "text-sm text-red-500", "{error_msg()}" }
                        }
                        div { class: "text-xs text-slate-500",
                            "Введите японские символы (кандзи) через запятую или пробел"
                        }
                    }
                }

                SheetFooter {
                    Button {
                        variant: ButtonVariant::Outline,
                        onclick: move |_| on_close.call(()),
                        "Отмена"
                    }
                    Button {
                        variant: ButtonVariant::Primary,
                        disabled: loading,
                        onclick: move |_| {
                            let input = kanji_input();
                            if input.trim().is_empty() {
                                error_msg.set("Введите хотя бы один символ".to_string());
                                return;
                            }

                            if !is_valid_kanji_input(&input) {
                                error_msg.set("Введите только японские символы (кандзи)".to_string());
                                return;
                            }

                            let kanjies: Vec<String> = input
                                .split(|c: char| c == ',' || c.is_whitespace())
                                .filter(|s| !s.is_empty())
                                .map(|s| s.trim().to_string())
                                .collect();

                            if kanjies.is_empty() {
                                error_msg.set("Введите хотя бы один символ".to_string());
                                return;
                            }

                            let on_success = on_success;
                            let on_error = on_error;
                            let count = kanjies.len();
                            spawn(async move {
                                match create_kanji_cards(kanjies).await {
                                    Ok(_) => {
                                        on_success.call(format!("Создано {} карточек", count));
                                    }
                                    Err(e) => {
                                        on_error.call(format!("Ошибка: {}", e));
                                    }
                                }
                            });
                        },
                        {if loading { "Создание..." } else { "Создать" }}
                    }
                }
            }
        }
    }
}

fn is_valid_kanji_input(s: &str) -> bool {
    let trimmed: String = s
        .chars()
        .filter(|c| !c.is_ascii_whitespace() && *c != ',')
        .collect();

    if trimmed.is_empty() {
        return true;
    }

    trimmed.chars().all(|c| is_japanese_char(c))
}

fn is_japanese_char(c: char) -> bool {
    (c >= '\u{4E00}' && c <= '\u{9FFF}') || // CJK Unified Ideographs
    (c >= '\u{3400}' && c <= '\u{4DBF}') || // CJK Unified Ideographs Extension A
    (c >= '\u{4E00}' && c <= '\u{9FFF}') || // CJK Unified Ideographs
    (c >= '\u{20000}' && c <= '\u{2A6DF}') || // CJK Unified Ideographs Extension B
    (c >= '\u{2A700}' && c <= '\u{2B73F}') || // CJK Unified Ideographs Extension C
    (c >= '\u{2B740}' && c <= '\u{2B81F}') || // CJK Unified Ideographs Extension D
    (c >= '\u{2B820}' && c <= '\u{2CEAF}') || // CJK Unified Ideographs Extension E
    (c >= '\u{2CEB0}' && c <= '\u{2EBEF}') || // CJK Unified Ideographs Extension F
    (c >= '\u{30000}' && c <= '\u{3134F}') || // CJK Unified Ideographs Extension G
    (c >= '\u{F900}' && c <= '\u{FAFF}') // CJK Compatibility Ideographs
}

async fn create_kanji_cards(kanjies: Vec<String>) -> Result<(), String> {
    let env = ApplicationEnvironment::get();
    let repo = env.get_repository().await.map_err(to_error)?;
    let user_id = ensure_user(env, DEFAULT_USERNAME).await?;

    CreateKanjiCardUseCase::new(repo)
        .execute(user_id, kanjies)
        .await
        .map_err(to_error)?;

    Ok(())
}
