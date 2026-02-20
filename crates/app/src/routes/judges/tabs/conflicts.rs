use dioxus::prelude::*;
use shared_types::{JudgeConflictResponse, RecusalMotionResponse};
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
                                        {render_conflict_row(row)}
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
                                        {render_recusal_row(row)}
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

/// Builds a display string for the conflicting party from available fields.
fn conflicting_party_display(conflict: &JudgeConflictResponse) -> String {
    let parts: Vec<&str> = [
        conflict.party_name.as_deref(),
        conflict.law_firm.as_deref(),
        conflict.corporation.as_deref(),
    ]
    .into_iter()
    .flatten()
    .collect();

    if parts.is_empty() {
        "—".to_string()
    } else {
        parts.join(", ")
    }
}

fn render_conflict_row(conflict: &JudgeConflictResponse) -> Element {
    let party_display = conflicting_party_display(conflict);
    let start_display = conflict.start_date.chars().take(10).collect::<String>();
    let notes_display = conflict.notes.as_deref().unwrap_or("—").to_string();

    rsx! {
        DataTableRow {
            DataTableCell { "{party_display}" }
            DataTableCell {
                Badge { variant: BadgeVariant::Destructive,
                    {conflict.conflict_type.as_str()}
                }
            }
            DataTableCell { "{start_display}" }
            DataTableCell { "{notes_display}" }
        }
    }
}

fn render_recusal_row(recusal: &RecusalMotionResponse) -> Element {
    let case_display = recusal.case_id.chars().take(8).collect::<String>();
    let filed_display = recusal.filed_date.chars().take(10).collect::<String>();

    let status_variant = match recusal.status.as_str() {
        "Granted" => BadgeVariant::Primary,
        "Denied" => BadgeVariant::Destructive,
        "Pending" => BadgeVariant::Secondary,
        _ => BadgeVariant::Outline,
    };

    rsx! {
        DataTableRow {
            DataTableCell { "{case_display}" }
            DataTableCell { {recusal.reason.as_str()} }
            DataTableCell {
                Badge {
                    variant: status_variant,
                    {recusal.status.as_str()}
                }
            }
            DataTableCell { "{filed_display}" }
        }
    }
}
