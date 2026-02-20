use dioxus::prelude::*;
use shared_types::CaseAssignmentResponse;
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
                .and_then(|json| serde_json::from_str::<Vec<CaseAssignmentResponse>>(&json).ok())
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
                                    {render_assignment_row(row)}
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

fn render_assignment_row(assignment: &CaseAssignmentResponse) -> Element {
    let case_display = assignment.case_id.chars().take(8).collect::<String>();
    let date_display = assignment.assigned_date.chars().take(10).collect::<String>();
    let reason_display = assignment.reason.as_deref().unwrap_or("—").to_string();

    rsx! {
        DataTableRow {
            DataTableCell { "{case_display}" }
            DataTableCell {
                Badge { variant: BadgeVariant::Secondary,
                    {assignment.assignment_type.as_str()}
                }
            }
            DataTableCell { "{date_display}" }
            DataTableCell { "{reason_display}" }
        }
    }
}
