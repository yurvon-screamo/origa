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
    #[prop(optional, into)] test_id: Signal<String>,
    #[prop(optional, into)] _headers: Signal<Vec<TableHeader>>,
    #[prop(optional, into)] _rows: Signal<Vec<TableRow>>,
) -> impl IntoView {
    let test_id_val = move || {
        let val = test_id.get();
        if val.is_empty() {
            None
        } else {
            Some(val)
        }
    };

    view! {
        <div class="table-container" data-testid=test_id_val>
            <table class="table">
                <thead>
                    <tr>
                        <For
                            each=move || _headers.get()
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
                        each=move || _rows.get()
                        key=|row| row.id.clone()
                        children=move |row| {
                            let row_id = row.id.clone();
                            let cells = row.cells.clone();
                            let row_test_id_val = move || {
                                let val = test_id.get();
                                if val.is_empty() { None } else { Some(format!("{}-row-{}", val, row_id)) }
                            };
                            view! {
                                <tr data-testid=row_test_id_val>
                                    <For
                                        each=move || cells.clone()
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
