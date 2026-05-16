use super::dashboard_stats::RecentlyStudiedItem;
use crate::i18n::{t, use_i18n};
use crate::ui_components::{Card, Text, TextSize, TypographyVariant};
use leptos::prelude::*;

#[component]
pub fn RecentStudyList(
    items: Signal<Vec<RecentlyStudiedItem>>,
    #[prop(optional, into)] test_id: Signal<String>,
) -> impl IntoView {
    let i18n = use_i18n();

    let test_id_val = move || {
        let val = test_id.get();
        if val.is_empty() { None } else { Some(val) }
    };

    let is_empty = Signal::derive(move || items.get().is_empty());

    view! {
        <div data-testid=test_id_val>
            <Text
                size=TextSize::Small
                variant=TypographyVariant::Muted
                uppercase=true
                tracking_widest=true
            >
                {t!(i18n, home.recent_studied)}
            </Text>

            <Show when=move || is_empty.get()>
                <div class="mt-3">
                    <Text size=TextSize::Small variant=TypographyVariant::Muted>
                        {t!(i18n, home.no_recent_studied)}
                    </Text>
                </div>
            </Show>

            <Show when=move || !is_empty.get()>
                <div class="grid grid-cols-2 sm:grid-cols-3 lg:grid-cols-5 gap-4 mt-3">
                    <For
                        each=move || items.get()
                        key=|item| item.card_id.clone()
                        children=move |item| {
                            view! {
                                <Card
                                    class=Signal::derive(|| "p-4".to_string())
                                    test_id=Signal::derive(move || format!("recent-{}", item.card_id))
                                >
                                    <div class="font-serif text-[24px] text-[var(--fg-black)] leading-tight">
                                        {item.japanese}
                                    </div>
                                    <div class="font-mono text-[12px] text-[var(--fg-muted)] mt-1">
                                        {item.reading}
                                    </div>
                                    <div class="font-mono text-[11px] text-[var(--fg-light)] mt-2 line-clamp-2">
                                        {item.meaning}
                                    </div>
                                </Card>
                            }
                        }
                    />
                </div>
            </Show>
        </div>
    }
}
