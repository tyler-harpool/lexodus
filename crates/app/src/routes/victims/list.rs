use dioxus::prelude::*;
use shared_ui::components::{PageHeader, PageTitle};

#[component]
pub fn VictimListPage() -> Element {
    rsx! {
        PageHeader {
            PageTitle { "Victims" }
        }
        div { class: "page-placeholder",
            p { "Victims list page — coming soon." }
        }
    }
}
