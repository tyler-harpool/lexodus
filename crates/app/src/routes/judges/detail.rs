use dioxus::prelude::*;
use shared_ui::components::{PageHeader, PageTitle};

#[component]
pub fn JudgeDetailPage(id: String) -> Element {
    rsx! {
        PageHeader {
            PageTitle { "Judge Detail" }
        }
        div { class: "page-placeholder",
            p { "Judge {id} — coming soon." }
        }
    }
}
