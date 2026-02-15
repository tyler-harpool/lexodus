use dioxus::prelude::*;
use shared_ui::components::{PageHeader, PageTitle};

#[component]
pub fn PartyListPage() -> Element {
    rsx! {
        PageHeader {
            PageTitle { "Parties" }
        }
        div { class: "page-placeholder",
            p { "Parties list page — coming soon." }
        }
    }
}
