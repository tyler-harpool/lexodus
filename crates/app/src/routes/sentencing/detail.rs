use dioxus::prelude::*;
use shared_ui::components::{PageHeader, PageTitle};

#[component]
pub fn SentencingDetailPage(id: String) -> Element {
    rsx! {
        PageHeader {
            PageTitle { "Sentencing Detail" }
        }
        div { class: "page-placeholder",
            p { "Sentencing {id} — coming soon." }
        }
    }
}
