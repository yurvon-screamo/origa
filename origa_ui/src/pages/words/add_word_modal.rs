use crate::ui_components::{
    Button, ButtonVariant, Input, Modal, Text, TextSize, TypographyVariant,
};
use leptos::prelude::*;
use origa::domain::{Answer, Card, Question, User, VocabularyCard};

#[component]
pub fn AddWordModal(is_open: RwSignal<bool>) -> impl IntoView {
    let new_word = RwSignal::new(String::new());
    let new_meaning = RwSignal::new(String::new());
    let current_user =
        use_context::<RwSignal<Option<User>>>().expect("current_user context not provided");

    let add_word = move || {
        let word = new_word.get().trim().to_string();
        let meaning = new_meaning.get().trim().to_string();

        if word.is_empty() || meaning.is_empty() {
            return;
        }

        if let Ok(question) = Question::new(word) {
            if let Ok(answer) = Answer::new(meaning) {
                let vocab_card = VocabularyCard::new(question, answer);
                let card = Card::Vocabulary(vocab_card);

                if let Some(mut user) = current_user.get() {
                    let _ = user.create_card(card);
                    current_user.set(Some(user));
                }
            }
        }

        new_word.set(String::new());
        new_meaning.set(String::new());
        is_open.set(false);
    };

    view! {
        <Modal
            is_open=is_open
            title=Signal::derive(|| "Добавить слово".to_string())
        >
            <div class="space-y-4">
                <div>
                    <Text size=TextSize::Small variant=TypographyVariant::Muted class=Signal::derive(|| "mb-2".to_string())>
                        "Слово (японский)"
                    </Text>
                    <Input
                        value=new_word
                        placeholder=Signal::derive(|| "例えば".to_string())
                    />
                </div>
                <div>
                    <Text size=TextSize::Small variant=TypographyVariant::Muted class=Signal::derive(|| "mb-2".to_string())>
                        "Перевод"
                    </Text>
                    <Input
                        value=new_meaning
                        placeholder=Signal::derive(|| "Например".to_string())
                    />
                </div>
                <div class="flex gap-2 justify-end">
                    <Button
                        variant=ButtonVariant::Ghost
                        on_click=Callback::new(move |_: leptos::ev::MouseEvent| {
                            is_open.set(false);
                        })
                    >
                        "Отмена"
                    </Button>
                    <Button
                        variant=ButtonVariant::Olive
                        on_click=Callback::new(move |_: leptos::ev::MouseEvent| {
                            add_word();
                        })
                    >
                        "Добавить"
                    </Button>
                </div>
            </div>
        </Modal>
    }
}
