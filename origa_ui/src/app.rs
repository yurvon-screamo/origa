use leptos::*;
use leptos_router::components::{Route, Router, Routes};
use leptos_router::path;
use thaw::*;

use crate::views::*;

#[component]
pub fn app() -> impl IntoView {
    view! {
        <ConfigProvider>
            <Router>
                <Routes fallback=|| "Not found.">
                    <Route path=path!("/") view=Overview />
                    <Route path=path!("/learn") view=Learn />
                    <Route path=path!("/import") view=Import />
                    <Route path=path!("/vocabulary") view=Vocabulary />
                    <Route path=path!("/kanji") view=Kanji />
                    <Route path=path!("/grammar-reference") view=GrammarReference />
                    <Route path=path!("/grammar") view=Grammar />
                    <Route path=path!("/overview") view=Overview />
                    <Route path=path!("/profile") view=Profile />
                </Routes>
            </Router>
        </ConfigProvider>
    }
}
