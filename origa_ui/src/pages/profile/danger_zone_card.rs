use crate::i18n::*;
use crate::ui_components::{Button, ButtonVariant};
use leptos::ev::MouseEvent;
use leptos::prelude::*;

#[component]
pub fn DangerZoneCard(
    #[prop(optional, into)] test_id: Signal<String>,
    on_logout: Callback<MouseEvent>,
    on_delete_account: Callback<MouseEvent>,
    is_logging_out: Signal<bool>,
    is_deleting: Signal<bool>,
) -> impl IntoView {
    let i18n = use_i18n();
    let show_delete_confirm = RwSignal::new(false);
    let test_id_val = move || {
        let val = test_id.get();
        if val.is_empty() { None } else { Some(val) }
    };

    view! {
        <div class="danger-zone-card" data-testid=test_id_val>
            <div class="danger-zone-title">{t!(i18n, profile.danger_zone)}</div>

            <div class="flex flex-col gap-3">
                <Button
                    variant={Signal::derive(|| ButtonVariant::Default)}
                    on_click={on_logout}
                    disabled=is_logging_out
                    class=Signal::derive(|| "danger-zone-btn".to_string())
                    test_id="profile-logout-btn"
                >
                    {move || if is_logging_out.get() {
                        t!(i18n, profile.logging_out).into_any()
                    } else {
                        t!(i18n, profile.logout).into_any()
                    }}
                </Button>

                {move || if show_delete_confirm.get() {
                    view! {
                        <div class="border border-[var(--error)] p-4">
                            <p class="text-[var(--error)] text-sm mb-3">
                                {t!(i18n, profile.delete_confirm_message)}
                            </p>
                            <div class="flex gap-3">
                                <Button
                                    variant={Signal::derive(|| ButtonVariant::Filled)}
                                    on_click={on_delete_account}
                                    disabled=is_deleting
                                    class=Signal::derive(|| "bg-[var(--error)] hover:bg-[var(--error)]".to_string())
                                    test_id="profile-confirm-delete-btn"
                                >
                                    {move || if is_deleting.get() {
                                        t!(i18n, common.deleting).into_any()
                                    } else {
                                        t!(i18n, common.confirm).into_any()
                                    }}
                                </Button>
                                <Button
                                    variant={Signal::derive(|| ButtonVariant::Default)}
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
                            variant={Signal::derive(|| ButtonVariant::Default)}
                            on_click={Callback::new(move |_| show_delete_confirm.set(true))}
                            class=Signal::derive(|| "danger-zone-btn danger-zone-btn--delete".to_string())
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
