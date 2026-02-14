use leptos::prelude::*;

#[derive(Clone, Debug)]
pub struct TableHeader {
    pub label: String,
}

#[derive(Clone, Debug)]
pub struct TableRow {
    pub id: String,
    pub cells: Vec<String>,
}

#[component]
pub fn Table(
    #[prop(into)] headers: Vec<TableHeader>,
    #[prop(into)] rows: Vec<TableRow>,
) -> impl IntoView {
    view! {
        <div class="table-container">
            <table class="table">
                <thead>
                    <tr>
                        <For
                            each=move || headers.clone()
                            key=|header| header.label.clone()
                            children=move |header| {
                                view! {
                                    <th>{header.label}</th>
                                }
                            }
                        />
                    </tr>
                </thead>
                <tbody>
                    <For
                        each=move || rows.clone()
                        key=|row| row.id.clone()
                        children=move |row| {
                            view! {
                                <tr>
                                    <For
                                        each=move || row.cells.clone()
                                        key=|cell| cell.clone()
                                        children=move |cell| {
                                            view! {
                                                <td>{cell}</td>
                                            }
                                        }
                                    />
                                </tr>
                            }
                        }
                    />
                </tbody>
            </table>
        </div>
    }
}
