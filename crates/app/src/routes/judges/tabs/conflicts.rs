use dioxus::prelude::*;
use shared_ui::components::{
    Badge, BadgeVariant, Card, CardContent, CardHeader, CardTitle, DataTable, DataTableBody,
    DataTableCell, DataTableColumn, DataTableHeader, DataTableRow, Skeleton,
};
use crate::CourtContext;

#[component]
pub fn ConflictsTab(judge_id: String) -> Element {
    let ctx = use_context::<CourtContext>();

    let jid_conflicts = judge_id.clone();
    let conflicts = use_resource(move || {
        let court = ctx.court_id.read().clone();
        let jid = jid_conflicts.clone();
        async move {
            server::api::list_judge_conflicts(court, jid)
                .await
                .ok()
                .and_then(|json| serde_json::from_str::<Vec<serde_json::Value>>(&json).ok())
        }
    });

    let jid_recusals = judge_id.clone();
    let recusals = use_resource(move || {
        let court = ctx.court_id.read().clone();
        let jid = jid_recusals.clone();
        async move {
            server::api::list_recusals_by_judge(court, jid)
                .await
                .ok()
                .and_then(|json| serde_json::from_str::<Vec<serde_json::Value>>(&json).ok())
        }
    });

    rsx! {
        div { class: "tab-section",
            Card {
                CardHeader { CardTitle { "Declared Conflicts" } }
                CardContent {
                    match &*conflicts.read() {
                        Some(Some(rows)) if !rows.is_empty() => rsx! {
                            DataTable {
                                DataTableHeader {
                                    DataTableColumn { "Party / Entity" }
                                    DataTableColumn { "Conflict Type" }
                                    DataTableColumn { "Start Date" }
                                    DataTableColumn { "Notes" }
                                }
                                DataTableBody {
                                    for row in rows.iter() {
                                        DataTableRow {
                                            DataTableCell { {row["conflicting_party"].as_str().unwrap_or("—")} }
                                            DataTableCell {
                                                Badge { variant: BadgeVariant::Destructive,
                                                    {row["conflict_type"].as_str().unwrap_or("—")}
                                                }
                                            }
                                            DataTableCell { {row["start_date"].as_str().unwrap_or("—").chars().take(10).collect::<String>()} }
                                            DataTableCell { {row["notes"].as_str().unwrap_or("—")} }
                                        }
                                    }
                                }
                            }
                        },
                        Some(_) => rsx! { p { class: "text-muted", "No declared conflicts." } },
                        None => rsx! { Skeleton { width: "100%", height: "120px" } },
                    }
                }
            }
        }

        div { class: "tab-section",
            Card {
                CardHeader { CardTitle { "Recusal Motions" } }
                CardContent {
                    match &*recusals.read() {
                        Some(Some(rows)) if !rows.is_empty() => rsx! {
                            DataTable {
                                DataTableHeader {
                                    DataTableColumn { "Case" }
                                    DataTableColumn { "Reason" }
                                    DataTableColumn { "Status" }
                                    DataTableColumn { "Filed" }
                                }
                                DataTableBody {
                                    for row in rows.iter() {
                                        DataTableRow {
                                            DataTableCell { {row["case_id"].as_str().unwrap_or("—").chars().take(8).collect::<String>()} }
                                            DataTableCell { {row["reason"].as_str().unwrap_or("—")} }
                                            DataTableCell {
                                                Badge {
                                                    variant: match row["status"].as_str().unwrap_or("") {
                                                        "Granted" => BadgeVariant::Primary,
                                                        "Denied" => BadgeVariant::Destructive,
                                                        "Pending" => BadgeVariant::Secondary,
                                                        _ => BadgeVariant::Outline,
                                                    },
                                                    {row["status"].as_str().unwrap_or("—")}
                                                }
                                            }
                                            DataTableCell { {row["filed_date"].as_str().unwrap_or("—").chars().take(10).collect::<String>()} }
                                        }
                                    }
                                }
                            }
                        },
                        Some(_) => rsx! { p { class: "text-muted", "No recusal motions." } },
                        None => rsx! { Skeleton { width: "100%", height: "120px" } },
                    }
                }
            }
        }
    }
}
