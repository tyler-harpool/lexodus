use dioxus::prelude::*;
use shared_ui::components::{PageHeader, PageTitle};

#[component]
pub fn EvidenceListPage() -> Element {
    rsx! {
        PageHeader {
            PageTitle { "Evidence" }
        }
        div { class: "page-placeholder",
            p { "Evidence list page — coming soon." }
        }
    }
}
