use dioxus::prelude::*;

/// Scrollable table wrapper with co-located styles.
#[component]
pub fn DataTable(children: Element) -> Element {
    rsx! {
        document::Link { rel: "stylesheet", href: asset!("./style.css") }
        div { class: "data-table",
            table {
                {children}
            }
        }
    }
}

/// Table header section — wraps `th` elements in a `thead > tr`.
#[component]
pub fn DataTableHeader(children: Element) -> Element {
    rsx! {
        thead {
            tr { {children} }
        }
    }
}

/// Table body section.
#[component]
pub fn DataTableBody(children: Element) -> Element {
    rsx! {
        tbody { {children} }
    }
}

/// Column header cell.
#[component]
pub fn DataTableColumn(children: Element) -> Element {
    rsx! {
        th { {children} }
    }
}

/// Clickable table row that navigates on click.
#[component]
pub fn DataTableRow(
    #[props(default)] onclick: Option<EventHandler<MouseEvent>>,
    children: Element,
) -> Element {
    let has_click = onclick.is_some();
    rsx! {
        tr {
            class: if has_click { "data-table-row clickable" } else { "data-table-row" },
            onclick: move |evt| {
                if let Some(handler) = &onclick {
                    handler.call(evt);
                }
            },
            {children}
        }
    }
}

/// Table data cell.
#[component]
pub fn DataTableCell(children: Element) -> Element {
    rsx! {
        td { {children} }
    }
}
