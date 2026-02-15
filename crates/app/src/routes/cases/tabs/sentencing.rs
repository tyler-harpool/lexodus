use dioxus::prelude::*;
use shared_ui::components::{Card, CardContent, CardHeader};

#[component]
pub fn SentencingTab(case_id: String) -> Element {
    rsx! {
        Card {
            CardHeader { "Sentencing" }
            CardContent {
                p { "Sentencing data for case {case_id} \u{2014} coming soon." }
            }
        }
    }
}
