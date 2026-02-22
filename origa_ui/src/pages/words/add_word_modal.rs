use crate::app::update_current_user;
use crate::repository::InMemoryUserRepository;
use crate::ui_components::{
    Button, ButtonVariant, Input, Modal, Text, TextSize, TypographyVariant,
};
use leptos::prelude::*;
use leptos::task::spawn_local;
use origa::application::CreateVocabularyCardUseCase;
use origa::domain::User;
use origa::infrastructure::LlmServiceInvoker;

#[component]
pub fn AddWordModal(is_open: RwSignal<bool>) -> impl IntoView {
    let new_word = RwSignal::new(String::new());
    let is_loading = RwSignal::new(false);
    let current_user =
        use_context::<RwSignal<Option<User>>>().expect("current_user context not provided");
    let repository =
        use_context::<InMemoryUserRepository>().expect("repository context not provided");
    let llm_service = use_context::<LlmServiceInvoker>().expect("llm_service context not provided");

    let on_add = StoredValue::new(Callback::new(move |_: leptos::ev::MouseEvent| {
        let word = new_word.get().trim().to_string();

        if word.is_empty() {
            return;
        }

        let user_id = current_user.with(|u| u.as_ref().map(|u| u.id())).unwrap();
        let repository_clone = repository.clone();
        let current_user_signal = current_user.clone();
        let is_loading_signal = is_loading.clone();
        let new_word_signal = new_word.clone();
        let is_open_signal = is_open.clone();
        let llm_service_clone = llm_service.clone();

        is_loading.set(true);

        spawn_local(async move {
            let use_case = CreateVocabularyCardUseCase::new(&repository_clone, &llm_service_clone);

            let _ = use_case.execute(user_id, word.clone()).await;

            is_loading_signal.set(false);
            update_current_user(repository_clone, current_user_signal);

            new_word_signal.set(String::new());
            is_open_signal.set(false);
        });
    }));

    let on_cancel = StoredValue::new(Callback::new(move |_: leptos::ev::MouseEvent| {
        is_open.set(false);
    }));

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
                <Text size=TextSize::Small variant=TypographyVariant::Muted>
                    "Перевод будет сгенерирован автоматически"
                </Text>
                <div class="flex gap-2 justify-end">
                    <Button
                        variant=ButtonVariant::Ghost
                        on_click=on_cancel.get_value()
                    >
                        "Отмена"
                    </Button>
                    <Button
                        variant=ButtonVariant::Olive
                        disabled=Signal::derive(move || is_loading.get())
                        on_click=on_add.get_value()
                    >
                        {move || if is_loading.get() { "Создание..." } else { "Добавить" }}
                    </Button>
                </div>
            </div>
        </Modal>
    }
}
