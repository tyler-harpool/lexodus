use dioxus::prelude::*;
use shared_ui::components::{PageHeader, PageTitle};

#[component]
pub fn DefendantListPage() -> Element {
    rsx! {
        PageHeader {
            PageTitle { "Defendants" }
        }
        div { class: "page-placeholder",
            p { "Defendants list page — coming soon." }
        }
    }
}
