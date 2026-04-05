use crate::ui_components::{Button, ButtonVariant, Drawer, Text, TextSize};
use leptos::prelude::*;
use leptos_router::components::A;

#[component]
pub fn NavDrawer(
    #[prop(optional)] is_open: RwSignal<bool>,
    #[prop(optional, into)] test_id: Signal<String>,
) -> impl IntoView {
    let nav_test_id = super::derive_test_id(test_id, "nav");
    let title = Signal::derive(|| "Навигация".to_string());

    view! {
        <Drawer is_open=is_open title=title test_id=test_id>
            <div class="flex flex-col gap-2">
                <NavItem href="/lesson" label="Урок" japanese="📚" test_id=super::derive_test_id(nav_test_id, "lesson") />
                <NavItem href="/words" label="Слова" japanese="言葉" test_id=super::derive_test_id(nav_test_id, "words") />
                <NavItem href="/grammar" label="Грамматика" japanese="文法" test_id=super::derive_test_id(nav_test_id, "grammar") />
                <NavItem href="/kanji" label="Кандзи" japanese="漢字" test_id=super::derive_test_id(nav_test_id, "kanji") />
                <NavItem href="/profile" label="Профиль" japanese="👤" test_id=super::derive_test_id(nav_test_id, "profile") />
            </div>
        </Drawer>
    }
}

#[component]
fn NavItem(
    #[prop(into)] href: String,
    #[prop(into)] label: String,
    #[prop(into)] japanese: String,
    #[prop(optional, into)] test_id: Signal<String>,
) -> impl IntoView {
    view! {
        <A href=href.clone()>
            <Button variant=ButtonVariant::Ghost class=Signal::derive(|| "w-full text-left".to_string()) test_id=test_id>
                <div class="flex items-center gap-3 w-full">
                    <span class="text-lg">{japanese}</span>
                    <Text size=TextSize::Default>{label}</Text>
                </div>
            </Button>
        </A>
    }
}
