use dioxus::prelude::*;
use shared_ui::components::{PageHeader, PageTitle};

#[component]
pub fn EvidenceDetailPage(id: String) -> Element {
    rsx! {
        PageHeader {
            PageTitle { "Evidence Detail" }
        }
        div { class: "page-placeholder",
            p { "Evidence {id} — coming soon." }
        }
    }
}
