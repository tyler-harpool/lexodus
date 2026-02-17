use dioxus::prelude::*;
use shared_ui::components::{
    Badge, BadgeVariant, Card, CardContent, CardHeader, CardTitle, DataTable, DataTableBody,
    DataTableCell, DataTableColumn, DataTableHeader, DataTableRow, Skeleton,
};

use crate::CourtContext;

#[component]
pub fn DisciplineTab(attorney_id: String) -> Element {
    let ctx = use_context::<CourtContext>();

    let data = use_resource(move || {
        let court = ctx.court_id.read().clone();
        let aid = attorney_id.clone();
        async move {
            server::api::list_discipline_records(court, aid)
                .await
                .ok()
                .and_then(|json| serde_json::from_str::<Vec<serde_json::Value>>(&json).ok())
        }
    });

    rsx! {
        Card {
            CardHeader { CardTitle { "Disciplinary Actions" } }
            CardContent {
                match &*data.read() {
                    Some(Some(rows)) if !rows.is_empty() => rsx! {
                        DataTable {
                            DataTableHeader {
                                DataTableColumn { "Action Type" }
                                DataTableColumn { "Jurisdiction" }
                                DataTableColumn { "Date" }
                                DataTableColumn { "Description" }
                            }
                            DataTableBody {
                                for row in rows.iter() {
                                    DataTableRow {
                                        DataTableCell {
                                            Badge { variant: BadgeVariant::Destructive,
                                                {row["action_type"].as_str().unwrap_or("—")}
                                            }
                                        }
                                        DataTableCell { {row["jurisdiction"].as_str().unwrap_or("—")} }
                                        DataTableCell { {row["action_date"].as_str().unwrap_or("—").chars().take(10).collect::<String>()} }
                                        DataTableCell { {row["description"].as_str().unwrap_or("—")} }
                                    }
                                }
                            }
                        }
                    },
                    Some(_) => rsx! { p { class: "text-muted", "No disciplinary actions on record." } },
                    None => rsx! { Skeleton {} },
                }
            }
        }
    }
}
