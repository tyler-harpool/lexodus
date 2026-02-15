use dioxus::prelude::*;

use crate::components::button::{Button, ButtonVariant};

/// Offset-based pagination controls with Previous/Next buttons.
#[component]
pub fn Pagination(total: i64, offset: Signal<i64>, limit: i64) -> Element {
    let current_page = (*offset.read() / limit) + 1;
    let total_pages = if limit > 0 {
        (total + limit - 1) / limit
    } else {
        1
    };

    rsx! {
        document::Link { rel: "stylesheet", href: asset!("./style.css") }
        div { class: "pagination",
            if current_page > 1 {
                Button {
                    variant: ButtonVariant::Outline,
                    onclick: move |_| {
                        let current = *offset.read();
                        offset.set((current - limit).max(0));
                    },
                    "Previous"
                }
            }
            span { class: "pagination-info",
                "Page {current_page} of {total_pages} ({total} total)"
            }
            if current_page < total_pages {
                Button {
                    variant: ButtonVariant::Outline,
                    onclick: move |_| {
                        let current = *offset.read();
                        offset.set(current + limit);
                    },
                    "Next"
                }
            }
        }
    }
}
