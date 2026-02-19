use dioxus::prelude::*;
use shared_ui::components::{
    Badge, BadgeVariant, Card, CardContent, CardHeader, CardTitle, DataTable, DataTableBody,
    DataTableCell, DataTableColumn, DataTableHeader, DataTableRow, Skeleton,
};
use crate::CourtContext;

#[component]
pub fn CaseloadTab(judge_id: String) -> Element {
    let ctx = use_context::<CourtContext>();

    let data = use_resource(move || {
        let court = ctx.court_id.read().clone();
        let jid = judge_id.clone();
        async move {
            server::api::list_assignments_by_judge(court, jid)
                .await
                .ok()
                .and_then(|json| serde_json::from_str::<Vec<serde_json::Value>>(&json).ok())
        }
    });

    rsx! {
        Card {
            CardHeader { CardTitle { "Assigned Cases" } }
            CardContent {
                match &*data.read() {
                    Some(Some(rows)) if !rows.is_empty() => rsx! {
                        DataTable {
                            DataTableHeader {
                                DataTableColumn { "Case" }
                                DataTableColumn { "Assignment Type" }
                                DataTableColumn { "Assigned Date" }
                                DataTableColumn { "Reason" }
                            }
                            DataTableBody {
                                for row in rows.iter() {
                                    DataTableRow {
                                        DataTableCell { {row["case_id"].as_str().unwrap_or("—").chars().take(8).collect::<String>()} }
                                        DataTableCell {
                                            Badge { variant: BadgeVariant::Secondary,
                                                {row["assignment_type"].as_str().unwrap_or("—")}
                                            }
                                        }
                                        DataTableCell { {row["assigned_date"].as_str().unwrap_or("—").chars().take(10).collect::<String>()} }
                                        DataTableCell { {row["reason"].as_str().unwrap_or("—")} }
                                    }
                                }
                            }
                        }
                    },
                    Some(_) => rsx! { p { class: "text-muted", "No case assignments." } },
                    None => rsx! { Skeleton { width: "100%", height: "200px" } },
                }
            }
        }
    }
}
