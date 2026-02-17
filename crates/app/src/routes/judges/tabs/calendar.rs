use dioxus::prelude::*;
use shared_ui::components::{
    Badge, BadgeVariant, Card, CardContent, CardHeader, CardTitle, DataTable, DataTableBody,
    DataTableCell, DataTableColumn, DataTableHeader, DataTableRow, Skeleton,
};
use crate::CourtContext;

#[component]
pub fn JudgeCalendarTab(judge_id: String) -> Element {
    let ctx = use_context::<CourtContext>();

    let data = use_resource(move || {
        let court = ctx.court_id.read().clone();
        let jid = judge_id.clone();
        async move {
            server::api::search_calendar_events(
                court, Some(jid), None, None, None, None, None, Some(0), Some(50),
            )
            .await
            .ok()
            .and_then(|json| serde_json::from_str::<serde_json::Value>(&json).ok())
            .and_then(|v| {
                v["data"].as_array().map(|arr| arr.clone())
            })
        }
    });

    rsx! {
        Card {
            CardHeader { CardTitle { "Scheduled Events" } }
            CardContent {
                match &*data.read() {
                    Some(Some(rows)) if !rows.is_empty() => rsx! {
                        DataTable {
                            DataTableHeader {
                                DataTableColumn { "Date" }
                                DataTableColumn { "Event Type" }
                                DataTableColumn { "Case" }
                                DataTableColumn { "Status" }
                            }
                            DataTableBody {
                                for row in rows.iter() {
                                    DataTableRow {
                                        DataTableCell { {row["event_date"].as_str().unwrap_or("—").chars().take(10).collect::<String>()} }
                                        DataTableCell {
                                            Badge { variant: BadgeVariant::Secondary,
                                                {row["event_type"].as_str().unwrap_or("—")}
                                            }
                                        }
                                        DataTableCell { {row["case_id"].as_str().unwrap_or("—").chars().take(8).collect::<String>()} }
                                        DataTableCell {
                                            Badge { variant: BadgeVariant::Primary,
                                                {row["status"].as_str().unwrap_or("—")}
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    },
                    Some(_) => rsx! { p { class: "text-muted", "No scheduled events." } },
                    None => rsx! { Skeleton { width: "100%", height: "200px" } },
                }
            }
        }
    }
}
