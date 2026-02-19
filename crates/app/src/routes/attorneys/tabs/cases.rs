use dioxus::prelude::*;
use shared_ui::components::{
    Badge, BadgeVariant, Card, CardContent, CardHeader, CardTitle, DataTable, DataTableBody,
    DataTableCell, DataTableColumn, DataTableHeader, DataTableRow, Skeleton,
};

use crate::CourtContext;

#[component]
pub fn AttorneyCasesTab(attorney_id: String) -> Element {
    let ctx = use_context::<CourtContext>();

    let data = use_resource(move || {
        let court = ctx.court_id.read().clone();
        let aid = attorney_id.clone();
        async move {
            server::api::list_active_representations(court, aid)
                .await
                .ok()
                .and_then(|json| serde_json::from_str::<Vec<serde_json::Value>>(&json).ok())
        }
    });

    rsx! {
        Card {
            CardHeader { CardTitle { "Active Representations" } }
            CardContent {
                match &*data.read() {
                    Some(Some(rows)) if !rows.is_empty() => rsx! {
                        DataTable {
                            DataTableHeader {
                                DataTableColumn { "Case" }
                                DataTableColumn { "Role" }
                                DataTableColumn { "Start Date" }
                                DataTableColumn { "Status" }
                            }
                            DataTableBody {
                                for row in rows.iter() {
                                    DataTableRow {
                                        DataTableCell { {row["case_id"].as_str().unwrap_or("—").chars().take(8).collect::<String>()} }
                                        DataTableCell { {row["role"].as_str().unwrap_or("—")} }
                                        DataTableCell { {row["start_date"].as_str().unwrap_or("—").chars().take(10).collect::<String>()} }
                                        DataTableCell {
                                            Badge { variant: BadgeVariant::Primary,
                                                {row["status"].as_str().unwrap_or("active")}
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    },
                    Some(_) => rsx! { p { class: "text-muted", "No active representations." } },
                    None => rsx! { Skeleton {} },
                }
            }
        }
    }
}
