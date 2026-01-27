use crate::services::kanji_service::KanjiService;
use leptos::prelude::*;

#[derive(Clone)]
pub struct AppServices {
    pub kanji_service: KanjiService,
}

impl AppServices {
    pub fn new() -> Self {
        Self {
            kanji_service: KanjiService::new(),
        }
    }
}

// Provide context for services
#[component]
pub fn ServicesProvider(services: AppServices, children: Children) -> impl IntoView {
    provide_context(services.kanji_service);
    children()
}
