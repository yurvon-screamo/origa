use crate::i18n::{t, use_i18n};
use crate::repository::HybridUserRepository;
use crate::ui_components::{
    Alert, AlertType, Button, ButtonVariant, Drawer, Input, Text, TextSize, TypographyVariant,
};
use leptos::prelude::*;
use leptos::task::spawn_local;
use origa::use_cases::CreateVocabularyCardUseCase;

#[component]
pub fn AddWordModal() -> impl IntoView {
    let i18n = use_i18n();
    let new_word = RwSignal::new(String::new());
    let is_loading = RwSignal::new(false);
    let error_message = RwSignal::new(None::<String>);
    let repository =
        use_context::<HybridUserRepository>().expect("repository context not provided");
    let disposed = StoredValue::new(());

    let on_add = {
        let repository = repository.clone();

        Callback::new(move |_: leptos::ev::MouseEvent| {
            let word = new_word.get().trim().to_string();
            if word.is_empty() {
                return;
            }

            let repository_clone = repository.clone();
            let is_loading_signal = is_loading;
            let new_word_signal = new_word;
            let error_signal = error_message;
            let disposed = disposed;

            is_loading_signal.set(true);
            error_signal.set(None);

            spawn_local(async move {
                let use_case = CreateVocabularyCardUseCase::new(&repository_clone);

                match use_case.execute(word.clone()).await {
                    Ok(_) => {
                        if disposed.is_disposed() {
                            return;
                        }
                        is_loading_signal.set(false);
                        new_word_signal.set(String::new());
                    },
                    Err(e) => {
                        if disposed.is_disposed() {
                            return;
                        }
                        is_loading_signal.set(false);
                        error_signal.set(Some(e.to_string()));
                    },
                }
            });
        })
    };

    let on_cancel = Callback::new(move |_: leptos::ev::MouseEvent| {
        error_message.set(None);
    });

    view! {
        <Drawer
            title=Signal::derive(move || i18n.get_keys().words().add_word().inner().to_string())
        >
            <div class="space-y-4">
                <div>
                    <Text size=TextSize::Small variant=TypographyVariant::Muted class=Signal::derive(|| "mb-2".to_string())>
                        {t!(i18n, words.word_japanese)}
                    </Text>
                    <Input
                        value=new_word
                        placeholder=Signal::derive(|| "例えば".to_string())
                    />
                </div>
                <Text size=TextSize::Small variant=TypographyVariant::Muted>
                    {t!(i18n, words.translation_auto)}
                </Text>
                {move || {
                    error_message.get().map(move |msg| view! {
                        <Alert
                            alert_type=Signal::derive(|| AlertType::Error)
                            title=Signal::derive(move || i18n.get_keys().common().error().inner().to_string())
                            message=Signal::derive(move || msg.clone())
                        />
                    })
                }}
                <div class="flex gap-2 justify-end">
                    <Button
                        variant=ButtonVariant::Ghost
                        on_click=on_cancel
                    >
                        {t!(i18n, common.cancel)}
                    </Button>
                    <Button
                        variant=ButtonVariant::Olive
                        disabled=Signal::derive(move || is_loading.get())
                        on_click=on_add
                    >
                        {move || if is_loading.get() { t!(i18n, words.creating).into_any() } else { t!(i18n, words.add_word).into_any() }}
                    </Button>
                </div>
            </div>
        </Drawer>
    }
}
