use leptos::*;
use leptos_router::*;
use thaw::*;
use crate::services::*;
use crate::components::*;

#[component]
pub fn Import() -> impl IntoView {
    let import_source = create_rw_signal("anki".to_string());
    let importing = create_rw_signal(false);
    let import_result = create_rw_signal(None::<i32>);

    let handle_anki_import = move |_| {
        importing.set(true);
        spawn_local(async move {
            match import_anki_file("test.apkg".to_string()).await {
                Ok(count) => import_result.set(Some(count)),
                Err(err) => {
                    logging::log!("Import error: {}", err);
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
                    logging::log!("Import error: {}", err);
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
                    <CardBody>
                        <Tabs value=import_source>
                            <Tab value="anki" label="Anki" />
                            <Tab value="duolingo" label="Duolingo" />
                            <Tab value="manual" label="Ручной ввод" />
                        </Tabs>

                        <div class="import-content" style="margin-top: 20px;">
                            {move || match import_source.get().as_str() {
                                "anki" => view! {
                                    <div class="anki-import">
                                        <div style="margin-bottom: 16px;">
                                            <h3>"Импорт из Anki"</h3>
                                            <p>"Выберите файл .apkg для импорта"</p>
                                        </div>
                                        
                                        <Upload
                                            accept=".apkg"
                                            multiple=false
                                            on_file_change=move |files| {
                                                if let Some(_file) = files.first() {
                                                    logging::log!("File selected for import");
                                                }
                                            }
                                        >
                                            <Button
                                                appearance=ButtonAppearance::Primary
                                                on_click=handle_anki_import
                                                loading=importing
                                            >
                                                "Выбрать и импортировать"
                                            </Button>
                                        </Upload>
                                        
                                        <div style="margin-top: 16px;">
                                            <p>"Поддерживаются колоды с форматом вопрос-ответ"</p>
                                        </div>
                                    </div>
                                }.into_view(),
                                
                                "duolingo" => view! {
                                    <div class="duolingo-import">
                                        <div style="margin-bottom: 16px;">
                                            <h3>"Синхронизация с Duolingo"</h3>
                                            <p>"Импортируйте слова из вашего профиля Duolingo"</p>
                                        </div>
                                        
                                        <Input
                                            placeholder="Имя пользователя Duolingo"
                                            style="margin-bottom: 16px;"
                                        />
                                        
                                        <Button
                                            appearance=ButtonAppearance::Primary
                                            on_click=handle_duolingo_import
                                            loading=importing
                                        >
                                            "Синхронизировать"
                                        </Button>
                                        
                                        <div style="margin-top: 16px;">
                                            <p>"Требуется подключение к интернету"</p>
                                        </div>
                                    </div>
                                }.into_view(),
                                
                                _ => view! {
                                    <div class="manual-import">
                                        <div style="margin-bottom: 16px;">
                                            <h3>"Ручной ввод"</h3>
                                            <p>"Введите слова в формате: слово|чтение|перевод"</p>
                                        </div>
                                        
                                        <TextArea
                                            placeholder="例文|れいぶん|Пример предложения&#10;日本語|にほんご|Японский язык"
                                            rows=10
                                            style="margin-bottom: 16px;"
                                        />
                                        
                                        <Button appearance=ButtonAppearance::Primary>
                                            "Импортировать"
                                        </Button>
                                        
                                        <div style="margin-top: 16px;">
                                            <p>"Каждая строка - новая карточка"</p>
                                        </div>
                                    </div>
                                }.into_view()
                            }}

                            <Show when=move || import_result.get().is_some()>
                                <div style="margin-top: 20px;">
                                    <Alert
                                        variant=match import_result.get() {
                                            Some(0) => AlertVariant::Error,
                                            Some(_) => AlertVariant::Success,
                                            None => AlertVariant::Info,
                                        }
                                    >
                                        {match import_result.get() {
                                            Some(0) => "Ошибка импорта".to_string(),
                                            Some(count) => format!("Успешно импортировано {} карточек", count),
                                            None => "".to_string(),
                                        }}
                                    </Alert>
                                </div>
                            </Show>
                        </div>
                    </CardBody>
                </Card>
            </div>
        </MobileLayout>
    }
}