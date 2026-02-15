use dioxus::prelude::*;
use shared_ui::components::{PageHeader, PageTitle};

#[component]
pub fn ServiceRecordListPage() -> Element {
    rsx! {
        PageHeader {
            PageTitle { "Service Records" }
        }
        div { class: "page-placeholder",
            p { "Service Records list page — coming soon." }
        }
    }
}
