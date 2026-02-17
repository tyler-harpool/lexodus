use dioxus::prelude::*;
use shared_ui::components::{Card, CardContent, CardHeader, CardTitle};

#[component]
pub fn VacationTab(judge_id: String) -> Element {
    rsx! {
        Card {
            CardHeader { CardTitle { "Vacation Schedule" } }
            CardContent {
                p { class: "text-muted",
                    "Vacation schedule management is planned for a future release. "
                    "Use the Calendar tab to view scheduled time off."
                }
            }
        }
    }
}
