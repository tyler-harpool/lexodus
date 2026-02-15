use dioxus::prelude::*;
use shared_ui::components::{PageHeader, PageTitle};

#[component]
pub fn RuleDetailPage(id: String) -> Element {
    rsx! {
        PageHeader {
            PageTitle { "Rule Detail" }
        }
        div { class: "page-placeholder",
            p { "Rule {id} — coming soon." }
        }
    }
}
