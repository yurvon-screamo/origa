use leptos::prelude::*;
use leptos_meta::*;
use leptos_router::{components::*, path};

use crate::components::{Layout, NotFound};
use crate::content::Locale;
use crate::pages::*;

pub fn shell(_options: LeptosOptions) -> impl IntoView {
    view! {
        <!DOCTYPE html>
        <html lang="en">
            <head>
                <meta charset="utf-8" />
                <meta name="viewport" content="width=device-width, initial-scale=1.0" />
                <meta name="theme-color" content="#3d4535" />
                <link rel="icon" type="image/png" href="/favicon.png" />
                <link rel="preconnect" href="https://fonts.googleapis.com" />
                <link rel="preconnect" href="https://fonts.gstatic.com" crossorigin />
                <link rel="stylesheet" href="https://fonts.googleapis.com/css2?family=Cormorant+Garamond:ital,wght@0,300;0,400;0,500;0,600;0,700;1,400&family=DM+Mono:ital,wght@0,300;0,400;0,500;1,400&display=swap" />
                <MetaTags />
            </head>
            <body class="min-h-screen paper-texture">
                <App />
            </body>
        </html>
    }
}

#[component]
pub fn App() -> impl IntoView {
    provide_meta_context();

    view! {
        <Stylesheet id="leptos" href="/landing.processed.css" />
        <Title text="Origa" />
        <Router>
            <Routes fallback=NotFound>
                <ParentRoute path=path!("") view=move || view! { <Layout locale=Locale::En /> }>
                    <Route path=path!("") view=HomePage />
                    <Route path=path!("features") view=FeaturesPage />
                    <Route path=path!("compare") view=ComparePage />
                    <Route path=path!("integrations") view=IntegrationsPage />
                    <Route path=path!("download") view=DownloadPage />
                </ParentRoute>
                <ParentRoute path=path!("ru") view=move || view! { <Layout locale=Locale::Ru /> }>
                    <Route path=path!("") view=HomePage />
                    <Route path=path!("features") view=FeaturesPage />
                    <Route path=path!("compare") view=ComparePage />
                    <Route path=path!("integrations") view=IntegrationsPage />
                    <Route path=path!("download") view=DownloadPage />
                </ParentRoute>
            </Routes>
        </Router>
    }
}
