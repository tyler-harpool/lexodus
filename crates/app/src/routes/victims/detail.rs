use dioxus::prelude::*;
use shared_ui::components::{PageHeader, PageTitle};

#[component]
pub fn VictimDetailPage(id: String) -> Element {
    rsx! {
        PageHeader {
            PageTitle { "Victim Detail" }
        }
        div { class: "page-placeholder",
            p { "Victim {id} — coming soon." }
        }
    }
}
