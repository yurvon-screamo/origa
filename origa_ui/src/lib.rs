use leptos::prelude::*;
use leptos_meta::*;
use leptos_router::{components::*, path};

// Modules
mod components;
mod pages;
mod services;

// Top-Level pages
use crate::pages::dashboard::Dashboard;
use crate::pages::grammar::Grammar;
use crate::pages::kanji::Kanji;
use crate::pages::profile::Profile;
use crate::pages::study::StudySession;
use crate::pages::vocabulary::Vocabulary;
use crate::services::app_services::{AppServices, ServicesProvider};

/// An app router which renders the homepage and handles 404's
#[component]
pub fn App() -> impl IntoView {
    // Provides context that manages stylesheets, titles, meta tags, etc.
    provide_meta_context();

    view! {
        <Html attr:lang="ru" attr:dir="ltr" attr:data-theme="light" />

        // sets the document title
        <Title text="オリガ" />

        // injects metadata in the <head> of the page
        <Meta charset="UTF-8" />
        <Meta name="viewport" content="width=device-width, initial-scale=1.0" />
        <Meta
            name="description"
            content="Приложение для изучения японского языка с карточками и spaced repetition"
        />
        <Meta name="theme-color" content="#4a6fa5" />

        <ServicesProvider services=AppServices::new()>
            <Router>
                <Routes fallback=|| view! { NotFound }>
                    <Route path=path!("/") view=Dashboard />
                    <Route path=path!("/dashboard") view=Dashboard />
                    <Route path=path!("/vocabulary") view=Vocabulary />
                    <Route path=path!("/kanji") view=Kanji />
                    <Route path=path!("/grammar") view=Grammar />
                    <Route path=path!("/study") view=StudySession />
                    <Route path=path!("/profile") view=Profile />
                </Routes>
            </Router>
        </ServicesProvider>
    }
}
