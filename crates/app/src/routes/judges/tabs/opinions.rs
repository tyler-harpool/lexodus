use dioxus::prelude::*;
use shared_ui::components::{
    Badge, BadgeVariant, Card, CardContent, CardHeader, CardTitle, DataTable, DataTableBody,
    DataTableCell, DataTableColumn, DataTableHeader, DataTableRow, Skeleton,
};
use crate::CourtContext;

#[component]
pub fn OpinionsTab(judge_id: String) -> Element {
    let ctx = use_context::<CourtContext>();

    let data = use_resource(move || {
        let court = ctx.court_id.read().clone();
        let jid = judge_id.clone();
        async move {
            server::api::list_opinions_by_judge(court, jid)
                .await
                .ok()
                .and_then(|json| serde_json::from_str::<Vec<serde_json::Value>>(&json).ok())
        }
    });

    rsx! {
        Card {
            CardHeader { CardTitle { "Authored Opinions" } }
            CardContent {
                match &*data.read() {
                    Some(Some(rows)) if !rows.is_empty() => rsx! {
                        DataTable {
                            DataTableHeader {
                                DataTableColumn { "Title" }
                                DataTableColumn { "Type" }
                                DataTableColumn { "Status" }
                                DataTableColumn { "Filed" }
                            }
                            DataTableBody {
                                for row in rows.iter() {
                                    DataTableRow {
                                        DataTableCell { {row["title"].as_str().unwrap_or("—")} }
                                        DataTableCell {
                                            Badge { variant: BadgeVariant::Secondary,
                                                {row["opinion_type"].as_str().unwrap_or("—")}
                                            }
                                        }
                                        DataTableCell {
                                            Badge {
                                                variant: match row["status"].as_str().unwrap_or("") {
                                                    "Published" => BadgeVariant::Primary,
                                                    "Draft" => BadgeVariant::Secondary,
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
                    Some(_) => rsx! { p { class: "text-muted", "No opinions authored." } },
                    None => rsx! { Skeleton { width: "100%", height: "200px" } },
                }
            }
        }
    }
}
