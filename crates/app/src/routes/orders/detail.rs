use dioxus::prelude::*;
use shared_ui::components::{PageHeader, PageTitle};

#[component]
pub fn OrderDetailPage(id: String) -> Element {
    rsx! {
        PageHeader {
            PageTitle { "Order Detail" }
        }
        div { class: "page-placeholder",
            p { "Order {id} — coming soon." }
        }
    }
}
