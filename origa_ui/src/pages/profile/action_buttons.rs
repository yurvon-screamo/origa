use crate::i18n::*;
use crate::ui_components::{Button, ButtonVariant, Text, TextSize, TypographyVariant};
use leptos::ev::MouseEvent;
use leptos::prelude::*;

#[component]
pub fn ActionButtons(
    #[prop(optional, into)] test_id: Signal<String>,
    on_save: Callback<MouseEvent>,
    on_logout: Callback<MouseEvent>,
    on_delete_account: Callback<MouseEvent>,
    is_saving: Signal<bool>,
    is_deleting: Signal<bool>,
    is_logging_out: Signal<bool>,
) -> impl IntoView {
    let i18n = use_i18n();
    let show_delete_confirm = RwSignal::new(false);
    let test_id_val = move || {
        let val = test_id.get();
        if val.is_empty() { None } else { Some(val) }
    };

    view! {
        <div class="space-y-4" data-testid=test_id_val>
            <div class="flex flex-wrap gap-4">
                <Button
                    variant={ButtonVariant::Filled}
                    on_click={on_save}
                    disabled=is_saving
                    test_id="profile-save-btn"
                >
                    {move || if is_saving.get() { t!(i18n, common.saving).into_any() } else { t!(i18n, profile.save_changes).into_any() }}
                </Button>

                <Button
                    variant={ButtonVariant::Ghost}
                    on_click={on_logout}
                    disabled=is_logging_out
                    test_id="profile-logout-btn"
                >
                    {move || if is_logging_out.get() { t!(i18n, profile.logging_out).into_any() } else { t!(i18n, profile.logout).into_any() }}
                </Button>
            </div>

            <div class="pt-4 border-t border-[var(--border-light)]">
                <Text size={TextSize::Small} variant={TypographyVariant::Muted} class="block mb-3">
                    {t!(i18n, profile.danger_zone)}
                </Text>

                {move || if show_delete_confirm.get() {
                    view! {
                        <div class="p-4 bg-[var(--error)]/20 border border-[var(--error)]/30 rounded-lg">
                            <p class="text-[var(--error)] text-sm mb-3">
                                {t!(i18n, profile.delete_confirm_message)}
                            </p>
                            <div class="flex gap-3">
                                <Button
                                    variant={ButtonVariant::Filled}
                                    on_click={on_delete_account}
                                    disabled=is_deleting
                                    class="bg-[var(--error)] hover:bg-[var(--error)]"
                                    test_id="profile-confirm-delete-btn"
                                >
                                    {move || if is_deleting.get() { t!(i18n, common.deleting).into_any() } else { t!(i18n, common.confirm).into_any() }}
                                </Button>
                                <Button
                                    variant={ButtonVariant::Ghost}
                                    on_click={Callback::new(move |_| show_delete_confirm.set(false))}
                                    test_id="profile-cancel-delete-btn"
                                >
                                    {t!(i18n, common.cancel)}
                                </Button>
                            </div>
                        </div>
                    }.into_any()
                } else {
                    view! {
                        <Button
                            variant={ButtonVariant::Ghost}
                            on_click={Callback::new(move |_| show_delete_confirm.set(true))}
                            class="text-[var(--error)] hover:text-[var(--error)] hover:bg-[var(--error)]/20"
                            test_id="profile-delete-btn"
                        >
                            {t!(i18n, profile.delete_account)}
                        </Button>
                    }.into_any()
                }}
            </div>
        </div>
    }
}
