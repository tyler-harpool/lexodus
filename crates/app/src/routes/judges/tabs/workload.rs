use dioxus::prelude::*;
use shared_ui::components::{
    Card, CardContent, CardHeader, CardTitle, DetailGrid, DetailItem, DetailList,
};

#[component]
pub fn WorkloadTab(judge_id: String, judge: serde_json::Value) -> Element {
    let current = judge["current_caseload"].as_i64().unwrap_or(0);
    let max = judge["max_caseload"].as_i64().unwrap_or(1);
    let utilization = if max > 0 { current as f64 / max as f64 * 100.0 } else { 0.0 };

    rsx! {
        DetailGrid {
            Card {
                CardHeader { CardTitle { "Caseload" } }
                CardContent {
                    DetailList {
                        DetailItem { label: "Active Cases", value: current.to_string() }
                        DetailItem { label: "Maximum Capacity", value: max.to_string() }
                        DetailItem { label: "Utilization", value: format!("{:.0}%", utilization) }
                    }
                }
            }

            Card {
                CardHeader { CardTitle { "Performance" } }
                CardContent {
                    DetailList {
                        DetailItem { label: "Status", value: judge["status"].as_str().unwrap_or("—").to_string() }
                        if let Some(d) = judge["appointed_date"].as_str() {
                            DetailItem { label: "Years on Bench", value: d.chars().take(4).collect::<String>() }
                        }
                    }
                }
            }
        }
    }
}
