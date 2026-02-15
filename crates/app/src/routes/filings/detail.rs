use dioxus::prelude::*;
use shared_ui::components::{PageHeader, PageTitle};

#[component]
pub fn FilingDetailPage(id: String) -> Element {
    rsx! {
        PageHeader {
            PageTitle { "Filing Detail" }
        }
        div { class: "page-placeholder",
            p { "Filing {id} — coming soon." }
        }
    }
}
