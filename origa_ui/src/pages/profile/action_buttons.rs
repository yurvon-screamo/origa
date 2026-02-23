use crate::ui_components::{Button, ButtonVariant, Text, TextSize, TypographyVariant};
use leptos::ev::MouseEvent;
use leptos::prelude::*;

#[component]
pub fn ActionButtons(
    on_save: Callback<MouseEvent>,
    on_logout: Callback<MouseEvent>,
    on_delete_account: Callback<MouseEvent>,
    is_saving: Signal<bool>,
    is_deleting: Signal<bool>,
) -> impl IntoView {
    let show_delete_confirm = RwSignal::new(false);

    view! {
        <div class="space-y-4">
            <div class="flex flex-wrap gap-4">
                <Button
                    variant={ButtonVariant::Filled}
                    on_click={on_save}
                    disabled=is_saving
                >
                    {move || if is_saving.get() { "Сохранение..." } else { "Сохранить изменения" }}
                </Button>

                <Button
                    variant={ButtonVariant::Ghost}
                    on_click={on_logout}
                >
                    "Выйти из аккаунта"
                </Button>
            </div>

            <div class="pt-4 border-t border-[var(--border-subtle)]">
                <Text size={TextSize::Small} variant={TypographyVariant::Muted} class="block mb-3">
                    "Опасная зона"
                </Text>

                {move || if show_delete_confirm.get() {
                    view! {
                        <div class="p-4 bg-red-950/20 border border-red-900/30 rounded-lg">
                            <p class="text-red-400 text-sm mb-3">
                                "Вы уверены? Это действие удалит ваш аккаунт и все данные безвозвратно."
                            </p>
                            <div class="flex gap-3">
                                <Button
                                    variant={ButtonVariant::Filled}
                                    on_click={on_delete_account}
                                    disabled=is_deleting
                                    class="bg-red-600 hover:bg-red-700"
                                >
                                    {move || if is_deleting.get() { "Удаление..." } else { "Да, удалить аккаунт" }}
                                </Button>
                                <Button
                                    variant={ButtonVariant::Ghost}
                                    on_click={Callback::new(move |_| show_delete_confirm.set(false))}
                                >
                                    "Отмена"
                                </Button>
                            </div>
                        </div>
                    }.into_any()
                } else {
                    view! {
                        <Button
                            variant={ButtonVariant::Ghost}
                            on_click={Callback::new(move |_| show_delete_confirm.set(true))}
                            class="text-red-400 hover:text-red-300 hover:bg-red-950/20"
                        >
                            "Удалить аккаунт"
                        </Button>
                    }.into_any()
                }}
            </div>
        </div>
    }
}
