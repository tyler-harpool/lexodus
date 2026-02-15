use dioxus::prelude::*;
use shared_ui::components::{PageHeader, PageTitle};

#[component]
pub fn OpinionDetailPage(id: String) -> Element {
    rsx! {
        PageHeader {
            PageTitle { "Opinion Detail" }
        }
        div { class: "page-placeholder",
            p { "Opinion {id} — coming soon." }
        }
    }
}
