use dioxus::prelude::*;
use shared_ui::components::{PageHeader, PageTitle};

#[component]
pub fn FilingListPage() -> Element {
    rsx! {
        PageHeader {
            PageTitle { "Filings" }
        }
        div { class: "page-placeholder",
            p { "Filings list page — coming soon." }
        }
    }
}
