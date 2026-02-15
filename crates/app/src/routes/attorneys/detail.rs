use dioxus::prelude::*;
use shared_types::AttorneyResponse;
use shared_ui::components::{
    AlertDialogAction, AlertDialogActions, AlertDialogCancel, AlertDialogContent,
    AlertDialogDescription, AlertDialogRoot, AlertDialogTitle, Badge, BadgeVariant, Button,
    ButtonVariant, Card, CardContent, CardHeader, CardTitle, DetailFooter, DetailGrid, DetailItem,
    DetailList, PageActions, PageHeader, PageTitle, Skeleton,
};

use crate::routes::Route;
use crate::CourtContext;

#[component]
pub fn AttorneyDetailPage(id: String) -> Element {
    let ctx = use_context::<CourtContext>();
    let court_id = ctx.court_id.read().clone();
    let attorney_id = id.clone();

    let mut show_delete_confirm = use_signal(|| false);
    let mut deleting = use_signal(|| false);

    let data = use_resource(move || {
        let court = court_id.clone();
        let aid = attorney_id.clone();
        async move {
            match server::api::get_attorney(court, aid).await {
                Ok(json) => serde_json::from_str::<AttorneyResponse>(&json).ok(),
                Err(_) => None,
            }
        }
    });

    let detail_id = id.clone();
    let handle_delete = move |_: MouseEvent| {
        let court = ctx.court_id.read().clone();
        let aid = detail_id.clone();
        spawn(async move {
            deleting.set(true);
            match server::api::delete_attorney(court, aid).await {
                Ok(()) => {
                    let nav = navigator();
                    nav.push(Route::AttorneyList {});
                }
                Err(_) => {
                    deleting.set(false);
                    show_delete_confirm.set(false);
                }
            }
        });
    };

    rsx! {
        div { class: "container",
            match &*data.read() {
                Some(Some(att)) => rsx! {
                    PageHeader {
                        PageTitle { "{att.last_name}, {att.first_name}" }
                        PageActions {
                            Link { to: Route::AttorneyList {},
                                Button { variant: ButtonVariant::Secondary, "Back to List" }
                            }
                            Button {
                                variant: ButtonVariant::Destructive,
                                onclick: move |_| show_delete_confirm.set(true),
                                "Delete"
                            }
                        }
                    }

                    AlertDialogRoot {
                        open: show_delete_confirm(),
                        on_open_change: move |v| show_delete_confirm.set(v),
                        AlertDialogContent {
                            AlertDialogTitle { "Delete Attorney" }
                            AlertDialogDescription {
                                "Are you sure you want to delete this attorney? This action cannot be undone."
                            }
                            AlertDialogActions {
                                AlertDialogCancel { "Cancel" }
                                AlertDialogAction {
                                    on_click: handle_delete,
                                    if *deleting.read() { "Deleting..." } else { "Delete" }
                                }
                            }
                        }
                    }

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
                            CardHeader { CardTitle { "Practice Details" } }
                            CardContent {
                                DetailList {
                                    DetailItem {
                                        label: "CJA Panel Member",
                                        value: (if att.cja_panel_member { "Yes" } else { "No" }).to_string()
                                    }
                                    DetailItem {
                                        label: "Cases Handled",
                                        value: att.cases_handled.to_string()
                                    }
                                    if let Some(wr) = att.win_rate_percentage {
                                        DetailItem {
                                            label: "Win Rate",
                                            value: format!("{:.1}%", wr)
                                        }
                                    }
                                    if let Some(dur) = att.avg_case_duration_days {
                                        DetailItem {
                                            label: "Avg Case Duration",
                                            value: format!("{} days", dur)
                                        }
                                    }
                                    if !att.languages_spoken.is_empty() {
                                        DetailItem {
                                            label: "Languages",
                                            value: att.languages_spoken.join(", ")
                                        }
                                    }
                                }
                            }
                        }
                    }

                    DetailFooter {
                        span { "ID: {att.id}" }
                        span { "Created: {att.created_at}" }
                        span { "Updated: {att.updated_at}" }
                    }
                },
                Some(None) => rsx! {
                    Card {
                        CardContent {
                            div { class: "empty-state",
                                h2 { "Attorney Not Found" }
                                p { "The attorney you're looking for doesn't exist in this court district." }
                                Link { to: Route::AttorneyList {},
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

fn status_badge_variant(status: &str) -> BadgeVariant {
    match status {
        "Active" => BadgeVariant::Primary,
        "Inactive" => BadgeVariant::Secondary,
        "Suspended" => BadgeVariant::Destructive,
        "Retired" => BadgeVariant::Outline,
        _ => BadgeVariant::Secondary,
    }
}
