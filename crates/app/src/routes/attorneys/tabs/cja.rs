use dioxus::prelude::*;
use shared_ui::components::{
    Badge, BadgeVariant, Card, CardContent, CardHeader, CardTitle, DataTable, DataTableBody,
    DataTableCell, DataTableColumn, DataTableHeader, DataTableRow, Skeleton,
};

use crate::CourtContext;

#[component]
pub fn CjaTab(attorney_id: String) -> Element {
    let ctx = use_context::<CourtContext>();

    let data = use_resource(move || {
        let court = ctx.court_id.read().clone();
        let aid = attorney_id.clone();
        async move {
            server::api::list_cja_appointments(court, aid)
                .await
                .ok()
        }
    });

    rsx! {
        Card {
            CardHeader { CardTitle { "CJA Appointments" } }
            CardContent {
                match &*data.read() {
                    Some(Some(rows)) if !rows.is_empty() => rsx! {
                        DataTable {
                            DataTableHeader {
                                DataTableColumn { "Case" }
                                DataTableColumn { "Appointed" }
                                DataTableColumn { "Terminated" }
                                DataTableColumn { "Voucher Status" }
                                DataTableColumn { "Amount" }
                            }
                            DataTableBody {
                                for row in rows.iter() {
                                    DataTableRow {
                                        DataTableCell { {row.case_id.as_deref().unwrap_or("—").chars().take(8).collect::<String>()} }
                                        DataTableCell { {row.appointment_date.chars().take(10).collect::<String>()} }
                                        DataTableCell { {row.termination_date.as_deref().and_then(|d| d.get(..10)).unwrap_or("—")} }
                                        DataTableCell {
                                            Badge {
                                                variant: voucher_status_variant(&row.voucher_status),
                                                {row.voucher_status.as_str()}
                                            }
                                        }
                                        DataTableCell {
                                            if let Some(amt) = row.voucher_amount {
                                                {format!("${:.2}", amt)}
                                            } else {
                                                {"—"}
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    },
                    Some(_) => rsx! { p { class: "text-muted", "No CJA appointments on file." } },
                    None => rsx! { Skeleton {} },
                }
            }
        }
    }
}

/// Map voucher status strings to appropriate badge variants.
fn voucher_status_variant(status: &str) -> BadgeVariant {
    match status {
        "Approved" | "Paid" => BadgeVariant::Primary,
        "Pending" | "Submitted" => BadgeVariant::Secondary,
        "Denied" => BadgeVariant::Destructive,
        _ => BadgeVariant::Outline,
    }
}
