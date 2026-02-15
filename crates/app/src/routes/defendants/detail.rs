use dioxus::prelude::*;
use shared_ui::components::{PageHeader, PageTitle};

#[component]
pub fn DefendantDetailPage(id: String) -> Element {
    rsx! {
        PageHeader {
            PageTitle { "Defendant Detail" }
        }
        div { class: "page-placeholder",
            p { "Defendant {id} — coming soon." }
        }
    }
}
