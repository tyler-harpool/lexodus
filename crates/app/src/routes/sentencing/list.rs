use dioxus::prelude::*;
use shared_ui::components::{PageHeader, PageTitle};

#[component]
pub fn SentencingListPage() -> Element {
    rsx! {
        PageHeader {
            PageTitle { "Sentencing" }
        }
        div { class: "page-placeholder",
            p { "Sentencing list page — coming soon." }
        }
    }
}
