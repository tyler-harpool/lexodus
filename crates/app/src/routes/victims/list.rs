use dioxus::prelude::*;
use shared_types::{
    CaseSearchResponse, PaginatedResponse, PaginationMeta, VictimResponse, VICTIM_TYPES,
};
use shared_ui::components::{
    Badge, BadgeVariant, Button, ButtonVariant, Card, CardContent, DataTable, DataTableBody,
    DataTableCell, DataTableColumn, DataTableHeader, DataTableRow, Form, Input, PageActions,
    PageHeader, PageTitle, SearchBar, Separator, Sheet, SheetClose, SheetContent, SheetDescription,
    SheetFooter, SheetHeader, SheetSide, SheetTitle, Skeleton,
};
use shared_ui::{use_toast, HoverCard, HoverCardContent, HoverCardTrigger, ToastOptions};

use crate::routes::Route;
use crate::CourtContext;

#[component]
pub fn VictimListPage() -> Element {
    let ctx = use_context::<CourtContext>();
    let toast = use_toast();

    let mut page = use_signal(|| 1i64);
    let mut search_query = use_signal(String::new);
    let mut search_input = use_signal(String::new);

    // Sheet state for creating a victim
    let mut show_sheet = use_signal(|| false);
    let mut form_name = use_signal(String::new);
    let mut form_case_id = use_signal(String::new);
    let mut form_victim_type = use_signal(|| "Individual".to_string());
    let mut form_email = use_signal(String::new);
    let mut form_phone = use_signal(String::new);
    let mut form_mail = use_signal(|| false);

    // Load available cases for the case selector in the create form
    let cases_for_select = use_resource(move || {
        let court = ctx.court_id.read().clone();
        async move {
            match server::api::search_cases(court, None, None, None, None, None, Some(100)).await {
                Ok(json) => serde_json::from_str::<CaseSearchResponse>(&json)
                    .ok()
                    .map(|r| r.cases),
                Err(_) => None,
            }
        }
    });

    // Load victims data
    let mut data = use_resource(move || {
        let court = ctx.court_id.read().clone();
        let q = search_query.read().clone();
        let p = *page.read();
        async move {
            let search = if q.is_empty() { None } else { Some(q) };
            match server::api::list_all_victims(court, search, Some(p), Some(20)).await {
                Ok(json) => {
                    serde_json::from_str::<PaginatedResponse<VictimResponse>>(&json).ok()
                }
                Err(_) => None,
            }
        }
    });

    let mut reset_form = move || {
        form_name.set(String::new());
        form_case_id.set(String::new());
        form_victim_type.set("Individual".to_string());
        form_email.set(String::new());
        form_phone.set(String::new());
        form_mail.set(false);
    };

    let open_create = move |_| {
        reset_form();
        show_sheet.set(true);
    };

    let handle_search = move |_| {
        search_query.set(search_input.read().clone());
        page.set(1);
    };

    let handle_clear = move |_| {
        search_input.set(String::new());
        search_query.set(String::new());
        page.set(1);
    };

    let handle_save = move |_: FormEvent| {
        let court = ctx.court_id.read().clone();
        let case_id_str = form_case_id.read().clone();

        if form_name.read().trim().is_empty() {
            toast.error("Name is required.".to_string(), ToastOptions::new());
            return;
        }
        if case_id_str.is_empty() {
            toast.error("Case is required.".to_string(), ToastOptions::new());
            return;
        }

        let body = serde_json::json!({
            "case_id": case_id_str,
            "name": form_name.read().clone(),
            "victim_type": form_victim_type.read().clone(),
            "notification_email": opt_str(&form_email.read()),
            "notification_phone": opt_str(&form_phone.read()),
            "notification_mail": *form_mail.read(),
        });

        spawn(async move {
            match server::api::create_victim(court, body.to_string()).await {
                Ok(_) => {
                    data.restart();
                    show_sheet.set(false);
                    toast.success(
                        "Victim record created successfully".to_string(),
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
                PageTitle { "Victims" }
                PageActions {
                    Button {
                        variant: ButtonVariant::Primary,
                        onclick: open_create,
                        "New Victim"
                    }
                }
            }

            SearchBar {
                Input {
                    value: search_input.read().clone(),
                    placeholder: "Search by victim name...",
                    label: "",
                    on_input: move |evt: FormEvent| search_input.set(evt.value().to_string()),
                }
                Button { onclick: handle_search, "Search" }
                if !search_query.read().is_empty() {
                    Button {
                        variant: ButtonVariant::Secondary,
                        onclick: handle_clear,
                        "Clear"
                    }
                }
            }

            match &*data.read() {
                Some(Some(resp)) => rsx! {
                    VictimTable { victims: resp.data.clone() }
                    PaginationControls { meta: resp.meta.clone(), page: page }
                },
                Some(None) => rsx! {
                    Card {
                        CardContent {
                            p { "No victims found for this court district." }
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

            // Create victim Sheet
            Sheet {
                open: show_sheet(),
                on_close: move |_| show_sheet.set(false),
                side: SheetSide::Right,
                SheetContent {
                    SheetHeader {
                        SheetTitle { "New Victim" }
                        SheetDescription {
                            "Add a victim record to an existing case."
                        }
                        SheetClose { on_close: move |_| show_sheet.set(false) }
                    }

                    Form {
                        onsubmit: handle_save,

                        div {
                            class: "sheet-form",

                            // Case selector
                            label { class: "input-label", "Case *" }
                            select {
                                class: "input",
                                value: form_case_id.read().clone(),
                                onchange: move |e: FormEvent| form_case_id.set(e.value().to_string()),
                                option { value: "", "-- Select a case --" }
                                {match &*cases_for_select.read() {
                                    Some(Some(cases)) => rsx! {
                                        for c in cases.iter() {
                                            option {
                                                value: "{c.id}",
                                                "{c.case_number} — {c.title}"
                                            }
                                        }
                                    },
                                    _ => rsx! {
                                        option { value: "", disabled: true, "Loading cases..." }
                                    },
                                }}
                            }

                            Input {
                                label: "Name *",
                                value: form_name.read().clone(),
                                on_input: move |e: FormEvent| form_name.set(e.value().to_string()),
                                placeholder: "e.g., Jane Smith",
                            }

                            // Victim type selector
                            label { class: "input-label", "Victim Type" }
                            select {
                                class: "input",
                                value: form_victim_type.read().clone(),
                                onchange: move |e: FormEvent| form_victim_type.set(e.value().to_string()),
                                for vtype in VICTIM_TYPES.iter() {
                                    option { value: *vtype, "{vtype}" }
                                }
                            }

                            Input {
                                label: "Email",
                                input_type: "email",
                                value: form_email.read().clone(),
                                on_input: move |e: FormEvent| form_email.set(e.value().to_string()),
                                placeholder: "victim@example.com",
                            }

                            Input {
                                label: "Phone",
                                input_type: "tel",
                                value: form_phone.read().clone(),
                                on_input: move |e: FormEvent| form_phone.set(e.value().to_string()),
                                placeholder: "(555) 123-4567",
                            }

                            // Notification by mail checkbox
                            label { class: "input-label checkbox-label",
                                input {
                                    r#type: "checkbox",
                                    checked: *form_mail.read(),
                                    onchange: move |e: FormEvent| {
                                        form_mail.set(e.value().parse::<bool>().unwrap_or(false));
                                    },
                                }
                                " Notify by postal mail"
                            }
                        }

                        Separator {}

                        SheetFooter {
                            div {
                                class: "sheet-footer-actions",
                                SheetClose { on_close: move |_| show_sheet.set(false) }
                                Button {
                                    variant: ButtonVariant::Primary,
                                    "Create Victim"
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
fn VictimTable(victims: Vec<VictimResponse>) -> Element {
    if victims.is_empty() {
        return rsx! {
            Card {
                CardContent {
                    p { "No victims found for this court district." }
                }
            }
        };
    }

    rsx! {
        DataTable {
            DataTableHeader {
                DataTableColumn { "Name" }
                DataTableColumn { "Type" }
                DataTableColumn { "Case" }
                DataTableColumn { "Notification Prefs" }
            }
            DataTableBody {
                for victim in victims {
                    VictimRow { victim: victim }
                }
            }
        }
    }
}

#[component]
fn VictimRow(victim: VictimResponse) -> Element {
    let id = victim.id.clone();
    let type_variant = victim_type_badge_variant(&victim.victim_type);
    let case_id_short = if victim.case_id.len() > 8 {
        format!("{}...", &victim.case_id[..8])
    } else {
        victim.case_id.clone()
    };

    let notification_prefs = build_notification_summary(
        &victim.notification_email,
        &victim.notification_phone,
        victim.notification_mail,
    );

    let email_display = victim
        .notification_email
        .clone()
        .unwrap_or_else(|| "--".to_string());
    let phone_display = victim
        .notification_phone
        .clone()
        .unwrap_or_else(|| "--".to_string());
    let mail_display = if victim.notification_mail { "Yes" } else { "No" };

    rsx! {
        DataTableRow {
            onclick: move |_| {
                let nav = navigator();
                nav.push(Route::VictimDetail { id: id.clone() });
            },
            DataTableCell {
                HoverCard {
                    HoverCardTrigger {
                        span { class: "attorney-name-link", "{victim.name}" }
                    }
                    HoverCardContent {
                        div { class: "hover-card-body",
                            div { class: "hover-card-details",
                                span { class: "hover-card-name", "{victim.name}" }
                                span { class: "hover-card-id", "Type: {victim.victim_type}" }
                                span { class: "hover-card-id", "Email: {email_display}" }
                                span { class: "hover-card-id", "Phone: {phone_display}" }
                                span { class: "hover-card-id", "Mail: {mail_display}" }
                            }
                        }
                    }
                }
            }
            DataTableCell {
                Badge { variant: type_variant, "{victim.victim_type}" }
            }
            DataTableCell { "{case_id_short}" }
            DataTableCell { "{notification_prefs}" }
        }
    }
}

#[component]
fn PaginationControls(meta: PaginationMeta, page: Signal<i64>) -> Element {
    rsx! {
        div { class: "pagination",
            if meta.has_prev {
                Button {
                    variant: ButtonVariant::Outline,
                    onclick: move |_| {
                        let current = *page.read();
                        page.set(current - 1);
                    },
                    "Previous"
                }
            }
            span { class: "pagination-info",
                "Page {meta.page} of {meta.total_pages} ({meta.total} total)"
            }
            if meta.has_next {
                Button {
                    variant: ButtonVariant::Outline,
                    onclick: move |_| {
                        let current = *page.read();
                        page.set(current + 1);
                    },
                    "Next"
                }
            }
        }
    }
}

/// Map victim type to an appropriate badge variant.
fn victim_type_badge_variant(victim_type: &str) -> BadgeVariant {
    match victim_type {
        "Individual" => BadgeVariant::Primary,
        "Organization" => BadgeVariant::Secondary,
        "Government" => BadgeVariant::Outline,
        "Minor" => BadgeVariant::Destructive,
        "Deceased" => BadgeVariant::Destructive,
        "Anonymous" => BadgeVariant::Outline,
        _ => BadgeVariant::Secondary,
    }
}

/// Build a short summary string of notification preferences.
fn build_notification_summary(
    email: &Option<String>,
    phone: &Option<String>,
    mail: bool,
) -> String {
    let mut parts = Vec::new();
    if email.is_some() {
        parts.push("Email");
    }
    if phone.is_some() {
        parts.push("Phone");
    }
    if mail {
        parts.push("Mail");
    }
    if parts.is_empty() {
        "None".to_string()
    } else {
        parts.join(", ")
    }
}

/// Return `serde_json::Value::Null` for empty strings, or the string value.
fn opt_str(s: &str) -> serde_json::Value {
    if s.trim().is_empty() {
        serde_json::Value::Null
    } else {
        serde_json::Value::String(s.to_string())
    }
}
