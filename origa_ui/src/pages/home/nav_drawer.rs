use crate::i18n::*;
use crate::ui_components::{Button, ButtonVariant, Drawer, Text, TextSize, derive_test_id};
use leptos::prelude::*;
use leptos_router::components::A;

#[component]
pub fn NavDrawer(
    #[prop(optional)] is_open: RwSignal<bool>,
    #[prop(optional, into)] test_id: Signal<String>,
) -> impl IntoView {
    let i18n = use_i18n();
    let nav_test_id = derive_test_id(test_id, "nav");
    let title = Signal::derive(move || i18n.get_keys().home().navigation().inner().to_string());

    view! {
        <Drawer is_open=is_open title=title test_id=test_id>
            <div class="flex flex-col gap-2">
                <NavItem href="/lesson" label=i18n.get_keys().home().lesson().inner().to_string() japanese="📚" test_id=derive_test_id(nav_test_id, "lesson") />
                <NavItem href="/words" label=i18n.get_keys().home().words().inner().to_string() japanese="言葉" test_id=derive_test_id(nav_test_id, "words") />
                <NavItem href="/grammar" label=i18n.get_keys().home().grammar().inner().to_string() japanese="文法" test_id=derive_test_id(nav_test_id, "grammar") />
                <NavItem href="/kanji" label=i18n.get_keys().home().kanji().inner().to_string() japanese="漢字" test_id=derive_test_id(nav_test_id, "kanji") />
                <NavItem href="/profile" label=i18n.get_keys().home().profile().inner().to_string() japanese="👤" test_id=derive_test_id(nav_test_id, "profile") />
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
