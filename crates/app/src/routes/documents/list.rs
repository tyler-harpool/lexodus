use dioxus::prelude::*;
use shared_ui::components::{PageHeader, PageTitle};

#[component]
pub fn DocumentListPage() -> Element {
    rsx! {
        PageHeader {
            PageTitle { "Documents" }
        }
        div { class: "page-placeholder",
            p { "Documents list page — coming soon." }
        }
    }
}
