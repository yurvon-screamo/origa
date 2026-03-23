use leptos::prelude::StoredValue;
use leptos::prelude::*;

#[component]
pub fn RadicalDetailsView(
    stroke_count: u32,
    #[prop(into)] kanji_examples: StoredValue<Option<Vec<char>>>,
) -> impl IntoView {
    view! {
        <div class="space-y-4">
            <div class="text-center">
                <p class="text-sm text-gray-500">"Количество черт"</p>
                <p class="text-2xl font-bold">{stroke_count.to_string()}</p>
            </div>

            <Show when=move || kanji_examples.get_value().is_some()>
                <div class="text-center">
                    <p class="text-sm text-gray-500 mb-2">"Примеры кандзи"</p>
                    <div class="flex flex-wrap justify-center gap-2">
                        {
                            let examples = kanji_examples.get_value().unwrap();
                            examples
                                .into_iter()
                                .map(|kanji| {
                                    view! {
                                        <span class="px-3 py-1 bg-gray-100 rounded text-lg">
                                            {kanji}
                                        </span>
                                    }
                                })
                                .collect::<Vec<_>>()
                        }
                    </div>
                </div>
            </Show>
        </div>
    }
}
