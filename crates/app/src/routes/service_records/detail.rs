use dioxus::prelude::*;
use shared_ui::components::{PageHeader, PageTitle};

#[component]
pub fn ServiceRecordDetailPage(id: String) -> Element {
    rsx! {
        PageHeader {
            PageTitle { "Service Record Detail" }
        }
        div { class: "page-placeholder",
            p { "Service Record {id} — coming soon." }
        }
    }
}
