use dioxus::prelude::*;
use shared_types::{Party, PartyResponse, RepresentationResponse};
use shared_ui::components::{
    AlertDialogAction, AlertDialogActions, AlertDialogCancel, AlertDialogContent,
    AlertDialogDescription, AlertDialogRoot, AlertDialogTitle, Badge, BadgeVariant, Button,
    ButtonVariant, Card, CardContent, CardHeader, CardTitle, DataTable, DataTableBody,
    DataTableCell, DataTableColumn, DataTableHeader, DataTableRow, DetailGrid, DetailItem,
    DetailList, PageActions, PageHeader, PageTitle, Skeleton, TabContent, TabList, TabTrigger, Tabs,
};
use shared_ui::{use_toast, ToastOptions};

use super::form_sheet::{FormMode, PartyFormSheet};
use crate::auth::{can, use_user_role, Action};
use crate::routes::Route;
use crate::CourtContext;

#[component]
pub fn PartyDetailPage(id: String) -> Element {
    let ctx = use_context::<CourtContext>();
    let court_id = ctx.court_id.read().clone();
    let party_id = id.clone();
    let toast = use_toast();

    let role = use_user_role();
    let mut show_edit = use_signal(|| false);
    let mut show_delete_confirm = use_signal(|| false);
    let mut deleting = use_signal(|| false);

    let mut data = use_resource(move || {
        let court = court_id.clone();
        let pid = party_id.clone();
        async move {
            match server::api::get_party(court, pid).await {
                Ok(json) => serde_json::from_str::<Party>(&json).ok(),
                Err(_) => None,
            }
        }
    });

    let detail_id = id.clone();
    let handle_delete = move |_: MouseEvent| {
        let court = ctx.court_id.read().clone();
        let pid = detail_id.clone();
        spawn(async move {
            deleting.set(true);
            match server::api::delete_party(court, pid).await {
                Ok(()) => {
                    toast.success(
                        "Party deleted successfully".to_string(),
                        ToastOptions::new(),
                    );
                    let nav = navigator();
                    nav.push(Route::PartyList {});
                }
                Err(e) => {
                    toast.error(format!("{}", e), ToastOptions::new());
                    deleting.set(false);
                    show_delete_confirm.set(false);
                }
            }
        });
    };

    rsx! {
        div { class: "container",
            match &*data.read() {
                Some(Some(party)) => rsx! {
                    PageHeader {
                        PageTitle { "{party.name}" }
                        PageActions {
                            Link { to: Route::PartyList {},
                                Button { variant: ButtonVariant::Secondary, "Back to List" }
                            }
                            if can(&role, Action::Edit) {
                                Button {
                                    variant: ButtonVariant::Primary,
                                    onclick: move |_| show_edit.set(true),
                                    "Edit"
                                }
                            }
                            if can(&role, Action::Delete) {
                                Button {
                                    variant: ButtonVariant::Destructive,
                                    onclick: move |_| show_delete_confirm.set(true),
                                    "Delete"
                                }
                            }
                        }
                    }

                    AlertDialogRoot {
                        open: show_delete_confirm(),
                        on_open_change: move |v| show_delete_confirm.set(v),
                        AlertDialogContent {
                            AlertDialogTitle { "Delete Party" }
                            AlertDialogDescription {
                                "Are you sure you want to delete this party? This action cannot be undone."
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

                    Tabs { default_value: "profile", horizontal: true,
                        TabList {
                            TabTrigger { value: "profile", index: 0usize, "Profile" }
                            TabTrigger { value: "representations", index: 1usize, "Representations" }
                            TabTrigger { value: "service", index: 2usize, "Service" }
                        }
                        TabContent { value: "profile", index: 0usize,
                            ProfileTab { party: party.clone() }
                        }
                        TabContent { value: "representations", index: 1usize,
                            RepresentationsTab { party_id: id.clone() }
                        }
                        TabContent { value: "service", index: 2usize,
                            ServiceTab { party: party.clone() }
                        }
                    }

                    {
                        let party_response: PartyResponse = party.clone().into();
                        rsx! {
                            PartyFormSheet {
                                mode: FormMode::Edit,
                                initial: Some(party_response),
                                open: show_edit(),
                                on_close: move |_| show_edit.set(false),
                                on_saved: move |_| data.restart(),
                            }
                        }
                    }
                },
                Some(None) => rsx! {
                    Card {
                        CardContent {
                            div { class: "empty-state",
                                h2 { "Party Not Found" }
                                p { "The party you're looking for doesn't exist in this court district." }
                                Link { to: Route::PartyList {},
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

/// Profile tab showing the party's information.
#[component]
fn ProfileTab(party: Party) -> Element {
    let p = &party;
    let email_display = p
        .email
        .clone()
        .unwrap_or_else(|| "--".to_string());
    let phone_display = p
        .phone
        .clone()
        .unwrap_or_else(|| "--".to_string());
    let dob_display = p
        .date_of_birth
        .map(|d| d.to_string())
        .unwrap_or_else(|| "--".to_string());
    let first_name_display = p
        .first_name
        .clone()
        .unwrap_or_else(|| "--".to_string());
    let last_name_display = p
        .last_name
        .clone()
        .unwrap_or_else(|| "--".to_string());
    let org_display = p
        .organization_name
        .clone()
        .unwrap_or_else(|| "--".to_string());
    let joined_display = p
        .joined_date
        .map(|d| d.format("%Y-%m-%d").to_string())
        .unwrap_or_else(|| "--".to_string());
    let terminated_display = p
        .terminated_date
        .map(|d| d.format("%Y-%m-%d").to_string())
        .unwrap_or_else(|| "--".to_string());

    let street_display = p
        .address_street1
        .clone()
        .unwrap_or_else(|| "--".to_string());
    let city_display = p
        .address_city
        .clone()
        .unwrap_or_default();
    let state_display = p
        .address_state
        .clone()
        .unwrap_or_default();
    let zip_display = p
        .address_zip
        .clone()
        .unwrap_or_default();

    let has_address = p.address_street1.is_some();
    let address_line = if has_address {
        format!("{}, {} {} {}", street_display, city_display, state_display, zip_display)
    } else {
        "--".to_string()
    };

    rsx! {
        DetailGrid {
            Card {
                CardHeader { CardTitle { "Party Information" } }
                CardContent {
                    DetailList {
                        DetailItem { label: "Full Name", value: p.name.clone() }
                        DetailItem { label: "First Name", value: first_name_display }
                        DetailItem { label: "Last Name", value: last_name_display }
                        DetailItem { label: "Party Type",
                            Badge {
                                variant: BadgeVariant::Secondary,
                                "{p.party_type}"
                            }
                        }
                        DetailItem { label: "Party Role", value: p.party_role.clone() }
                        DetailItem { label: "Entity Type",
                            Badge {
                                variant: BadgeVariant::Outline,
                                "{p.entity_type}"
                            }
                        }
                        if p.entity_type == "Corporation" || p.entity_type == "LLC" || p.entity_type == "Partnership" {
                            DetailItem { label: "Organization Name", value: org_display }
                        }
                        DetailItem { label: "Date of Birth", value: dob_display }
                    }
                }
            }

            Card {
                CardHeader { CardTitle { "Contact Information" } }
                CardContent {
                    DetailList {
                        DetailItem { label: "Email", value: email_display }
                        DetailItem { label: "Phone", value: phone_display }
                        DetailItem { label: "Address", value: address_line }
                    }
                }
            }

            Card {
                CardHeader { CardTitle { "Status & Dates" } }
                CardContent {
                    DetailList {
                        DetailItem { label: "Status",
                            Badge {
                                variant: party_status_badge_variant(&p.status),
                                "{p.status}"
                            }
                        }
                        DetailItem { label: "Represented",
                            if p.represented {
                                Badge { variant: BadgeVariant::Primary, "Yes" }
                            } else {
                                Badge { variant: BadgeVariant::Outline, "No" }
                            }
                        }
                        DetailItem { label: "Pro Se",
                            if p.pro_se {
                                Badge { variant: BadgeVariant::Secondary, "Yes" }
                            } else {
                                Badge { variant: BadgeVariant::Outline, "No" }
                            }
                        }
                        DetailItem { label: "Joined Date", value: joined_display }
                        DetailItem { label: "Terminated Date", value: terminated_display }
                        DetailItem {
                            label: "Created",
                            value: p.created_at.format("%Y-%m-%d").to_string()
                        }
                        DetailItem {
                            label: "Updated",
                            value: p.updated_at.format("%Y-%m-%d").to_string()
                        }
                    }
                }
            }
        }
    }
}

/// Representations tab listing attorney representations for this party.
#[component]
fn RepresentationsTab(party_id: String) -> Element {
    let ctx = use_context::<CourtContext>();

    let representations = use_resource(move || {
        let court = ctx.court_id.read().clone();
        let pid = party_id.clone();
        async move {
            match server::api::list_representations_by_party(court, pid).await {
                Ok(json) => serde_json::from_str::<Vec<RepresentationResponse>>(&json).ok(),
                Err(_) => None,
            }
        }
    });

    rsx! {
        match &*representations.read() {
            Some(Some(list)) if !list.is_empty() => rsx! {
                DataTable {
                    DataTableHeader {
                        DataTableColumn { "Attorney" }
                        DataTableColumn { "Type" }
                        DataTableColumn { "Status" }
                        DataTableColumn { "Lead" }
                        DataTableColumn { "Start Date" }
                        DataTableColumn { "End Date" }
                    }
                    DataTableBody {
                        for rep in list.iter() {
                            RepresentationRow { rep: rep.clone() }
                        }
                    }
                }
            },
            Some(_) => rsx! {
                Card {
                    CardContent {
                        p { class: "text-muted", "No attorney representations found for this party." }
                    }
                }
            },
            None => rsx! { Skeleton {} },
        }
    }
}

/// Individual representation row in the representations table.
#[component]
fn RepresentationRow(rep: RepresentationResponse) -> Element {
    let status_variant = rep_status_badge_variant(&rep.status);
    let attorney_id_short = if rep.attorney_id.len() > 8 {
        format!("{}...", &rep.attorney_id[..8])
    } else {
        rep.attorney_id.clone()
    };
    let start_date = rep.start_date.chars().take(10).collect::<String>();
    let end_date = rep
        .end_date
        .as_deref()
        .map(|d| d.chars().take(10).collect::<String>())
        .unwrap_or_else(|| "--".to_string());

    rsx! {
        DataTableRow {
            DataTableCell { "{attorney_id_short}" }
            DataTableCell { "{rep.representation_type}" }
            DataTableCell {
                Badge { variant: status_variant, "{rep.status}" }
            }
            DataTableCell {
                if rep.lead_counsel {
                    Badge { variant: BadgeVariant::Primary, "Yes" }
                } else {
                    span { "No" }
                }
            }
            DataTableCell { "{start_date}" }
            DataTableCell { "{end_date}" }
        }
    }
}

/// Service tab showing service method and NEF opt-in status.
#[component]
fn ServiceTab(party: Party) -> Element {
    let p = &party;
    let service_method_display = p
        .service_method
        .clone()
        .unwrap_or_else(|| "--".to_string());

    rsx! {
        DetailGrid {
            Card {
                CardHeader { CardTitle { "Service Information" } }
                CardContent {
                    DetailList {
                        DetailItem { label: "Service Method",
                            if p.service_method.is_some() {
                                Badge { variant: BadgeVariant::Secondary, "{service_method_display}" }
                            } else {
                                span { "{service_method_display}" }
                            }
                        }
                        DetailItem { label: "NEF SMS Opt-In",
                            if p.nef_sms_opt_in {
                                Badge { variant: BadgeVariant::Primary, "Opted In" }
                            } else {
                                Badge { variant: BadgeVariant::Outline, "Not Opted In" }
                            }
                        }
                        DetailItem { label: "Pro Se",
                            if p.pro_se {
                                Badge { variant: BadgeVariant::Secondary, "Yes" }
                            } else {
                                Badge { variant: BadgeVariant::Outline, "No" }
                            }
                        }
                    }
                }
            }
        }
    }
}

/// Map party status to a badge variant.
fn party_status_badge_variant(status: &str) -> BadgeVariant {
    match status {
        "Active" => BadgeVariant::Primary,
        "Terminated" | "Dismissed" | "Deceased" => BadgeVariant::Destructive,
        "Defaulted" | "In Contempt" => BadgeVariant::Destructive,
        "Settled" => BadgeVariant::Secondary,
        _ => BadgeVariant::Outline,
    }
}

/// Map representation status to a badge variant.
fn rep_status_badge_variant(status: &str) -> BadgeVariant {
    match status {
        "Active" => BadgeVariant::Primary,
        "Withdrawn" | "Terminated" => BadgeVariant::Destructive,
        "Substituted" => BadgeVariant::Secondary,
        "Suspended" => BadgeVariant::Outline,
        "Completed" => BadgeVariant::Secondary,
        _ => BadgeVariant::Outline,
    }
}
