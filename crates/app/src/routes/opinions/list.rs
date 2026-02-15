use dioxus::prelude::*;
use shared_ui::components::{PageHeader, PageTitle};

#[component]
pub fn OpinionListPage() -> Element {
    rsx! {
        PageHeader {
            PageTitle { "Opinions" }
        }
        div { class: "page-placeholder",
            p { "Opinions list page — coming soon." }
        }
    }
}
