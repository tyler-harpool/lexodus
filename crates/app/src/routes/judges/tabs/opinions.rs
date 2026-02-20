use dioxus::prelude::*;
use shared_types::JudicialOpinionResponse;
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
                .and_then(|json| serde_json::from_str::<Vec<JudicialOpinionResponse>>(&json).ok())
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
                                    OpinionRow { opinion: row.clone() }
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

#[component]
fn OpinionRow(opinion: JudicialOpinionResponse) -> Element {
    let filed_display = opinion.filed_at
        .as_deref()
        .map(|d| d.chars().take(10).collect::<String>())
        .unwrap_or_else(|| "—".to_string());

    let status_variant = match opinion.status.as_str() {
        "Published" => BadgeVariant::Primary,
        "Draft" => BadgeVariant::Secondary,
        _ => BadgeVariant::Outline,
    };

    rsx! {
        DataTableRow {
            DataTableCell { {opinion.title.as_str()} }
            DataTableCell {
                Badge { variant: BadgeVariant::Secondary,
                    {opinion.opinion_type.as_str()}
                }
            }
            DataTableCell {
                Badge {
                    variant: status_variant,
                    {opinion.status.as_str()}
                }
            }
            DataTableCell { "{filed_display}" }
        }
    }
}
