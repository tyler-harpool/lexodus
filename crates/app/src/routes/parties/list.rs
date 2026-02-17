use dioxus::prelude::*;
use shared_types::{
    CaseSearchResponse, PaginatedResponse, PaginationMeta, PartyResponse,
    VALID_ENTITY_TYPES, VALID_PARTY_TYPES,
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
pub fn PartyListPage() -> Element {
    let ctx = use_context::<CourtContext>();
    let toast = use_toast();

    let mut page = use_signal(|| 1i64);
    let mut search_query = use_signal(String::new);
    let mut search_input = use_signal(String::new);

    // Sheet state for creating a party
    let mut show_sheet = use_signal(|| false);
    let mut form_name = use_signal(String::new);
    let mut form_case_id = use_signal(String::new);
    let mut form_party_type = use_signal(|| "Defendant".to_string());
    let mut form_entity_type = use_signal(|| "Individual".to_string());
    let mut form_email = use_signal(String::new);
    let mut form_phone = use_signal(String::new);
    let mut form_first_name = use_signal(String::new);
    let mut form_last_name = use_signal(String::new);

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

    // Load parties data
    let mut data = use_resource(move || {
        let court = ctx.court_id.read().clone();
        let q = search_query.read().clone();
        let p = *page.read();
        async move {
            let search = if q.is_empty() { None } else { Some(q) };
            match server::api::list_all_parties(court, search, Some(p), Some(20)).await {
                Ok(json) => {
                    serde_json::from_str::<PaginatedResponse<PartyResponse>>(&json).ok()
                }
                Err(_) => None,
            }
        }
    });

    let mut reset_form = move || {
        form_name.set(String::new());
        form_case_id.set(String::new());
        form_party_type.set("Defendant".to_string());
        form_entity_type.set("Individual".to_string());
        form_email.set(String::new());
        form_phone.set(String::new());
        form_first_name.set(String::new());
        form_last_name.set(String::new());
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
            "party_type": form_party_type.read().clone(),
            "entity_type": form_entity_type.read().clone(),
            "first_name": opt_str(&form_first_name.read()),
            "last_name": opt_str(&form_last_name.read()),
            "email": opt_str(&form_email.read()),
            "phone": opt_str(&form_phone.read()),
        });

        spawn(async move {
            match server::api::create_party(court, body.to_string()).await {
                Ok(_) => {
                    data.restart();
                    show_sheet.set(false);
                    toast.success(
                        "Party created successfully".to_string(),
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
                PageTitle { "Parties" }
                PageActions {
                    Button {
                        variant: ButtonVariant::Primary,
                        onclick: open_create,
                        "New Party"
                    }
                }
            }

            SearchBar {
                Input {
                    value: search_input.read().clone(),
                    placeholder: "Search by party name...",
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
                    PartyTable { parties: resp.data.clone() }
                    PaginationControls { meta: resp.meta.clone(), page: page }
                },
                Some(None) => rsx! {
                    Card {
                        CardContent {
                            p { "No parties found for this court district." }
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

            // Create party Sheet
            Sheet {
                open: show_sheet(),
                on_close: move |_| show_sheet.set(false),
                side: SheetSide::Right,
                SheetContent {
                    SheetHeader {
                        SheetTitle { "New Party" }
                        SheetDescription {
                            "Add a party to an existing case."
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
                                label: "Full Name *",
                                value: form_name.read().clone(),
                                on_input: move |e: FormEvent| form_name.set(e.value().to_string()),
                                placeholder: "e.g., John Doe",
                            }

                            Input {
                                label: "First Name",
                                value: form_first_name.read().clone(),
                                on_input: move |e: FormEvent| form_first_name.set(e.value().to_string()),
                            }

                            Input {
                                label: "Last Name",
                                value: form_last_name.read().clone(),
                                on_input: move |e: FormEvent| form_last_name.set(e.value().to_string()),
                            }

                            label { class: "input-label", "Party Type" }
                            select {
                                class: "input",
                                value: form_party_type.read().clone(),
                                onchange: move |e: FormEvent| form_party_type.set(e.value().to_string()),
                                for pt in VALID_PARTY_TYPES.iter() {
                                    option { value: *pt, "{pt}" }
                                }
                            }

                            label { class: "input-label", "Entity Type" }
                            select {
                                class: "input",
                                value: form_entity_type.read().clone(),
                                onchange: move |e: FormEvent| form_entity_type.set(e.value().to_string()),
                                for et in VALID_ENTITY_TYPES.iter() {
                                    option { value: *et, "{et}" }
                                }
                            }

                            Input {
                                label: "Email",
                                value: form_email.read().clone(),
                                on_input: move |e: FormEvent| form_email.set(e.value().to_string()),
                                placeholder: "e.g., john@example.com",
                            }

                            Input {
                                label: "Phone",
                                value: form_phone.read().clone(),
                                on_input: move |e: FormEvent| form_phone.set(e.value().to_string()),
                                placeholder: "e.g., (555) 123-4567",
                            }
                        }

                        Separator {}

                        SheetFooter {
                            div {
                                class: "sheet-footer-actions",
                                SheetClose { on_close: move |_| show_sheet.set(false) }
                                Button {
                                    variant: ButtonVariant::Primary,
                                    "Create Party"
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
fn PartyTable(parties: Vec<PartyResponse>) -> Element {
    if parties.is_empty() {
        return rsx! {
            Card {
                CardContent {
                    p { "No parties found for this court district." }
                }
            }
        };
    }

    rsx! {
        DataTable {
            DataTableHeader {
                DataTableColumn { "Name" }
                DataTableColumn { "Type" }
                DataTableColumn { "Entity Type" }
                DataTableColumn { "Case" }
                DataTableColumn { "Status" }
            }
            DataTableBody {
                for party in parties {
                    PartyRow { party: party }
                }
            }
        }
    }
}

#[component]
fn PartyRow(party: PartyResponse) -> Element {
    let id = party.id.clone();
    let status_variant = party_status_badge_variant(&party.status);
    let case_id_short = if party.case_id.len() > 8 {
        format!("{}...", &party.case_id[..8])
    } else {
        party.case_id.clone()
    };
    let email_display = party
        .email
        .clone()
        .unwrap_or_else(|| "--".to_string());
    let phone_display = party
        .phone
        .clone()
        .unwrap_or_else(|| "--".to_string());

    rsx! {
        DataTableRow {
            onclick: move |_| {
                let nav = navigator();
                nav.push(Route::PartyDetail { id: id.clone() });
            },
            DataTableCell {
                HoverCard {
                    HoverCardTrigger {
                        span { class: "attorney-name-link", "{party.name}" }
                    }
                    HoverCardContent {
                        div { class: "hover-card-body",
                            div { class: "hover-card-details",
                                span { class: "hover-card-name", "{party.name}" }
                                span { class: "hover-card-id", "Type: {party.party_type}" }
                                span { class: "hover-card-id", "Entity: {party.entity_type}" }
                                span { class: "hover-card-id", "Email: {email_display}" }
                                span { class: "hover-card-id", "Phone: {phone_display}" }
                                div { class: "hover-card-meta",
                                    Badge { variant: status_variant,
                                        "{party.status}"
                                    }
                                    if party.pro_se {
                                        Badge { variant: BadgeVariant::Outline,
                                            "Pro Se"
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
            DataTableCell { "{party.party_type}" }
            DataTableCell { "{party.entity_type}" }
            DataTableCell { "{case_id_short}" }
            DataTableCell {
                Badge { variant: status_variant, "{party.status}" }
            }
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

/// Map party status to an appropriate badge variant.
fn party_status_badge_variant(status: &str) -> BadgeVariant {
    match status {
        "Active" => BadgeVariant::Primary,
        "Terminated" | "Dismissed" | "Deceased" => BadgeVariant::Destructive,
        "Defaulted" | "In Contempt" => BadgeVariant::Destructive,
        "Settled" => BadgeVariant::Secondary,
        _ => BadgeVariant::Outline,
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
