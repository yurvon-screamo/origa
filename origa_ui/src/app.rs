use leptos::*;
use leptos_router::*;
use thaw::*;

use crate::components::layout::*;
use crate::views::*;

#[component]
pub fn app() -> impl IntoView {
    view! {
        <ConfigProvider>
            <Router>
                <Routes>
                    <Route path="/" view=Overview />
                    <Route path="/learn" view=Learn />
                    <Route path="/import" view=Import />
                    <Route path="/vocabulary" view=Vocabulary />
                    <Route path="/kanji" view=Kanji />
                    <Route path="/grammar-reference" view=GrammarReference />
                    <Route path="/grammar" view=Grammar />
                    <Route path="/overview" view=Overview />
                    <Route path="/profile" view=Profile />
                </Routes>
            </Router>
        </ConfigProvider>
    }
}
