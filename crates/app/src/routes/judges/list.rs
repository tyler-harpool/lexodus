use dioxus::prelude::*;
use shared_ui::components::{PageHeader, PageTitle};

#[component]
pub fn JudgeListPage() -> Element {
    rsx! {
        PageHeader {
            PageTitle { "Judges" }
        }
        div { class: "page-placeholder",
            p { "Judges list page — coming soon." }
        }
    }
}
