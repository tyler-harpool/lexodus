use dioxus::prelude::*;
use shared_types::{BarAdmissionResponse, FederalAdmissionResponse};
use shared_ui::components::{
    Badge, BadgeVariant, Card, CardContent, CardHeader, CardTitle, DataTable, DataTableBody,
    DataTableCell, DataTableColumn, DataTableHeader, DataTableRow, Skeleton,
};

use crate::CourtContext;

#[component]
pub fn AdmissionsTab(attorney_id: String) -> Element {
    let ctx = use_context::<CourtContext>();

    let aid_bar = attorney_id.clone();
    let bar_data = use_resource(move || {
        let court = ctx.court_id.read().clone();
        let aid = aid_bar.clone();
        async move {
            server::api::list_bar_admissions(court, aid)
                .await
                .ok()
                .and_then(|json| serde_json::from_str::<Vec<BarAdmissionResponse>>(&json).ok())
        }
    });

    let aid_fed = attorney_id.clone();
    let fed_data = use_resource(move || {
        let court = ctx.court_id.read().clone();
        let aid = aid_fed.clone();
        async move {
            server::api::list_federal_admissions(court, aid)
                .await
                .ok()
                .and_then(|json| serde_json::from_str::<Vec<FederalAdmissionResponse>>(&json).ok())
        }
    });

    rsx! {
        div { class: "tab-section",
            Card {
                CardHeader { CardTitle { "State Bar Admissions" } }
                CardContent {
                    match &*bar_data.read() {
                        Some(Some(rows)) if !rows.is_empty() => rsx! {
                            DataTable {
                                DataTableHeader {
                                    DataTableColumn { "State" }
                                    DataTableColumn { "Bar Number" }
                                    DataTableColumn { "Admission Date" }
                                    DataTableColumn { "Status" }
                                }
                                DataTableBody {
                                    for row in rows.iter() {
                                        DataTableRow {
                                            DataTableCell { {row.state.as_str()} }
                                            DataTableCell { {row.bar_number.as_str()} }
                                            DataTableCell { {row.admission_date.chars().take(10).collect::<String>()} }
                                            DataTableCell {
                                                Badge { variant: BadgeVariant::Primary,
                                                    {row.status.as_str()}
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        },
                        Some(_) => rsx! { p { class: "text-muted", "No state bar admissions on file." } },
                        None => rsx! { Skeleton {} },
                    }
                }
            }
        }

        div { class: "tab-section",
            Card {
                CardHeader { CardTitle { "Federal Court Admissions" } }
                CardContent {
                    match &*fed_data.read() {
                        Some(Some(rows)) if !rows.is_empty() => rsx! {
                            DataTable {
                                DataTableHeader {
                                    DataTableColumn { "Court" }
                                    DataTableColumn { "Admission Date" }
                                    DataTableColumn { "Status" }
                                }
                                DataTableBody {
                                    for row in rows.iter() {
                                        DataTableRow {
                                            DataTableCell { {row.court_name.as_str()} }
                                            DataTableCell { {row.admission_date.chars().take(10).collect::<String>()} }
                                            DataTableCell {
                                                Badge { variant: BadgeVariant::Primary,
                                                    {row.status.as_str()}
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        },
                        Some(_) => rsx! { p { class: "text-muted", "No federal court admissions on file." } },
                        None => rsx! { Skeleton {} },
                    }
                }
            }
        }
    }
}
