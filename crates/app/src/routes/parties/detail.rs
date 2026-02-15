use dioxus::prelude::*;
use shared_ui::components::{PageHeader, PageTitle};

#[component]
pub fn PartyDetailPage(id: String) -> Element {
    rsx! {
        PageHeader {
            PageTitle { "Party Detail" }
        }
        div { class: "page-placeholder",
            p { "Party {id} — coming soon." }
        }
    }
}
