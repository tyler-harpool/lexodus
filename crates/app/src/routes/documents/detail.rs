use dioxus::prelude::*;
use shared_ui::components::{PageHeader, PageTitle};

#[component]
pub fn DocumentDetailPage(id: String) -> Element {
    rsx! {
        PageHeader {
            PageTitle { "Document Detail" }
        }
        div { class: "page-placeholder",
            p { "Document {id} — coming soon." }
        }
    }
}
