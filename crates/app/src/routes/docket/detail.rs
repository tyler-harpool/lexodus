use dioxus::prelude::*;
use shared_ui::components::{PageHeader, PageTitle};

#[component]
pub fn DocketDetailPage(id: String) -> Element {
    rsx! {
        PageHeader {
            PageTitle { "Docket Entry Detail" }
        }
        div { class: "page-placeholder",
            p { "Docket Entry {id} — coming soon." }
        }
    }
}
