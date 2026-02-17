use dioxus::prelude::*;
use shared_types::{
    BopDesignationResponse, PriorSentenceResponse, SentencingResponse,
    SpecialConditionResponse,
};
use shared_ui::components::{
    AlertDialogAction, AlertDialogActions, AlertDialogCancel, AlertDialogContent,
    AlertDialogDescription, AlertDialogRoot, AlertDialogTitle, Badge, BadgeVariant, Button,
    ButtonVariant, Card, CardContent, CardHeader, CardTitle, DetailGrid, DetailItem, DetailList,
    PageActions, PageHeader, PageTitle, Skeleton, TabContent, TabList, TabTrigger, Tabs,
};
use shared_ui::{use_toast, ToastOptions};

use crate::routes::Route;
use crate::CourtContext;

#[component]
pub fn SentencingDetailPage(id: String) -> Element {
    let ctx = use_context::<CourtContext>();
    let court_id = ctx.court_id.read().clone();
    let sentencing_id = id.clone();
    let toast = use_toast();

    let mut show_delete_confirm = use_signal(|| false);
    let mut deleting = use_signal(|| false);

    let data = use_resource(move || {
        let court = court_id.clone();
        let sid = sentencing_id.clone();
        async move {
            match server::api::get_sentencing(court, sid).await {
                Ok(json) => serde_json::from_str::<SentencingResponse>(&json).ok(),
                Err(_) => None,
            }
        }
    });

    let detail_id = id.clone();
    let handle_delete = move |_: MouseEvent| {
        let court = ctx.court_id.read().clone();
        let sid = detail_id.clone();
        spawn(async move {
            deleting.set(true);
            match server::api::delete_sentencing(court, sid).await {
                Ok(()) => {
                    toast.success(
                        "Sentencing record deleted successfully".to_string(),
                        ToastOptions::new(),
                    );
                    let nav = navigator();
                    nav.push(Route::SentencingList {});
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
                Some(Some(record)) => rsx! {
                    PageHeader {
                        PageTitle { "Sentencing Record" }
                        PageActions {
                            Link { to: Route::SentencingList {},
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
                            AlertDialogTitle { "Delete Sentencing Record" }
                            AlertDialogDescription {
                                "Are you sure you want to delete this sentencing record? This action cannot be undone."
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

                    Tabs { default_value: "guidelines", horizontal: true,
                        TabList {
                            TabTrigger { value: "guidelines", index: 0usize, "Guidelines" }
                            TabTrigger { value: "sentence", index: 1usize, "Sentence" }
                            TabTrigger { value: "conditions", index: 2usize, "Conditions" }
                            TabTrigger { value: "bop", index: 3usize, "BOP" }
                            TabTrigger { value: "prior", index: 4usize, "Prior Sentences" }
                        }
                        TabContent { value: "guidelines", index: 0usize,
                            GuidelinesTab { record: record.clone() }
                        }
                        TabContent { value: "sentence", index: 1usize,
                            SentenceTab { record: record.clone() }
                        }
                        TabContent { value: "conditions", index: 2usize,
                            ConditionsTab { sentencing_id: id.clone() }
                        }
                        TabContent { value: "bop", index: 3usize,
                            BopTab { sentencing_id: id.clone() }
                        }
                        TabContent { value: "prior", index: 4usize,
                            PriorSentencesTab { sentencing_id: id.clone() }
                        }
                    }
                },
                Some(None) => rsx! {
                    Card {
                        CardContent {
                            div { class: "empty-state",
                                h2 { "Sentencing Record Not Found" }
                                p { "The sentencing record you're looking for doesn't exist in this court district." }
                                Link { to: Route::SentencingList {},
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

/// Guidelines tab showing offense level, criminal history, and guidelines range.
#[component]
fn GuidelinesTab(record: SentencingResponse) -> Element {
    let base_level = display_opt_i32(record.base_offense_level);
    let specific_level = display_opt_i32(record.specific_offense_level);
    let adjusted_level = display_opt_i32(record.adjusted_offense_level);
    let total_level = display_opt_i32(record.total_offense_level);
    let history_cat = record
        .criminal_history_category
        .clone()
        .unwrap_or_else(|| "--".to_string());
    let history_points = display_opt_i32(record.criminal_history_points);
    let range_low = display_opt_i32(record.guidelines_range_low_months);
    let range_high = display_opt_i32(record.guidelines_range_high_months);
    let guidelines_display = match (record.guidelines_range_low_months, record.guidelines_range_high_months) {
        (Some(low), Some(high)) => format!("{} - {} months", low, high),
        _ => "--".to_string(),
    };

    let departure_type = record
        .departure_type
        .clone()
        .unwrap_or_else(|| "None".to_string());
    let departure_reason = record
        .departure_reason
        .clone()
        .unwrap_or_else(|| "--".to_string());
    let variance_type = record
        .variance_type
        .clone()
        .unwrap_or_else(|| "None".to_string());
    let variance_justification = record
        .variance_justification
        .clone()
        .unwrap_or_else(|| "--".to_string());

    let departure_variant = departure_badge_variant(&departure_type);
    let variance_variant = departure_badge_variant(&variance_type);

    rsx! {
        DetailGrid {
            Card {
                CardHeader { CardTitle { "Offense Level Calculation" } }
                CardContent {
                    DetailList {
                        DetailItem { label: "Base Offense Level", value: base_level }
                        DetailItem { label: "Specific Offense Level", value: specific_level }
                        DetailItem { label: "Adjusted Offense Level", value: adjusted_level }
                        DetailItem { label: "Total Offense Level", value: total_level }
                    }
                }
            }

            Card {
                CardHeader { CardTitle { "Criminal History" } }
                CardContent {
                    DetailList {
                        DetailItem { label: "Category",
                            if history_cat != "--" {
                                Badge { variant: BadgeVariant::Primary, "{history_cat}" }
                            } else {
                                span { "{history_cat}" }
                            }
                        }
                        DetailItem { label: "Points", value: history_points }
                    }
                }
            }

            Card {
                CardHeader { CardTitle { "Guidelines Range" } }
                CardContent {
                    DetailList {
                        DetailItem { label: "Range Low (months)", value: range_low }
                        DetailItem { label: "Range High (months)", value: range_high }
                        DetailItem { label: "Full Range", value: guidelines_display }
                    }
                }
            }

            Card {
                CardHeader { CardTitle { "Departure / Variance" } }
                CardContent {
                    DetailList {
                        DetailItem { label: "Departure Type",
                            Badge { variant: departure_variant, "{departure_type}" }
                        }
                        DetailItem { label: "Departure Reason", value: departure_reason }
                        DetailItem { label: "Variance Type",
                            Badge { variant: variance_variant, "{variance_type}" }
                        }
                        DetailItem { label: "Variance Justification", value: variance_justification }
                    }
                }
            }
        }
    }
}

/// Sentence tab showing custody, supervised release, fines, and financial details.
#[component]
fn SentenceTab(record: SentencingResponse) -> Element {
    let custody = display_opt_i32(record.custody_months);
    let probation = display_opt_i32(record.probation_months);
    let supervised = display_opt_i32(record.supervised_release_months);
    let fine = display_opt_money(record.fine_amount);
    let restitution = display_opt_money(record.restitution_amount);
    let forfeiture = display_opt_money(record.forfeiture_amount);
    let special_assessment = display_opt_money(record.special_assessment);
    let appeal_waiver_display = if record.appeal_waiver { "Yes" } else { "No" };
    let sentencing_date = record
        .sentencing_date
        .as_deref()
        .map(|d| d.chars().take(10).collect::<String>())
        .unwrap_or_else(|| "--".to_string());
    let judgment_date = record
        .judgment_date
        .as_deref()
        .map(|d| d.chars().take(10).collect::<String>())
        .unwrap_or_else(|| "--".to_string());

    rsx! {
        DetailGrid {
            Card {
                CardHeader { CardTitle { "Incarceration" } }
                CardContent {
                    DetailList {
                        DetailItem { label: "Custody Months", value: custody }
                        DetailItem { label: "Probation Months", value: probation }
                        DetailItem { label: "Supervised Release Months", value: supervised }
                    }
                }
            }

            Card {
                CardHeader { CardTitle { "Financial" } }
                CardContent {
                    DetailList {
                        DetailItem { label: "Fine", value: fine }
                        DetailItem { label: "Restitution", value: restitution }
                        DetailItem { label: "Forfeiture", value: forfeiture }
                        DetailItem { label: "Special Assessment", value: special_assessment }
                    }
                }
            }

            Card {
                CardHeader { CardTitle { "Key Dates" } }
                CardContent {
                    DetailList {
                        DetailItem { label: "Sentencing Date", value: sentencing_date }
                        DetailItem { label: "Judgment Date", value: judgment_date }
                        DetailItem { label: "Appeal Waiver",
                            Badge {
                                variant: if record.appeal_waiver { BadgeVariant::Destructive } else { BadgeVariant::Secondary },
                                "{appeal_waiver_display}"
                            }
                        }
                    }
                }
            }

            Card {
                CardHeader { CardTitle { "Record Info" } }
                CardContent {
                    DetailList {
                        DetailItem { label: "Case ID", value: record.case_id.clone() }
                        DetailItem { label: "Defendant ID", value: record.defendant_id.clone() }
                        DetailItem { label: "Judge ID", value: record.judge_id.clone() }
                        DetailItem {
                            label: "Created",
                            value: record.created_at.chars().take(10).collect::<String>()
                        }
                        DetailItem {
                            label: "Updated",
                            value: record.updated_at.chars().take(10).collect::<String>()
                        }
                    }
                }
            }
        }
    }
}

/// Conditions tab listing special conditions for this sentencing record.
#[component]
fn ConditionsTab(sentencing_id: String) -> Element {
    let ctx = use_context::<CourtContext>();

    let conditions = use_resource(move || {
        let court = ctx.court_id.read().clone();
        let sid = sentencing_id.clone();
        async move {
            match server::api::list_sentencing_conditions(court, sid).await {
                Ok(json) => serde_json::from_str::<Vec<SpecialConditionResponse>>(&json).ok(),
                Err(_) => None,
            }
        }
    });

    rsx! {
        match &*conditions.read() {
            Some(Some(list)) if !list.is_empty() => rsx! {
                div { class: "charges-list",
                    for condition in list.iter() {
                        ConditionCard { condition: condition.clone() }
                    }
                }
            },
            Some(_) => rsx! {
                Card {
                    CardContent {
                        p { class: "text-muted", "No special conditions recorded for this sentencing." }
                    }
                }
            },
            None => rsx! { Skeleton {} },
        }
    }
}

/// Individual condition card display.
#[component]
fn ConditionCard(condition: SpecialConditionResponse) -> Element {
    let status_variant = condition_status_variant(&condition.status);
    let effective_display = condition
        .effective_date
        .as_deref()
        .map(|d| d.chars().take(10).collect::<String>())
        .unwrap_or_else(|| "--".to_string());

    rsx! {
        Card {
            CardHeader {
                CardTitle { "{condition.condition_type}" }
            }
            CardContent {
                DetailList {
                    DetailItem { label: "Description", value: condition.description.clone() }
                    DetailItem { label: "Status",
                        Badge { variant: status_variant, "{condition.status}" }
                    }
                    DetailItem { label: "Effective Date", value: effective_display }
                    DetailItem {
                        label: "Created",
                        value: condition.created_at.chars().take(10).collect::<String>()
                    }
                }
            }
        }
    }
}

/// BOP tab listing Bureau of Prisons designations.
#[component]
fn BopTab(sentencing_id: String) -> Element {
    let ctx = use_context::<CourtContext>();

    let designations = use_resource(move || {
        let court = ctx.court_id.read().clone();
        let sid = sentencing_id.clone();
        async move {
            match server::api::list_bop_designations(court, sid).await {
                Ok(json) => serde_json::from_str::<Vec<BopDesignationResponse>>(&json).ok(),
                Err(_) => None,
            }
        }
    });

    rsx! {
        match &*designations.read() {
            Some(Some(list)) if !list.is_empty() => rsx! {
                div { class: "charges-list",
                    for bop in list.iter() {
                        BopCard { designation: bop.clone() }
                    }
                }
            },
            Some(_) => rsx! {
                Card {
                    CardContent {
                        p { class: "text-muted", "No BOP designations recorded for this sentencing." }
                    }
                }
            },
            None => rsx! { Skeleton {} },
        }
    }
}

/// Individual BOP designation card.
#[component]
fn BopCard(designation: BopDesignationResponse) -> Element {
    let security_variant = security_level_variant(&designation.security_level);
    let designation_date = designation
        .designation_date
        .chars()
        .take(10)
        .collect::<String>();
    let reason_display = designation
        .designation_reason
        .clone()
        .unwrap_or_else(|| "--".to_string());

    rsx! {
        Card {
            CardHeader {
                CardTitle { "{designation.facility}" }
            }
            CardContent {
                DetailList {
                    DetailItem { label: "Security Level",
                        Badge { variant: security_variant, "{designation.security_level}" }
                    }
                    DetailItem { label: "Designation Date", value: designation_date }
                    DetailItem { label: "Reason", value: reason_display }
                    DetailItem { label: "RDAP Eligible",
                        Badge {
                            variant: if designation.rdap_eligible { BadgeVariant::Primary } else { BadgeVariant::Outline },
                            if designation.rdap_eligible { "Yes" } else { "No" }
                        }
                    }
                    DetailItem { label: "RDAP Enrolled",
                        Badge {
                            variant: if designation.rdap_enrolled { BadgeVariant::Primary } else { BadgeVariant::Outline },
                            if designation.rdap_enrolled { "Yes" } else { "No" }
                        }
                    }
                }
            }
        }
    }
}

/// Prior sentences tab listing criminal history entries.
#[component]
fn PriorSentencesTab(sentencing_id: String) -> Element {
    let ctx = use_context::<CourtContext>();

    let prior = use_resource(move || {
        let court = ctx.court_id.read().clone();
        let sid = sentencing_id.clone();
        async move {
            match server::api::list_prior_sentences(court, sid).await {
                Ok(json) => serde_json::from_str::<Vec<PriorSentenceResponse>>(&json).ok(),
                Err(_) => None,
            }
        }
    });

    rsx! {
        match &*prior.read() {
            Some(Some(list)) if !list.is_empty() => rsx! {
                div { class: "charges-list",
                    for sentence in list.iter() {
                        PriorSentenceCard { prior: sentence.clone() }
                    }
                }
            },
            Some(_) => rsx! {
                Card {
                    CardContent {
                        p { class: "text-muted", "No prior sentences recorded for this sentencing." }
                    }
                }
            },
            None => rsx! { Skeleton {} },
        }
    }
}

/// Individual prior sentence card.
#[component]
fn PriorSentenceCard(prior: PriorSentenceResponse) -> Element {
    let case_number_display = prior
        .prior_case_number
        .clone()
        .unwrap_or_else(|| "--".to_string());
    let sentence_length_display = prior
        .sentence_length_months
        .map(|m| format!("{} months", m))
        .unwrap_or_else(|| "--".to_string());
    let conviction_date = prior
        .conviction_date
        .chars()
        .take(10)
        .collect::<String>();

    rsx! {
        Card {
            CardHeader {
                CardTitle { "{prior.offense}" }
            }
            CardContent {
                DetailList {
                    DetailItem { label: "Jurisdiction", value: prior.jurisdiction.clone() }
                    DetailItem { label: "Prior Case Number", value: case_number_display }
                    DetailItem { label: "Conviction Date", value: conviction_date }
                    DetailItem { label: "Sentence Length", value: sentence_length_display }
                    DetailItem { label: "Points Assigned",
                        Badge { variant: BadgeVariant::Outline, "{prior.points_assigned}" }
                    }
                }
            }
        }
    }
}

/// Map departure/variance type to a badge variant.
fn departure_badge_variant(departure: &str) -> BadgeVariant {
    match departure {
        "Upward" => BadgeVariant::Destructive,
        "Downward" => BadgeVariant::Primary,
        "None" => BadgeVariant::Secondary,
        _ => BadgeVariant::Outline,
    }
}

/// Map special condition status to a badge variant.
fn condition_status_variant(status: &str) -> BadgeVariant {
    match status {
        "Active" => BadgeVariant::Primary,
        "Modified" => BadgeVariant::Secondary,
        "Terminated" => BadgeVariant::Destructive,
        "Expired" => BadgeVariant::Outline,
        _ => BadgeVariant::Outline,
    }
}

/// Map BOP security level to a badge variant.
fn security_level_variant(level: &str) -> BadgeVariant {
    match level {
        "High" => BadgeVariant::Destructive,
        "Medium" => BadgeVariant::Primary,
        "Low" => BadgeVariant::Secondary,
        "Minimum" => BadgeVariant::Outline,
        "Administrative" => BadgeVariant::Secondary,
        _ => BadgeVariant::Outline,
    }
}

/// Display an optional i32 as a string, defaulting to "--".
fn display_opt_i32(val: Option<i32>) -> String {
    val.map(|v| v.to_string())
        .unwrap_or_else(|| "--".to_string())
}

/// Display an optional f64 as a dollar amount, defaulting to "--".
fn display_opt_money(val: Option<f64>) -> String {
    val.map(|v| format!("${:.2}", v))
        .unwrap_or_else(|| "--".to_string())
}
