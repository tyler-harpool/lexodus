use dioxus::prelude::*;
use shared_types::AttorneyResponse;
use shared_ui::components::{
    Badge, BadgeVariant, Card, CardContent, CardHeader, CardTitle, DetailGrid, DetailItem,
    DetailList,
};

#[component]
pub fn AttorneyMetricsTab(attorney: AttorneyResponse) -> Element {
    rsx! {
        DetailGrid {
            Card {
                CardHeader { CardTitle { "Caseload" } }
                CardContent {
                    DetailList {
                        DetailItem { label: "Cases Handled", value: attorney.cases_handled.to_string() }
                        DetailItem {
                            label: "CJA Panel Member",
                            Badge {
                                variant: if attorney.cja_panel_member { BadgeVariant::Primary } else { BadgeVariant::Secondary },
                                if attorney.cja_panel_member { "Yes" } else { "No" }
                            }
                        }
                    }
                }
            }

            Card {
                CardHeader { CardTitle { "Performance" } }
                CardContent {
                    DetailList {
                        if let Some(wr) = attorney.win_rate_percentage {
                            DetailItem { label: "Win Rate", value: format!("{:.1}%", wr) }
                        }
                        if let Some(dur) = attorney.avg_case_duration_days {
                            DetailItem { label: "Avg Case Duration", value: format!("{} days", dur) }
                        }
                    }
                }
            }

            Card {
                CardHeader { CardTitle { "Languages" } }
                CardContent {
                    if !attorney.languages_spoken.is_empty() {
                        div { class: "badge-group",
                            for lang in attorney.languages_spoken.iter() {
                                Badge { variant: BadgeVariant::Outline, "{lang}" }
                            }
                        }
                    } else {
                        p { class: "text-muted", "No languages listed." }
                    }
                }
            }
        }
    }
}
