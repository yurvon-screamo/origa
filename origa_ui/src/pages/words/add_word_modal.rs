use crate::app::update_current_user;
use crate::repository::HybridUserRepository;
use crate::ui_components::{
    Alert, AlertType, Button, ButtonVariant, Input, Modal, Text, TextSize, TypographyVariant,
};
use leptos::prelude::*;
use leptos::task::spawn_local;
use origa::domain::User;
use origa::use_cases::CreateVocabularyCardUseCase;

#[component]
pub fn AddWordModal() -> impl IntoView {
    let new_word = RwSignal::new(String::new());
    let is_loading = RwSignal::new(false);
    let error_message = RwSignal::new(None::<String>);
    let current_user =
        use_context::<RwSignal<Option<User>>>().expect("current_user context not provided");
    let repository =
        use_context::<HybridUserRepository>().expect("repository context not provided");

    let on_add = {
        let current_user = current_user;
        let repository = repository.clone();

        Callback::new(move |_: leptos::ev::MouseEvent| {
            let word = new_word.get().trim().to_string();
            if word.is_empty() {
                return;
            }

            let user_id = current_user.with(|u| u.as_ref().map(|u| u.id())).unwrap();
            let repository_clone = repository.clone();
            let current_user_signal = current_user;
            let is_loading_signal = is_loading;
            let new_word_signal = new_word;
            let error_signal = error_message;

            is_loading_signal.set(true);
            error_signal.set(None);

            spawn_local(async move {
                let use_case = CreateVocabularyCardUseCase::new(&repository_clone);

                match use_case.execute(user_id, word.clone()).await {
                    Ok(_) => {
                        is_loading_signal.set(false);
                        update_current_user(repository_clone, current_user_signal);
                        new_word_signal.set(String::new());
                    }
                    Err(e) => {
                        is_loading_signal.set(false);
                        error_signal.set(Some(e.to_string()));
                    }
                }
            });
        })
    };

    let on_cancel = Callback::new(move |_: leptos::ev::MouseEvent| {
        error_message.set(None);
    });

    view! {
        <Modal
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
                <Text size=TextSize::Small variant=TypographyVariant::Muted>
                    "Перевод будет сгенерирован автоматически"
                </Text>
                {move || {
                    error_message.get().map(|msg| view! {
                        <Alert
                            alert_type=Signal::derive(|| AlertType::Error)
                            title=Signal::derive(|| "Ошибка".to_string())
                            message=Signal::derive(move || msg.clone())
                        />
                    })
                }}
                <div class="flex gap-2 justify-end">
                    <Button
                        variant=ButtonVariant::Ghost
                        on_click=on_cancel
                    >
                        "Отмена"
                    </Button>
                    <Button
                        variant=ButtonVariant::Olive
                        disabled=Signal::derive(move || is_loading.get())
                        on_click=on_add
                    >
                        {move || if is_loading.get() { "Создание..." } else { "Добавить" }}
                    </Button>
                </div>
            </div>
        </Modal>
    }
}
