use dioxus::prelude::*;
use shared_ui::components::{Card, CardContent, CardHeader};

#[component]
pub fn CalendarTab(case_id: String) -> Element {
    rsx! {
        Card {
            CardHeader { "Case Calendar" }
            CardContent {
                p { "Scheduled events for case {case_id} \u{2014} coming soon." }
            }
        }
    }
}
