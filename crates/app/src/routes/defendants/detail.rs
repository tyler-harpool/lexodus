use dioxus::prelude::*;
use shared_types::{ChargeResponse, DefendantResponse};
use shared_ui::components::{
    AlertDialogAction, AlertDialogActions, AlertDialogCancel, AlertDialogContent,
    AlertDialogDescription, AlertDialogRoot, AlertDialogTitle, Badge, BadgeVariant, Button,
    ButtonVariant, Card, CardContent, CardHeader, CardTitle, DetailGrid, DetailItem, DetailList,
    PageActions, PageHeader, PageTitle, Skeleton, TabContent, TabList, TabTrigger, Tabs,
};
use shared_ui::{use_toast, ToastOptions};

use super::form_sheet::{DefendantFormSheet, FormMode};
use crate::auth::{can, use_user_role, Action};
use crate::routes::Route;
use crate::CourtContext;

#[component]
pub fn DefendantDetailPage(id: String) -> Element {
    let ctx = use_context::<CourtContext>();
    let court_id = ctx.court_id.read().clone();
    let defendant_id = id.clone();
    let toast = use_toast();

    let role = use_user_role();
    let mut show_edit = use_signal(|| false);
    let mut show_delete_confirm = use_signal(|| false);
    let mut deleting = use_signal(|| false);

    let mut data = use_resource(move || {
        let court = court_id.clone();
        let did = defendant_id.clone();
        async move {
            match server::api::get_defendant(court, did).await {
                Ok(json) => serde_json::from_str::<DefendantResponse>(&json).ok(),
                Err(_) => None,
            }
        }
    });

    let detail_id = id.clone();
    let handle_delete = move |_: MouseEvent| {
        let court = ctx.court_id.read().clone();
        let did = detail_id.clone();
        spawn(async move {
            deleting.set(true);
            match server::api::delete_defendant(court, did).await {
                Ok(()) => {
                    toast.success(
                        "Defendant deleted successfully".to_string(),
                        ToastOptions::new(),
                    );
                    let nav = navigator();
                    nav.push(Route::DefendantList {});
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
                Some(Some(def)) => rsx! {
                    PageHeader {
                        PageTitle { "{def.name}" }
                        PageActions {
                            Link { to: Route::DefendantList {},
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
                            AlertDialogTitle { "Delete Defendant" }
                            AlertDialogDescription {
                                "Are you sure you want to delete this defendant? This action cannot be undone."
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
                            TabTrigger { value: "charges", index: 1usize, "Charges" }
                            TabTrigger { value: "bond", index: 2usize, "Bond" }
                        }
                        TabContent { value: "profile", index: 0usize,
                            ProfileTab { defendant: def.clone() }
                        }
                        TabContent { value: "charges", index: 1usize,
                            ChargesTab { defendant_id: id.clone() }
                        }
                        TabContent { value: "bond", index: 2usize,
                            BondTab { defendant: def.clone() }
                        }
                    }

                    DefendantFormSheet {
                        mode: FormMode::Edit,
                        initial: Some(def.clone()),
                        open: show_edit(),
                        on_close: move |_| show_edit.set(false),
                        on_saved: move |_| data.restart(),
                    }
                },
                Some(None) => rsx! {
                    Card {
                        CardContent {
                            div { class: "empty-state",
                                h2 { "Defendant Not Found" }
                                p { "The defendant you're looking for doesn't exist in this court district." }
                                Link { to: Route::DefendantList {},
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

/// Profile tab showing the defendant's personal information.
#[component]
fn ProfileTab(defendant: DefendantResponse) -> Element {
    let def = &defendant;
    let dob_display = def
        .date_of_birth
        .clone()
        .unwrap_or_else(|| "--".to_string());
    let usm_display = def
        .usm_number
        .clone()
        .unwrap_or_else(|| "--".to_string());
    let fbi_display = def
        .fbi_number
        .clone()
        .unwrap_or_else(|| "--".to_string());

    rsx! {
        DetailGrid {
            Card {
                CardHeader { CardTitle { "Personal Information" } }
                CardContent {
                    DetailList {
                        DetailItem { label: "Full Name", value: def.name.clone() }
                        DetailItem { label: "Date of Birth", value: dob_display }
                        DetailItem { label: "Citizenship Status",
                            Badge {
                                variant: BadgeVariant::Secondary,
                                "{def.citizenship_status}"
                            }
                        }
                        if !def.aliases.is_empty() {
                            DetailItem {
                                label: "Aliases",
                                value: def.aliases.join(", ")
                            }
                        }
                    }
                }
            }

            Card {
                CardHeader { CardTitle { "Identification" } }
                CardContent {
                    DetailList {
                        DetailItem { label: "USM Number", value: usm_display }
                        DetailItem { label: "FBI Number", value: fbi_display }
                        DetailItem { label: "Case ID", value: def.case_id.clone() }
                    }
                }
            }

            Card {
                CardHeader { CardTitle { "Custody" } }
                CardContent {
                    DetailList {
                        DetailItem { label: "Custody Status",
                            Badge {
                                variant: custody_badge_variant(&def.custody_status),
                                "{def.custody_status}"
                            }
                        }
                        DetailItem {
                            label: "Created",
                            value: def.created_at.chars().take(10).collect::<String>()
                        }
                        DetailItem {
                            label: "Updated",
                            value: def.updated_at.chars().take(10).collect::<String>()
                        }
                    }
                }
            }
        }
    }
}

/// Charges tab listing all charges against this defendant.
#[component]
fn ChargesTab(defendant_id: String) -> Element {
    let ctx = use_context::<CourtContext>();

    let charges = use_resource(move || {
        let court = ctx.court_id.read().clone();
        let did = defendant_id.clone();
        async move {
            match server::api::list_charges_by_defendant(court, did).await {
                Ok(json) => serde_json::from_str::<Vec<ChargeResponse>>(&json).ok(),
                Err(_) => None,
            }
        }
    });

    rsx! {
        match &*charges.read() {
            Some(Some(list)) if !list.is_empty() => rsx! {
                div { class: "charges-list",
                    for charge in list.iter() {
                        ChargeCard { charge: charge.clone() }
                    }
                }
            },
            Some(_) => rsx! {
                Card {
                    CardContent {
                        p { class: "text-muted", "No charges filed against this defendant." }
                    }
                }
            },
            None => rsx! { Skeleton {} },
        }
    }
}

/// Individual charge card display.
#[component]
fn ChargeCard(charge: ChargeResponse) -> Element {
    let plea_variant = plea_badge_variant(&charge.plea);
    let verdict_variant = verdict_badge_variant(&charge.verdict);

    let sentence_range = match (charge.statutory_min_months, charge.statutory_max_months) {
        (Some(min), Some(max)) => format!("{} - {} months", min, max),
        (None, Some(max)) => format!("Up to {} months", max),
        (Some(min), None) => format!("{} months minimum", min),
        (None, None) => "--".to_string(),
    };

    let plea_date_display = charge
        .plea_date
        .as_deref()
        .map(|d| d.chars().take(10).collect::<String>())
        .unwrap_or_else(|| "--".to_string());

    let verdict_date_display = charge
        .verdict_date
        .as_deref()
        .map(|d| d.chars().take(10).collect::<String>())
        .unwrap_or_else(|| "--".to_string());

    rsx! {
        Card {
            CardHeader {
                CardTitle { "Count {charge.count_number}: {charge.statute}" }
            }
            CardContent {
                DetailList {
                    DetailItem { label: "Offense", value: charge.offense_description.clone() }
                    DetailItem { label: "Statutory Range", value: sentence_range }
                    DetailItem { label: "Plea",
                        Badge { variant: plea_variant, "{charge.plea}" }
                    }
                    DetailItem { label: "Plea Date", value: plea_date_display }
                    DetailItem { label: "Verdict",
                        Badge { variant: verdict_variant, "{charge.verdict}" }
                    }
                    DetailItem { label: "Verdict Date", value: verdict_date_display }
                }
            }
        }
    }
}

/// Bond tab displaying bail/bond information for the defendant.
#[component]
fn BondTab(defendant: DefendantResponse) -> Element {
    let def = &defendant;

    let bail_type_display = def
        .bail_type
        .clone()
        .unwrap_or_else(|| "--".to_string());
    let bail_amount_display = def
        .bail_amount
        .map(|a| format!("${:.2}", a))
        .unwrap_or_else(|| "--".to_string());
    let bond_posted_display = def
        .bond_posted_date
        .as_deref()
        .map(|d| d.chars().take(10).collect::<String>())
        .unwrap_or_else(|| "--".to_string());
    let surety_display = def
        .surety_name
        .clone()
        .unwrap_or_else(|| "--".to_string());

    rsx! {
        DetailGrid {
            Card {
                CardHeader { CardTitle { "Bail Information" } }
                CardContent {
                    DetailList {
                        DetailItem { label: "Bail Type",
                            if def.bail_type.is_some() {
                                Badge { variant: BadgeVariant::Primary, "{bail_type_display}" }
                            } else {
                                span { "{bail_type_display}" }
                            }
                        }
                        DetailItem { label: "Bail Amount", value: bail_amount_display }
                        DetailItem { label: "Bond Posted Date", value: bond_posted_display }
                        DetailItem { label: "Surety Name", value: surety_display }
                    }
                }
            }

            Card {
                CardHeader { CardTitle { "Bond Conditions" } }
                CardContent {
                    if def.bond_conditions.is_empty() {
                        p { class: "text-muted", "No bond conditions recorded." }
                    } else {
                        ul { class: "conditions-list",
                            for condition in def.bond_conditions.iter() {
                                li { "{condition}" }
                            }
                        }
                    }
                }
            }
        }
    }
}

/// Map custody status to a badge variant.
fn custody_badge_variant(status: &str) -> BadgeVariant {
    match status {
        "In Custody" => BadgeVariant::Destructive,
        "Bail" | "Bond" => BadgeVariant::Primary,
        "Released" | "Supervised Release" => BadgeVariant::Secondary,
        "Fugitive" => BadgeVariant::Destructive,
        _ => BadgeVariant::Outline,
    }
}

/// Map plea type to a badge variant.
fn plea_badge_variant(plea: &str) -> BadgeVariant {
    match plea {
        "Guilty" | "No Contest" | "Alford" => BadgeVariant::Destructive,
        "Not Guilty" => BadgeVariant::Primary,
        "Not Yet Entered" => BadgeVariant::Outline,
        _ => BadgeVariant::Secondary,
    }
}

/// Map verdict type to a badge variant.
fn verdict_badge_variant(verdict: &str) -> BadgeVariant {
    match verdict {
        "Guilty" => BadgeVariant::Destructive,
        "Not Guilty" | "Acquitted" => BadgeVariant::Primary,
        "Dismissed" => BadgeVariant::Secondary,
        "Mistrial" | "Hung Jury" => BadgeVariant::Outline,
        _ => BadgeVariant::Outline,
    }
}
