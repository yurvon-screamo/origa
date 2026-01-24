use crate::components::*;
use crate::services::*;
use leptos::control_flow::Show;
use leptos::prelude::*;
use leptos::task::spawn_local;
use thaw::*;

#[component]
pub fn Import() -> impl IntoView {
    let import_source = RwSignal::new("anki".to_string());
    let importing = RwSignal::new(false);
    let import_result = RwSignal::new(None::<i32>);

    let handle_anki_import = move |_| {
        importing.set(true);
        spawn_local(async move {
            match import_anki_file("test.apkg".to_string()).await {
                Ok(count) => import_result.set(Some(count)),
                Err(err) => {
                    leptos::logging::log!("Import error: {}", err);
                    import_result.set(Some(0));
                }
            }
            importing.set(false);
        });
    };

    let handle_duolingo_import = move |_| {
        importing.set(true);
        spawn_local(async move {
            match import_duolingo_data().await {
                Ok(count) => import_result.set(Some(count)),
                Err(err) => {
                    leptos::logging::log!("Import error: {}", err);
                    import_result.set(Some(0));
                }
            }
            importing.set(false);
        });
    };

    view! {
        <MobileLayout>
            <div class="import-page">
                <Card>
                    <CardHeader>
                        <h2>"Импорт карточек"</h2>
                        <p>"Добавьте карточки из различных источников"</p>
                    </CardHeader>
                    <div class="import-body">
                        <div class="import-tabs">
                            <Button on_click=move |_| import_source.set("anki".to_string())>"Anki"</Button>
                            <Button on_click=move |_| import_source.set("duolingo".to_string())>"Duolingo"</Button>
                            <Button on_click=move |_| import_source.set("manual".to_string())>"Ручной ввод"</Button>
                        </div>

                        <div class="import-content" style="margin-top: 20px;">
                            {move || match import_source.get().as_str() {
                                "anki" => view! {
                                    <div class="anki-import">
                                        <div style="margin-bottom: 16px;">
                                            <h3>"Импорт из Anki"</h3>
                                            <p>"Выберите файл .apkg для импорта"</p>
                                        </div>

                                        <Button
                                            appearance=ButtonAppearance::Primary
                                            on_click=handle_anki_import
                                            disabled=importing.get()
                                        >
                                            {if importing.get() { "Импорт..." } else { "Импортировать из Anki" }}
                                        </Button>

                                        <div style="margin-top: 16px;">
                                            <p>"Поддерживаются колоды с форматом вопрос-ответ"</p>
                                        </div>
                                    </div>
                                        }.into_any(),

                                "duolingo" => view! {
                                    <div class="duolingo-import">
                                        <div style="margin-bottom: 16px;">
                                            <h3>"Синхронизация с Duolingo"</h3>
                                            <p>"Импортируйте слова из вашего профиля Duolingo"</p>
                                        </div>

                                        <Button
                                            appearance=ButtonAppearance::Primary
                                            on_click=handle_duolingo_import
                                            disabled=importing.get()
                                        >
                                            {if importing.get() { "Синхронизация..." } else { "Синхронизировать" }}
                                        </Button>

                                        <div style="margin-top: 16px;">
                                            <p>"Требуется подключение к интернету"</p>
                                        </div>
                                    </div>
                                        }.into_any(),

                                _ => view! {
                                    <div class="manual-import">
                                        <div style="margin-bottom: 16px;">
                                            <h3>"Ручной ввод"</h3>
                                            <p>"Введите слова в формате: слово|чтение|перевод"</p>
                                        </div>

                                        <p>"Функция временно недоступна"</p>

                                        <div style="margin-top: 16px;">
                                            <p>"Функция будет добавлена в будущих версиях"</p>
                                        </div>

                                        <Button
                                            appearance=ButtonAppearance::Primary
                                            disabled=true
                                        >
                                            "Не реализовано"
                                        </Button>

                                    </div>
                                        }.into_any()
                            }}

                            <Show when=move || import_result.get().is_some()>
                                <div style="margin-top: 20px;">
                                    {match import_result.get() {
                                        Some(0) => "Ошибка импорта".to_string(),
                                        Some(count) => format!("Успешно импортировано {} карточек", count),
                                        None => "".to_string(),
                                    }}
                                </div>
                            </Show>
                        </div>
                    </div>
                </Card>
            </div>
        </MobileLayout>
    }
}
