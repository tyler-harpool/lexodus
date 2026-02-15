use dioxus::prelude::*;
use shared_ui::components::{PageHeader, PageTitle};

#[component]
pub fn OrderListPage() -> Element {
    rsx! {
        PageHeader {
            PageTitle { "Orders" }
        }
        div { class: "page-placeholder",
            p { "Orders list page — coming soon." }
        }
    }
}
