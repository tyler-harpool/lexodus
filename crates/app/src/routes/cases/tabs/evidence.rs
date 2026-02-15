use dioxus::prelude::*;
use shared_ui::components::{Card, CardContent, CardHeader};

#[component]
pub fn EvidenceTab(case_id: String) -> Element {
    rsx! {
        Card {
            CardHeader { "Evidence" }
            CardContent {
                p { "Evidence exhibit list for case {case_id} \u{2014} coming soon." }
            }
        }
    }
}
