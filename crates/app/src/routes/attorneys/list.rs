use dioxus::prelude::*;
use shared_types::{AttorneyResponse, PaginatedResponse, PaginationMeta};
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
pub fn AttorneyListPage() -> Element {
    let ctx = use_context::<CourtContext>();
    let toast = use_toast();

    let mut page = use_signal(|| 1i64);
    let mut search_query = use_signal(String::new);
    let mut search_input = use_signal(String::new);

    // Sheet state for creating attorneys
    let mut show_sheet = use_signal(|| false);
    let mut form_bar_number = use_signal(String::new);
    let mut form_first_name = use_signal(String::new);
    let mut form_last_name = use_signal(String::new);
    let mut form_middle_name = use_signal(String::new);
    let mut form_email = use_signal(String::new);
    let mut form_phone = use_signal(String::new);
    let mut form_firm_name = use_signal(String::new);
    let mut form_fax = use_signal(String::new);
    let mut form_street1 = use_signal(String::new);
    let mut form_street2 = use_signal(String::new);
    let mut form_city = use_signal(String::new);
    let mut form_state = use_signal(String::new);
    let mut form_zip_code = use_signal(String::new);
    let mut form_country = use_signal(|| "US".to_string());

    let mut data = use_resource(move || {
        let court = ctx.court_id.read().clone();
        let q = search_query.read().clone();
        let p = *page.read();
        async move {
            let result = if q.is_empty() {
                server::api::list_attorneys(court, Some(p), Some(20)).await
            } else {
                server::api::search_attorneys(court, q, Some(p), Some(20)).await
            };

            match result {
                Ok(json) => {
                    serde_json::from_str::<PaginatedResponse<AttorneyResponse>>(&json).ok()
                }
                Err(_) => None,
            }
        }
    });

    let mut reset_form = move || {
        form_bar_number.set(String::new());
        form_first_name.set(String::new());
        form_last_name.set(String::new());
        form_middle_name.set(String::new());
        form_email.set(String::new());
        form_phone.set(String::new());
        form_firm_name.set(String::new());
        form_fax.set(String::new());
        form_street1.set(String::new());
        form_street2.set(String::new());
        form_city.set(String::new());
        form_state.set(String::new());
        form_zip_code.set(String::new());
        form_country.set("US".to_string());
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

        let body = serde_json::json!({
            "bar_number": form_bar_number.read().clone(),
            "first_name": form_first_name.read().clone(),
            "last_name": form_last_name.read().clone(),
            "middle_name": opt_str(&form_middle_name.read()),
            "firm_name": opt_str(&form_firm_name.read()),
            "email": form_email.read().clone(),
            "phone": form_phone.read().clone(),
            "fax": opt_str(&form_fax.read()),
            "address": {
                "street1": form_street1.read().clone(),
                "street2": opt_str(&form_street2.read()),
                "city": form_city.read().clone(),
                "state": form_state.read().clone(),
                "zip_code": form_zip_code.read().clone(),
                "country": form_country.read().clone(),
            }
        });

        spawn(async move {
            match server::api::create_attorney(court, body.to_string()).await {
                Ok(_) => {
                    data.restart();
                    show_sheet.set(false);
                    toast.success(
                        "Attorney created successfully".to_string(),
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
                PageTitle { "Attorneys" }
                PageActions {
                    Button {
                        variant: ButtonVariant::Primary,
                        onclick: open_create,
                        "New Attorney"
                    }
                }
            }

            SearchBar {
                Input {
                    value: search_input.read().clone(),
                    placeholder: "Search by name, bar number, email, or firm...",
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
                    AttorneyTable { attorneys: resp.data.clone() }
                    PaginationControls { meta: resp.meta.clone(), page: page }
                },
                Some(None) => rsx! {
                    Card {
                        CardContent {
                            p { "No attorneys found for this court district." }
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

            // Create attorney Sheet
            Sheet {
                open: show_sheet(),
                on_close: move |_| show_sheet.set(false),
                side: SheetSide::Right,
                SheetContent {
                    SheetHeader {
                        SheetTitle { "New Attorney" }
                        SheetDescription {
                            "Register a new attorney in the court system."
                        }
                        SheetClose { on_close: move |_| show_sheet.set(false) }
                    }

                    Form {
                        onsubmit: handle_save,

                        div {
                            class: "sheet-form",

                            Input {
                                label: "Bar Number *",
                                value: form_bar_number.read().clone(),
                                on_input: move |e: FormEvent| form_bar_number.set(e.value().to_string()),
                                placeholder: "e.g., NY12345",
                            }

                            Input {
                                label: "First Name *",
                                value: form_first_name.read().clone(),
                                on_input: move |e: FormEvent| form_first_name.set(e.value().to_string()),
                            }

                            Input {
                                label: "Last Name *",
                                value: form_last_name.read().clone(),
                                on_input: move |e: FormEvent| form_last_name.set(e.value().to_string()),
                            }

                            Input {
                                label: "Middle Name",
                                value: form_middle_name.read().clone(),
                                on_input: move |e: FormEvent| form_middle_name.set(e.value().to_string()),
                            }

                            Input {
                                label: "Email *",
                                input_type: "email",
                                value: form_email.read().clone(),
                                on_input: move |e: FormEvent| form_email.set(e.value().to_string()),
                            }

                            Input {
                                label: "Phone *",
                                input_type: "tel",
                                value: form_phone.read().clone(),
                                on_input: move |e: FormEvent| form_phone.set(e.value().to_string()),
                            }

                            Input {
                                label: "Firm Name",
                                value: form_firm_name.read().clone(),
                                on_input: move |e: FormEvent| form_firm_name.set(e.value().to_string()),
                            }

                            Input {
                                label: "Fax",
                                input_type: "tel",
                                value: form_fax.read().clone(),
                                on_input: move |e: FormEvent| form_fax.set(e.value().to_string()),
                            }

                            Separator {}

                            Input {
                                label: "Street Address *",
                                value: form_street1.read().clone(),
                                on_input: move |e: FormEvent| form_street1.set(e.value().to_string()),
                            }

                            Input {
                                label: "Street Address 2",
                                value: form_street2.read().clone(),
                                on_input: move |e: FormEvent| form_street2.set(e.value().to_string()),
                            }

                            Input {
                                label: "City *",
                                value: form_city.read().clone(),
                                on_input: move |e: FormEvent| form_city.set(e.value().to_string()),
                            }

                            Input {
                                label: "State *",
                                value: form_state.read().clone(),
                                on_input: move |e: FormEvent| form_state.set(e.value().to_string()),
                            }

                            Input {
                                label: "ZIP Code *",
                                value: form_zip_code.read().clone(),
                                on_input: move |e: FormEvent| form_zip_code.set(e.value().to_string()),
                            }

                            Input {
                                label: "Country *",
                                value: form_country.read().clone(),
                                on_input: move |e: FormEvent| form_country.set(e.value().to_string()),
                            }
                        }

                        Separator {}

                        SheetFooter {
                            div {
                                class: "sheet-footer-actions",
                                SheetClose { on_close: move |_| show_sheet.set(false) }
                                Button {
                                    variant: ButtonVariant::Primary,
                                    "Create Attorney"
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
fn AttorneyTable(attorneys: Vec<AttorneyResponse>) -> Element {
    if attorneys.is_empty() {
        return rsx! {
            Card {
                CardContent {
                    p { "No attorneys found for this court district." }
                }
            }
        };
    }

    rsx! {
        DataTable {
            DataTableHeader {
                DataTableColumn { "Name" }
                DataTableColumn { "Bar Number" }
                DataTableColumn { "Email" }
                DataTableColumn { "Firm" }
                DataTableColumn { "Status" }
            }
            DataTableBody {
                for attorney in attorneys {
                    AttorneyRow { attorney: attorney }
                }
            }
        }
    }
}

#[component]
fn AttorneyRow(attorney: AttorneyResponse) -> Element {
    let id = attorney.id.clone();
    let badge_variant = status_badge_variant(&attorney.status);
    let full_name = format!("{}, {}", attorney.last_name, attorney.first_name);
    let firm_display = attorney.firm_name.clone().unwrap_or_else(|| "--".to_string());
    let city_state = format!("{}, {}", attorney.address.city, attorney.address.state);

    rsx! {
        DataTableRow {
            onclick: move |_| {
                let nav = navigator();
                nav.push(Route::AttorneyDetail { id: id.clone() });
            },
            DataTableCell {
                HoverCard {
                    HoverCardTrigger {
                        span { class: "attorney-name-link", "{full_name}" }
                    }
                    HoverCardContent {
                        div { class: "hover-card-body",
                            div { class: "hover-card-details",
                                span { class: "hover-card-name", "{full_name}" }
                                span { class: "hover-card-username", "Bar: {attorney.bar_number}" }
                                span { class: "hover-card-id", "{attorney.email}" }
                                span { class: "hover-card-id", "{attorney.phone}" }
                                if !firm_display.is_empty() && firm_display != "--" {
                                    span { class: "hover-card-id", "Firm: {firm_display}" }
                                }
                                span { class: "hover-card-id", "{city_state}" }
                                div { class: "hover-card-meta",
                                    Badge { variant: badge_variant, "{attorney.status}" }
                                    if attorney.cja_panel_member {
                                        Badge { variant: BadgeVariant::Outline, "CJA Panel" }
                                    }
                                    if attorney.cases_handled > 0 {
                                        Badge { variant: BadgeVariant::Secondary, "{attorney.cases_handled} cases" }
                                    }
                                }
                            }
                        }
                    }
                }
            }
            DataTableCell { "{attorney.bar_number}" }
            DataTableCell { "{attorney.email}" }
            DataTableCell { "{firm_display}" }
            DataTableCell {
                Badge { variant: badge_variant, "{attorney.status}" }
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

fn status_badge_variant(status: &str) -> BadgeVariant {
    match status {
        "Active" => BadgeVariant::Primary,
        "Inactive" => BadgeVariant::Secondary,
        "Suspended" => BadgeVariant::Destructive,
        "Retired" => BadgeVariant::Outline,
        _ => BadgeVariant::Secondary,
    }
}

fn opt_str(s: &str) -> serde_json::Value {
    if s.trim().is_empty() {
        serde_json::Value::Null
    } else {
        serde_json::Value::String(s.to_string())
    }
}
