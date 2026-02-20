use dioxus::prelude::*;
use shared_types::{AttorneyResponse, EcfRegistrationResponse, PracticeAreaResponse};
use shared_ui::components::{
    Badge, BadgeVariant, Card, CardContent, CardHeader, CardTitle, DetailGrid, DetailItem,
    DetailList, Skeleton,
};

use crate::CourtContext;

#[component]
pub fn ProfileTab(attorney: AttorneyResponse, attorney_id: String) -> Element {
    let ctx = use_context::<CourtContext>();

    let aid_pa = attorney_id.clone();
    let practice_areas = use_resource(move || {
        let court = ctx.court_id.read().clone();
        let aid = aid_pa.clone();
        async move {
            server::api::list_practice_areas(court, aid)
                .await
                .ok()
                .and_then(|json| serde_json::from_str::<Vec<PracticeAreaResponse>>(&json).ok())
        }
    });

    let aid_ecf = attorney_id.clone();
    let ecf = use_resource(move || {
        let court = ctx.court_id.read().clone();
        let aid = aid_ecf.clone();
        async move {
            server::api::get_ecf_registration(court, aid)
                .await
                .ok()
                .and_then(|json| serde_json::from_str::<EcfRegistrationResponse>(&json).ok())
        }
    });

    let att = &attorney;

    rsx! {
        DetailGrid {
            Card {
                CardHeader { CardTitle { "Basic Information" } }
                CardContent {
                    DetailList {
                        DetailItem { label: "Bar Number", value: att.bar_number.clone() }
                        DetailItem { label: "First Name", value: att.first_name.clone() }
                        DetailItem { label: "Last Name", value: att.last_name.clone() }
                        if let Some(mid) = &att.middle_name {
                            DetailItem { label: "Middle Name", value: mid.clone() }
                        }
                        if let Some(firm) = &att.firm_name {
                            DetailItem { label: "Firm", value: firm.clone() }
                        }
                        DetailItem { label: "Status",
                            Badge {
                                variant: status_badge_variant(&att.status),
                                "{att.status}"
                            }
                        }
                    }
                }
            }

            Card {
                CardHeader { CardTitle { "Contact" } }
                CardContent {
                    DetailList {
                        DetailItem { label: "Email", value: att.email.clone() }
                        DetailItem { label: "Phone", value: att.phone.clone() }
                        if let Some(fax) = &att.fax {
                            DetailItem { label: "Fax", value: fax.clone() }
                        }
                    }
                }
            }

            Card {
                CardHeader { CardTitle { "Address" } }
                CardContent {
                    DetailList {
                        DetailItem { label: "Street", value: att.address.street1.clone() }
                        if let Some(s2) = &att.address.street2 {
                            DetailItem { label: "Street 2", value: s2.clone() }
                        }
                        DetailItem { label: "City", value: att.address.city.clone() }
                        DetailItem { label: "State", value: att.address.state.clone() }
                        DetailItem { label: "ZIP", value: att.address.zip_code.clone() }
                        DetailItem { label: "Country", value: att.address.country.clone() }
                    }
                }
            }

            Card {
                CardHeader { CardTitle { "Practice Areas" } }
                CardContent {
                    match &*practice_areas.read() {
                        Some(Some(areas)) if !areas.is_empty() => rsx! {
                            div { class: "badge-group",
                                for area in areas.iter() {
                                    Badge { variant: BadgeVariant::Secondary,
                                        {area.area.as_str()}
                                    }
                                }
                            }
                        },
                        Some(_) => rsx! { p { class: "text-muted", "No practice areas listed." } },
                        None => rsx! { Skeleton {} },
                    }
                }
            }

            Card {
                CardHeader { CardTitle { "ECF Registration" } }
                CardContent {
                    match &*ecf.read() {
                        Some(Some(reg)) => rsx! {
                            DetailList {
                                DetailItem { label: "Status",
                                    Badge { variant: BadgeVariant::Primary,
                                        {reg.status.as_str()}
                                    }
                                }
                                DetailItem {
                                    label: "Registered",
                                    value: reg.registration_date.chars().take(10).collect::<String>()
                                }
                            }
                        },
                        Some(None) => rsx! { p { class: "text-muted", "Not registered for ECF." } },
                        None => rsx! { Skeleton {} },
                    }
                }
            }
        }
    }
}

fn status_badge_variant(status: &str) -> BadgeVariant {
    match status {
        "Active" => BadgeVariant::Primary,
        "Inactive" => BadgeVariant::Secondary,
        "Suspended" => BadgeVariant::Destructive,
        "Retired" => BadgeVariant::Outline,
        _ => BadgeVariant::Secondary,
    }
}
