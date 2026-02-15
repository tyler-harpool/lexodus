use dioxus::prelude::*;
use shared_ui::components::{Card, CardContent, CardHeader};

#[component]
pub fn SpeedyTrialTab(case_id: String) -> Element {
    rsx! {
        Card {
            CardHeader { "Speedy Trial Clock" }
            CardContent {
                p { "Speedy trial tracking for case {case_id} \u{2014} coming soon." }
            }
        }
    }
}
