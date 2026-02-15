use dioxus::prelude::*;
use shared_ui::components::{PageHeader, PageTitle};

#[component]
pub fn RuleListPage() -> Element {
    rsx! {
        PageHeader {
            PageTitle { "Rules" }
        }
        div { class: "page-placeholder",
            p { "Rules list page — coming soon." }
        }
    }
}
