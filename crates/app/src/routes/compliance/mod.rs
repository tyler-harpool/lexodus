use dioxus::prelude::*;
use shared_ui::components::{PageHeader, PageTitle};

#[component]
pub fn ComplianceDashboardPage() -> Element {
    rsx! {
        PageHeader {
            PageTitle { "Compliance Dashboard" }
        }
        div { class: "page-placeholder",
            p { "Compliance dashboard — coming soon." }
        }
    }
}
