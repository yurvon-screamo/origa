use leptos::prelude::*;
use crate::services::kanji_service::KanjiService;

#[derive(Clone)]
pub struct AppServices {
    pub kanji_service: KanjiService,
}

impl AppServices {
    pub fn new(kanji_service: KanjiService) -> Self {
        Self { kanji_service }
    }
}

// Provide context for services
#[component]
pub fn ServicesProvider(
    services: AppServices,
    children: Children,
) -> impl IntoView {
    provide_context(services.kanji_service);
    children()
}