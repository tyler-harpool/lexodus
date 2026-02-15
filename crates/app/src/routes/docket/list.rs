use dioxus::prelude::*;
use shared_ui::components::{PageHeader, PageTitle};

#[component]
pub fn DocketListPage() -> Element {
    rsx! {
        PageHeader {
            PageTitle { "Docket" }
        }
        div { class: "page-placeholder",
            p { "Docket list page — coming soon." }
        }
    }
}
