use dioxus::prelude::*;
use shared_types::{CaseResponse, CaseSearchResponse};
use shared_ui::components::{
    Badge, BadgeVariant, Button, ButtonVariant, Card, CardContent, DataTable, DataTableBody,
    DataTableCell, DataTableColumn, DataTableHeader, DataTableRow, Form, FormSelect, Input,
    PageActions, PageHeader, PageTitle, Pagination, SearchBar, Separator, Sheet, SheetClose,
    SheetContent, SheetDescription, SheetFooter, SheetHeader, SheetSide, SheetTitle, Skeleton,
    Textarea,
};
use shared_ui::{use_toast, HoverCard, HoverCardContent, HoverCardTrigger, ToastOptions};

use crate::routes::Route;
use crate::CourtContext;

#[component]
pub fn CaseListPage() -> Element {
    let ctx = use_context::<CourtContext>();
    let toast = use_toast();

    let mut offset = use_signal(|| 0i64);
    let mut filter_status = use_signal(String::new);
    let mut filter_crime_type = use_signal(String::new);
    let mut search_query = use_signal(String::new);
    let limit: i64 = 20;

    // Sheet state for creating cases
    let mut show_sheet = use_signal(|| false);
    let mut form_title = use_signal(String::new);
    let mut form_description = use_signal(String::new);
    let mut form_crime_type = use_signal(|| "fraud".to_string());
    let mut form_priority = use_signal(|| "medium".to_string());
    let mut form_district_code = use_signal(String::new);
    let mut form_location = use_signal(String::new);

    let mut data = use_resource(move || {
        let court = ctx.court_id.read().clone();
        let st = filter_status.read().clone();
        let ct = filter_crime_type.read().clone();
        let q = search_query.read().clone();
        let off = *offset.read();
        async move {
            let result = server::api::search_cases(
                court,
                if st.is_empty() { None } else { Some(st) },
                if ct.is_empty() { None } else { Some(ct) },
                None, // priority
                if q.is_empty() { None } else { Some(q) },
                Some(off),
                Some(limit),
            )
            .await;

            match result {
                Ok(json) => serde_json::from_str::<CaseSearchResponse>(&json).ok(),
                Err(_) => None,
            }
        }
    });

    let mut reset_form = move || {
        form_title.set(String::new());
        form_description.set(String::new());
        form_crime_type.set("fraud".to_string());
        form_priority.set("medium".to_string());
        form_district_code.set(String::new());
        form_location.set(String::new());
    };

    let open_create = move |_| {
        reset_form();
        show_sheet.set(true);
    };

    let handle_clear = move |_| {
        filter_status.set(String::new());
        filter_crime_type.set(String::new());
        search_query.set(String::new());
        offset.set(0);
    };

    let has_filters = !filter_status.read().is_empty()
        || !filter_crime_type.read().is_empty()
        || !search_query.read().is_empty();

    let handle_save = move |_: FormEvent| {
        let court = ctx.court_id.read().clone();
        let t = form_title.read().clone();
        let d = form_description.read().clone();
        let ct = form_crime_type.read().clone();
        let p = form_priority.read().clone();
        let dc = form_district_code.read().clone();
        let loc = form_location.read().clone();

        spawn(async move {
            if t.trim().is_empty() {
                toast.error("Title is required.".to_string(), ToastOptions::new());
                return;
            }

            let body = serde_json::json!({
                "title": t.trim(),
                "description": d.trim(),
                "crime_type": ct,
                "priority": p,
                "district_code": if dc.is_empty() { court.clone() } else { dc },
                "location": loc.trim(),
            });

            match server::api::create_case(court, body.to_string()).await {
                Ok(_) => {
                    data.restart();
                    show_sheet.set(false);
                    toast.success(
                        "Case created successfully".to_string(),
                        ToastOptions::new(),
                    );
                }
                Err(e) => {
                    toast.error(format!("{}", e), ToastOptions::new());
                }
            }
        });
    };

    rsx! {
        div { class: "container",
            PageHeader {
                PageTitle { "Cases" }
                PageActions {
                    Button {
                        variant: ButtonVariant::Primary,
                        onclick: open_create,
                        "New Case"
                    }
                }
            }

            SearchBar {
                Input {
                    value: search_query.read().clone(),
                    placeholder: "Search by title or case number...",
                    label: "",
                    on_input: move |evt: FormEvent| {
                        search_query.set(evt.value().to_string());
                        offset.set(0);
                    },
                }
                FormSelect {
                    value: "{filter_status}",
                    onchange: move |evt: Event<FormData>| {
                        filter_status.set(evt.value().to_string());
                        offset.set(0);
                    },
                    option { value: "", "All Statuses" }
                    option { value: "filed", "Filed" }
                    option { value: "arraigned", "Arraigned" }
                    option { value: "discovery", "Discovery" }
                    option { value: "pretrial_motions", "Pretrial Motions" }
                    option { value: "plea_negotiations", "Plea Negotiations" }
                    option { value: "trial_ready", "Trial Ready" }
                    option { value: "in_trial", "In Trial" }
                    option { value: "awaiting_sentencing", "Awaiting Sentencing" }
                    option { value: "sentenced", "Sentenced" }
                    option { value: "dismissed", "Dismissed" }
                    option { value: "on_appeal", "On Appeal" }
                }
                FormSelect {
                    value: "{filter_crime_type}",
                    onchange: move |evt: Event<FormData>| {
                        filter_crime_type.set(evt.value().to_string());
                        offset.set(0);
                    },
                    option { value: "", "All Crime Types" }
                    option { value: "fraud", "Fraud" }
                    option { value: "drug_offense", "Drug Offense" }
                    option { value: "racketeering", "Racketeering" }
                    option { value: "cybercrime", "Cybercrime" }
                    option { value: "tax_offense", "Tax Offense" }
                    option { value: "money_laundering", "Money Laundering" }
                    option { value: "immigration", "Immigration" }
                    option { value: "firearms", "Firearms" }
                    option { value: "other", "Other" }
                }
                if has_filters {
                    Button {
                        variant: ButtonVariant::Secondary,
                        onclick: handle_clear,
                        "Clear Filters"
                    }
                }
            }

            match &*data.read() {
                Some(Some(resp)) => rsx! {
                    CaseTable { cases: resp.cases.clone() }
                    Pagination {
                        total: resp.total,
                        offset: offset,
                        limit: limit,
                    }
                },
                Some(None) => rsx! {
                    Card {
                        CardContent {
                            p { "No cases found for this court district." }
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

            // Create case Sheet
            Sheet {
                open: show_sheet(),
                on_close: move |_| show_sheet.set(false),
                side: SheetSide::Right,
                SheetContent {
                    SheetHeader {
                        SheetTitle { "New Case" }
                        SheetDescription {
                            "Fill in the details to file a new case."
                        }
                        SheetClose { on_close: move |_| show_sheet.set(false) }
                    }

                    Form {
                        onsubmit: handle_save,

                        div {
                            class: "sheet-form",

                            Input {
                                label: "Title *",
                                value: form_title.read().clone(),
                                on_input: move |e: FormEvent| form_title.set(e.value().to_string()),
                                placeholder: "e.g., United States v. Smith",
                            }

                            Textarea {
                                label: "Description",
                                value: form_description.read().clone(),
                                on_input: move |e: FormEvent| form_description.set(e.value().to_string()),
                                placeholder: "Case description...",
                            }

                            FormSelect {
                                label: "Crime Type *",
                                value: "{form_crime_type}",
                                onchange: move |evt: Event<FormData>| form_crime_type.set(evt.value().to_string()),
                                option { value: "fraud", "Fraud" }
                                option { value: "drug_offense", "Drug Offense" }
                                option { value: "racketeering", "Racketeering" }
                                option { value: "cybercrime", "Cybercrime" }
                                option { value: "tax_offense", "Tax Offense" }
                                option { value: "money_laundering", "Money Laundering" }
                                option { value: "immigration", "Immigration" }
                                option { value: "firearms", "Firearms" }
                                option { value: "other", "Other" }
                            }

                            FormSelect {
                                label: "Priority",
                                value: "{form_priority}",
                                onchange: move |evt: Event<FormData>| form_priority.set(evt.value().to_string()),
                                option { value: "low", "Low" }
                                option { value: "medium", "Medium" }
                                option { value: "high", "High" }
                                option { value: "critical", "Critical" }
                            }

                            Input {
                                label: "District Code",
                                value: form_district_code.read().clone(),
                                on_input: move |e: FormEvent| form_district_code.set(e.value().to_string()),
                                placeholder: "Defaults to current court",
                            }

                            Input {
                                label: "Location",
                                value: form_location.read().clone(),
                                on_input: move |e: FormEvent| form_location.set(e.value().to_string()),
                                placeholder: "e.g., Federal Courthouse Room 301",
                            }
                        }

                        Separator {}

                        SheetFooter {
                            div {
                                class: "sheet-footer-actions",
                                SheetClose { on_close: move |_| show_sheet.set(false) }
                                Button {
                                    variant: ButtonVariant::Primary,
                                    "File Case"
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn CaseTable(cases: Vec<CaseResponse>) -> Element {
    if cases.is_empty() {
        return rsx! {
            Card {
                CardContent {
                    p { "No cases found." }
                }
            }
        };
    }

    rsx! {
        DataTable {
            DataTableHeader {
                DataTableColumn { "Case Number" }
                DataTableColumn { "Title" }
                DataTableColumn { "Type" }
                DataTableColumn { "Status" }
                DataTableColumn { "Priority" }
                DataTableColumn { "Opened" }
            }
            DataTableBody {
                for c in cases {
                    CaseRow { case_item: c }
                }
            }
        }
    }
}

#[component]
fn CaseRow(case_item: CaseResponse) -> Element {
    let id = case_item.id.clone();
    let status_variant = status_badge_variant(&case_item.status);
    let priority_variant = priority_badge_variant(&case_item.priority);
    let display_date = format_date(&case_item.opened_at);
    let display_type = format_crime_type(&case_item.crime_type);
    let display_status = format_status(&case_item.status);
    let desc_preview = if case_item.description.len() > 80 {
        format!("{}...", &case_item.description[..80])
    } else if case_item.description.is_empty() {
        "No description".to_string()
    } else {
        case_item.description.clone()
    };

    rsx! {
        DataTableRow {
            onclick: move |_| {
                let nav = navigator();
                nav.push(Route::CaseDetail { id: id.clone() });
            },
            DataTableCell {
                HoverCard {
                    HoverCardTrigger {
                        span { class: "case-number-link", "{case_item.case_number}" }
                    }
                    HoverCardContent {
                        div { class: "hover-card-body",
                            div { class: "hover-card-details",
                                span { class: "hover-card-name", "{case_item.title}" }
                                span { class: "hover-card-username", "{case_item.case_number}" }
                                span { class: "hover-card-id", "{desc_preview}" }
                                div { class: "hover-card-meta",
                                    Badge { variant: status_variant, "{display_status}" }
                                    Badge { variant: priority_variant, "{case_item.priority}" }
                                    if case_item.is_sealed {
                                        Badge { variant: BadgeVariant::Destructive, "SEALED" }
                                    }
                                }
                                if !case_item.location.is_empty() {
                                    span { class: "hover-card-id", "Location: {case_item.location}" }
                                }
                                span { class: "hover-card-id", "District: {case_item.district_code}" }
                            }
                        }
                    }
                }
            }
            DataTableCell { "{case_item.title}" }
            DataTableCell { "{display_type}" }
            DataTableCell {
                Badge { variant: status_variant, "{display_status}" }
            }
            DataTableCell {
                Badge { variant: priority_variant, "{case_item.priority}" }
            }
            DataTableCell { "{display_date}" }
        }
    }
}

fn status_badge_variant(status: &str) -> BadgeVariant {
    match status {
        "filed" => BadgeVariant::Primary,
        "arraigned" | "discovery" | "pretrial_motions" | "plea_negotiations" => {
            BadgeVariant::Secondary
        }
        "trial_ready" | "in_trial" => BadgeVariant::Outline,
        "awaiting_sentencing" | "sentenced" => BadgeVariant::Secondary,
        "dismissed" | "on_appeal" => BadgeVariant::Destructive,
        _ => BadgeVariant::Secondary,
    }
}

fn priority_badge_variant(priority: &str) -> BadgeVariant {
    match priority {
        "low" => BadgeVariant::Secondary,
        "medium" => BadgeVariant::Outline,
        "high" => BadgeVariant::Primary,
        "critical" => BadgeVariant::Destructive,
        _ => BadgeVariant::Secondary,
    }
}

fn format_date(date_str: &str) -> String {
    if date_str.len() >= 10 {
        date_str[..10].to_string()
    } else {
        date_str.to_string()
    }
}

fn format_crime_type(ct: &str) -> String {
    ct.replace('_', " ")
}

fn format_status(s: &str) -> String {
    s.replace('_', " ")
}
