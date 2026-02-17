use dioxus::prelude::*;
use shared_types::{FilingListItem, NefResponse};
use shared_ui::components::{
    Badge, BadgeVariant, Button, ButtonVariant, Card, CardContent, CardHeader, CardTitle,
    DetailGrid, DetailItem, DetailList, PageActions, PageHeader, PageTitle, Skeleton, TabContent,
    TabList, TabTrigger, Tabs,
};

use crate::routes::Route;
use crate::CourtContext;

#[component]
pub fn FilingDetailPage(id: String) -> Element {
    let ctx = use_context::<CourtContext>();
    let court_id = ctx.court_id.read().clone();
    let filing_id = id.clone();

    let data = use_resource(move || {
        let court = court_id.clone();
        let fid = filing_id.clone();
        async move {
            match server::api::get_filing_by_id(court, fid).await {
                Ok(json) => serde_json::from_str::<FilingListItem>(&json).ok(),
                Err(_) => None,
            }
        }
    });

    rsx! {
        div { class: "container",
            match &*data.read() {
                Some(Some(filing)) => rsx! {
                    PageHeader {
                        PageTitle { "{filing.filing_type} Filing" }
                        PageActions {
                            Link { to: Route::FilingList {},
                                Button { variant: ButtonVariant::Secondary, "Back to List" }
                            }
                        }
                    }

                    Tabs { default_value: "info", horizontal: true,
                        TabList {
                            TabTrigger { value: "info", index: 0usize, "Info" }
                            TabTrigger { value: "validation", index: 1usize, "Validation" }
                            TabTrigger { value: "nef", index: 2usize, "NEF" }
                        }
                        TabContent { value: "info", index: 0usize,
                            InfoTab { filing: filing.clone() }
                        }
                        TabContent { value: "validation", index: 1usize,
                            ValidationTab { filing_id: id.clone() }
                        }
                        TabContent { value: "nef", index: 2usize,
                            NefTab { filing_id: id.clone() }
                        }
                    }
                },
                Some(None) => rsx! {
                    Card {
                        CardContent {
                            div { class: "empty-state",
                                h2 { "Filing Not Found" }
                                p { "The filing you're looking for doesn't exist in this court district." }
                                Link { to: Route::FilingList {},
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

/// Info tab showing the filing's core details.
#[component]
fn InfoTab(filing: FilingListItem) -> Element {
    let f = &filing;
    let date_display = f.filed_date.chars().take(10).collect::<String>();
    let created_display = f.created_at.chars().take(10).collect::<String>();
    let doc_id_display = f
        .document_id
        .clone()
        .unwrap_or_else(|| "--".to_string());
    let docket_id_display = f
        .docket_entry_id
        .clone()
        .unwrap_or_else(|| "--".to_string());

    rsx! {
        DetailGrid {
            Card {
                CardHeader { CardTitle { "Filing Information" } }
                CardContent {
                    DetailList {
                        DetailItem { label: "Filing Type",
                            Badge {
                                variant: BadgeVariant::Secondary,
                                "{f.filing_type}"
                            }
                        }
                        DetailItem { label: "Filed By", value: f.filed_by.clone() }
                        DetailItem { label: "Filed Date", value: date_display }
                        DetailItem { label: "Status",
                            Badge {
                                variant: filing_status_badge_variant(&f.status),
                                "{f.status}"
                            }
                        }
                    }
                }
            }

            Card {
                CardHeader { CardTitle { "References" } }
                CardContent {
                    DetailList {
                        DetailItem { label: "Filing ID", value: f.id.clone() }
                        DetailItem { label: "Case ID", value: f.case_id.clone() }
                        DetailItem { label: "Court ID", value: f.court_id.clone() }
                        DetailItem { label: "Document ID", value: doc_id_display }
                        DetailItem { label: "Docket Entry ID", value: docket_id_display }
                        DetailItem { label: "Created", value: created_display }
                    }
                }
            }
        }
    }
}

/// Validation tab showing the validation results for this filing.
#[component]
fn ValidationTab(filing_id: String) -> Element {
    let ctx = use_context::<CourtContext>();

    // We re-fetch the raw filing to read validation_errors from the DB.
    // Since FilingListItem doesn't include validation_errors, we use get_filing_by_id
    // which returns the FilingListItem. For the full Filing with validation_errors,
    // we'd need a different endpoint. For now, show the filing status as the
    // validation outcome.
    let filing_data = use_resource(move || {
        let court = ctx.court_id.read().clone();
        let fid = filing_id.clone();
        async move {
            match server::api::get_filing_by_id(court, fid).await {
                Ok(json) => serde_json::from_str::<FilingListItem>(&json).ok(),
                Err(_) => None,
            }
        }
    });

    rsx! {
        match &*filing_data.read() {
            Some(Some(filing)) => rsx! {
                Card {
                    CardHeader { CardTitle { "Validation Status" } }
                    CardContent {
                        DetailList {
                            DetailItem { label: "Status",
                                Badge {
                                    variant: filing_status_badge_variant(&filing.status),
                                    "{filing.status}"
                                }
                            }
                            DetailItem { label: "Filing Type", value: filing.filing_type.clone() }
                            DetailItem { label: "Filed By", value: filing.filed_by.clone() }
                        }

                        if filing.status == "Filed" || filing.status == "Accepted" {
                            div { class: "validation-summary",
                                Badge { variant: BadgeVariant::Primary, "Validation Passed" }
                                p { class: "text-muted",
                                    "This filing passed all validation checks and was accepted."
                                }
                            }
                        } else if filing.status == "Rejected" {
                            div { class: "validation-summary",
                                Badge { variant: BadgeVariant::Destructive, "Validation Failed" }
                                p { class: "text-muted",
                                    "This filing was rejected. Check the filing details for errors."
                                }
                            }
                        } else {
                            div { class: "validation-summary",
                                Badge { variant: BadgeVariant::Secondary, "{filing.status}" }
                                p { class: "text-muted",
                                    "This filing is currently being reviewed."
                                }
                            }
                        }
                    }
                }
            },
            Some(None) => rsx! {
                Card {
                    CardContent {
                        p { class: "text-muted", "Validation data not available." }
                    }
                }
            },
            None => rsx! { Skeleton {} },
        }
    }
}

/// NEF tab showing the Notice of Electronic Filing details.
#[component]
fn NefTab(filing_id: String) -> Element {
    let ctx = use_context::<CourtContext>();

    let nef_data = use_resource(move || {
        let court = ctx.court_id.read().clone();
        let fid = filing_id.clone();
        async move {
            match server::api::get_nef(court, fid).await {
                Ok(json) => serde_json::from_str::<NefResponse>(&json).ok(),
                Err(_) => None,
            }
        }
    });

    rsx! {
        match &*nef_data.read() {
            Some(Some(nef)) => rsx! {
                DetailGrid {
                    Card {
                        CardHeader { CardTitle { "Notice of Electronic Filing" } }
                        CardContent {
                            DetailList {
                                DetailItem { label: "NEF ID", value: nef.id.clone() }
                                DetailItem { label: "Filing ID", value: nef.filing_id.clone() }
                                DetailItem { label: "Document ID", value: nef.document_id.clone() }
                                DetailItem { label: "Case ID", value: nef.case_id.clone() }
                                DetailItem { label: "Docket Entry ID", value: nef.docket_entry_id.clone() }
                                DetailItem {
                                    label: "Created",
                                    value: nef.created_at.chars().take(10).collect::<String>()
                                }
                            }
                        }
                    }

                    Card {
                        CardHeader { CardTitle { "Recipients" } }
                        CardContent {
                            if nef.recipients.is_array() {
                                if let Some(arr) = nef.recipients.as_array() {
                                    if arr.is_empty() {
                                        p { class: "text-muted", "No recipients recorded." }
                                    } else {
                                        ul { class: "conditions-list",
                                            for recipient in arr.iter() {
                                                li {
                                                    if let Some(s) = recipient.as_str() {
                                                        "{s}"
                                                    } else {
                                                        "{recipient}"
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            } else {
                                p { class: "text-muted", "{nef.recipients}" }
                            }
                        }
                    }
                }
            },
            Some(None) => rsx! {
                Card {
                    CardContent {
                        p { class: "text-muted", "No Notice of Electronic Filing available for this filing." }
                    }
                }
            },
            None => rsx! { Skeleton {} },
        }
    }
}

/// Map filing status to an appropriate badge variant.
fn filing_status_badge_variant(status: &str) -> BadgeVariant {
    match status {
        "Filed" | "Accepted" => BadgeVariant::Primary,
        "Pending" | "Under Review" => BadgeVariant::Secondary,
        "Rejected" => BadgeVariant::Destructive,
        "Returned" => BadgeVariant::Outline,
        _ => BadgeVariant::Outline,
    }
}
