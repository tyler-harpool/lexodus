use dioxus::prelude::*;
use shared_ui::components::{
    Badge, BadgeVariant, Card, CardContent, CardHeader, CardTitle, DataTable, DataTableBody,
    DataTableCell, DataTableColumn, DataTableHeader, DataTableRow, Skeleton,
};

use crate::CourtContext;

#[component]
pub fn ProHacViceTab(attorney_id: String) -> Element {
    let ctx = use_context::<CourtContext>();

    let data = use_resource(move || {
        let court = ctx.court_id.read().clone();
        let aid = attorney_id.clone();
        async move {
            server::api::list_pro_hac_vice(court, aid)
                .await
                .ok()
                .and_then(|json| serde_json::from_str::<Vec<serde_json::Value>>(&json).ok())
        }
    });

    rsx! {
        Card {
            CardHeader { CardTitle { "Pro Hac Vice Applications" } }
            CardContent {
                match &*data.read() {
                    Some(Some(rows)) if !rows.is_empty() => rsx! {
                        DataTable {
                            DataTableHeader {
                                DataTableColumn { "Case" }
                                DataTableColumn { "Status" }
                                DataTableColumn { "Admitted" }
                                DataTableColumn { "Expires" }
                            }
                            DataTableBody {
                                for row in rows.iter() {
                                    DataTableRow {
                                        DataTableCell { {row["case_id"].as_str().unwrap_or("—").chars().take(8).collect::<String>()} }
                                        DataTableCell {
                                            Badge {
                                                variant: phv_status_variant(row["status"].as_str().unwrap_or("")),
                                                {row["status"].as_str().unwrap_or("—")}
                                            }
                                        }
                                        DataTableCell { {row["admission_date"].as_str().unwrap_or("—").chars().take(10).collect::<String>()} }
                                        DataTableCell { {row["expiration_date"].as_str().unwrap_or("—").chars().take(10).collect::<String>()} }
                                    }
                                }
                            }
                        }
                    },
                    Some(_) => rsx! { p { class: "text-muted", "No pro hac vice applications." } },
                    None => rsx! { Skeleton {} },
                }
            }
        }
    }
}

/// Map pro hac vice status strings to appropriate badge variants.
fn phv_status_variant(status: &str) -> BadgeVariant {
    match status {
        "Active" => BadgeVariant::Primary,
        "Pending" => BadgeVariant::Secondary,
        "Expired" | "Revoked" => BadgeVariant::Destructive,
        _ => BadgeVariant::Outline,
    }
}
