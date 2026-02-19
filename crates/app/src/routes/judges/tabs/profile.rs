use dioxus::prelude::*;
use shared_ui::components::{
    Badge, BadgeVariant, Card, CardContent, CardHeader, CardTitle, DetailGrid, DetailItem,
    DetailList,
};

#[component]
pub fn JudgeProfileTab(judge: serde_json::Value) -> Element {
    rsx! {
        DetailGrid {
            Card {
                CardHeader { CardTitle { "Biographical Information" } }
                CardContent {
                    DetailList {
                        DetailItem { label: "Name", value: judge["name"].as_str().unwrap_or("—").to_string() }
                        DetailItem { label: "Title", value: judge["title"].as_str().unwrap_or("—").to_string() }
                        DetailItem { label: "District", value: judge["district"].as_str().unwrap_or("—").to_string() }
                        DetailItem { label: "Status",
                            Badge {
                                variant: match judge["status"].as_str().unwrap_or("") {
                                    "Active" => BadgeVariant::Primary,
                                    "Senior" => BadgeVariant::Secondary,
                                    "Retired" => BadgeVariant::Outline,
                                    _ => BadgeVariant::Secondary,
                                },
                                {judge["status"].as_str().unwrap_or("—")}
                            }
                        }
                        if let Some(d) = judge["appointed_date"].as_str() {
                            DetailItem { label: "Appointed", value: d.chars().take(10).collect::<String>() }
                        }
                    }
                }
            }

            Card {
                CardHeader { CardTitle { "Chambers" } }
                CardContent {
                    DetailList {
                        if let Some(cr) = judge["courtroom"].as_str() {
                            DetailItem { label: "Courtroom", value: cr.to_string() }
                        }
                        DetailItem {
                            label: "Specializations",
                            value: judge["specializations"].as_array()
                                .map(|arr| arr.iter().filter_map(|v| v.as_str()).collect::<Vec<_>>().join(", "))
                                .unwrap_or_else(|| "None".to_string())
                        }
                    }
                }
            }

            Card {
                CardHeader { CardTitle { "Caseload Capacity" } }
                CardContent {
                    DetailList {
                        DetailItem {
                            label: "Current / Max",
                            value: format!("{} / {}",
                                judge["current_caseload"].as_i64().unwrap_or(0),
                                judge["max_caseload"].as_i64().unwrap_or(0)
                            )
                        }
                    }
                }
            }
        }
    }
}
