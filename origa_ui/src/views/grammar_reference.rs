use leptos::*;
use leptos_router::*;
use thaw::*;
use crate::services::*;
use crate::components::*;

#[component]
pub fn GrammarReference() -> impl IntoView {
    view! {
        <MobileLayout>
            <div class="grammar-reference-page">
                <Card>
                    <CardHeader>
                        <h2>"Справочник по грамматике"</h2>
                        <p>"Основные правила японской грамматики"</p>
                    </CardHeader>
                    <CardBody>
                        <Accordion>
                            <AccordionItem value="particles" title="Частицы">
                                <div style="padding: 16px;">
                                    <h4>"は (wa)"</h4>
                                    <p>"Указывает на тему предложения"</p>
                                    <p><em>"私は学生です"</em> " - "Я студент"</p>
                                    
                                    <h4 style="margin-top: 16px;">"が (ga)"</h4>
                                    <p>"Указывает на подлежащее"</p>
                                    <p><em>"猫が好きです"</em> " - "Нравятся кошки"</p>
                                </div>
                            </AccordionItem>
                            
                            <AccordionItem value="verb-forms" title="Формы глаголов">
                                <div style="padding: 16px;">
                                    <h4>"Простая форма (辞書形)"</h4>
                                    <p>"Форма глагола в словаре"</p>
                                    
                                    <h4 style="margin-top: 16px;">"Вежливая форма (ます形)"</h4>
                                    <p>"Используется в формальной речи"</p>
                                    
                                    <h4 style="margin-top: 16px;">"Прошедшее время (た形)"</h4>
                                    <p>"Описывает завершенные действия"</p>
                                </div>
                            </AccordionItem>
                            
                            <AccordionItem value="adjectives" title="Прилагательные">
                                <div style="padding: 16px;">
                                    <h4>"И-прилагательные"</h4>
                                    <p>"Заканчиваются на い"</p>
                                    <p><em>"高い"</em> " - высокий"</p>
                                    
                                    <h4 style="margin-top: 16px;">"На-прилагательные"</h4>
                                    <p>"Заканчиваются на な"</p>
                                    <p><em>"静か"</em> " - тихий"</p>
                                </div>
                            </AccordionItem>
                            
                            <AccordionItem value="counters" title="Счетные слова">
                                <div style="padding: 16px;">
                                    <h4>"Люди: 人 (にん)"</h4>
                                    <p>"一人、二人、三人..."</p>
                                    
                                    <h4 style="margin-top: 16px;">"Предметы: 個 (こ)"</h4>
                                    <p>"一個、二個、三個..."</p>
                                    
                                    <h4 style="margin-top: 16px;">"Место buildings: 棟 (とう)"</h4>
                                    <p>"一棟、二棟、三棟..."</p>
                                </div>
                            </AccordionItem>
                            
                            <AccordionItem value="time" title="Время и периоды">
                                <div style="padding: 16px;">
                                    <h4>"Дни недели"</h4>
                                    <p>"月曜日、火曜日、水曜日..."</p>
                                    
                                    <h4 style="margin-top: 16px;">"Время суток"</h4>
                                    <p>"朝、昼、夜"</p>
                                    
                                    <h4 style="margin-top: 16px;">"Периоды времени"</h4>
                                    <p>"今日、昨日、明日"</p>
                                </div>
                            </AccordionItem>
                        </Accordion>
                        
                        <div style="margin-top: 20px;">
                            <Button
                                appearance=ButtonAppearance::Primary
                                on_click=move |_| {
                                    leptos_router::use_navigate()("/grammar", Default::default());
                                }
                            >
                                "Перейти к грамматическим карточкам"
                            </Button>
                        </div>
                    </CardBody>
                </Card>
            </div>
        </MobileLayout>
    }
}