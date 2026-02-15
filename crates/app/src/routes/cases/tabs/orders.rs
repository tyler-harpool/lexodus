use dioxus::prelude::*;
use shared_ui::components::{Card, CardContent, CardHeader};

#[component]
pub fn OrdersTab(case_id: String) -> Element {
    rsx! {
        Card {
            CardHeader { "Court Orders" }
            CardContent {
                p { "Orders for case {case_id} \u{2014} coming soon." }
            }
        }
    }
}
