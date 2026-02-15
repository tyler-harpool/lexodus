use dioxus::prelude::*;
use shared_ui::components::{Card, CardContent, CardHeader};

#[component]
pub fn DeadlinesTab(case_id: String) -> Element {
    rsx! {
        Card {
            CardHeader { "Case Deadlines" }
            CardContent {
                p { "Deadline tracking for case {case_id} \u{2014} coming soon." }
            }
        }
    }
}
