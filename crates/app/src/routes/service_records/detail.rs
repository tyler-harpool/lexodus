use dioxus::prelude::*;
use shared_types::ServiceRecordResponse;
use shared_ui::components::{
    Badge, BadgeVariant, Button, ButtonVariant, Card, CardContent, CardHeader, CardTitle,
    DetailGrid, DetailItem, DetailList, PageActions, PageHeader, PageTitle, Skeleton,
};

use crate::routes::Route;
use crate::CourtContext;

#[component]
pub fn ServiceRecordDetailPage(id: String) -> Element {
    let ctx = use_context::<CourtContext>();
    let court_id = ctx.court_id.read().clone();
    let record_id = id.clone();

    let data = use_resource(move || {
        let court = court_id.clone();
        let rid = record_id.clone();
        async move {
            match server::api::get_service_record(court, rid).await {
                Ok(json) => serde_json::from_str::<ServiceRecordResponse>(&json).ok(),
                Err(_) => None,
            }
        }
    });

    rsx! {
        div { class: "container",
            match &*data.read() {
                Some(Some(record)) => rsx! {
                    ServiceRecordDetail { record: record.clone(), id: id.clone() }
                },
                Some(None) => rsx! {
                    Card {
                        CardContent {
                            div { class: "empty-state",
                                h2 { "Service Record Not Found" }
                                p { "The service record you're looking for doesn't exist in this court district." }
                                Link { to: Route::ServiceRecordList {},
                                    Button { "Back to List" }
                                }
                            }
                        }
                    }
                },
                None => rsx! {
                    div { class: "loading",
                        Skeleton {}
                        Skeleton {}
                        Skeleton {}
                    }
                },
            }
        }
    }
}

#[component]
fn ServiceRecordDetail(record: ServiceRecordResponse, id: String) -> Element {
    let date_display = record
        .service_date
        .chars()
        .take(10)
        .collect::<String>();

    let successful_variant = if record.successful {
        BadgeVariant::Primary
    } else {
        BadgeVariant::Destructive
    };
    let successful_label = if record.successful { "Yes" } else { "No" };

    let proof_variant = if record.proof_of_service_filed {
        BadgeVariant::Primary
    } else {
        BadgeVariant::Outline
    };
    let proof_label = if record.proof_of_service_filed { "Filed" } else { "Not Filed" };

    let notes_display = record
        .notes
        .clone()
        .unwrap_or_else(|| "--".to_string());

    let certificate_display = record
        .certificate_of_service
        .clone()
        .unwrap_or_else(|| "--".to_string());

    rsx! {
        PageHeader {
            PageTitle { "Service Record" }
            PageActions {
                Link { to: Route::ServiceRecordList {},
                    Button { variant: ButtonVariant::Secondary, "Back to List" }
                }
            }
        }

        DetailGrid {
            Card {
                CardHeader { CardTitle { "Service Information" } }
                CardContent {
                    DetailList {
                        DetailItem { label: "Document ID", value: record.document_id.clone() }
                        DetailItem { label: "Party",
                            span {
                                "{record.party_name}"
                                span { class: "text-muted", " ({record.party_type})" }
                            }
                        }
                        DetailItem { label: "Service Method",
                            Badge { variant: method_badge_variant(&record.service_method),
                                "{record.service_method}"
                            }
                        }
                        DetailItem { label: "Served By", value: record.served_by.clone() }
                        DetailItem { label: "Service Date", value: date_display }
                    }
                }
            }

            Card {
                CardHeader { CardTitle { "Status" } }
                CardContent {
                    DetailList {
                        DetailItem { label: "Successful",
                            Badge { variant: successful_variant, "{successful_label}" }
                        }
                        DetailItem { label: "Proof of Service",
                            Badge { variant: proof_variant, "{proof_label}" }
                        }
                        DetailItem { label: "Attempts", value: format!("{}", record.attempts) }
                    }
                }
            }

            Card {
                CardHeader { CardTitle { "Additional Details" } }
                CardContent {
                    DetailList {
                        DetailItem { label: "Notes", value: notes_display }
                        DetailItem { label: "Certificate of Service", value: certificate_display }
                        DetailItem { label: "Record ID", value: record.id.clone() }
                        DetailItem { label: "Court ID", value: record.court_id.clone() }
                    }
                }
            }
        }
    }
}

/// Map service method to a badge variant.
fn method_badge_variant(method: &str) -> BadgeVariant {
    match method {
        "ECF" | "Electronic" => BadgeVariant::Primary,
        "Personal Service" => BadgeVariant::Secondary,
        "Mail" | "Certified Mail" | "Express Mail" => BadgeVariant::Outline,
        "Publication" => BadgeVariant::Outline,
        _ => BadgeVariant::Outline,
    }
}
