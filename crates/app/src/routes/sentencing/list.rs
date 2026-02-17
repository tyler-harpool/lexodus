use dioxus::prelude::*;
use shared_types::{
    CaseSearchResponse, DefendantResponse, PaginatedResponse, PaginationMeta,
    SentencingResponse, CRIMINAL_HISTORY_CATEGORIES, DEPARTURE_TYPES,
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
pub fn SentencingListPage() -> Element {
    let ctx = use_context::<CourtContext>();
    let toast = use_toast();

    let mut page = use_signal(|| 1i64);
    let mut search_query = use_signal(String::new);
    let mut search_input = use_signal(String::new);

    // Sheet state for creating a new sentencing record
    let mut show_sheet = use_signal(|| false);
    let mut form_case_id = use_signal(String::new);
    let mut form_defendant_id = use_signal(String::new);
    let mut form_judge_id = use_signal(String::new);
    let mut form_custody_months = use_signal(String::new);
    let mut form_probation_months = use_signal(String::new);
    let mut form_supervised_release = use_signal(String::new);
    let mut form_fine = use_signal(String::new);
    let mut form_restitution = use_signal(String::new);
    let mut form_special_assessment = use_signal(String::new);
    let mut form_base_offense = use_signal(String::new);
    let mut form_total_offense = use_signal(String::new);
    let mut form_history_category = use_signal(String::new);
    let mut form_departure_type = use_signal(|| "None".to_string());
    let mut form_sentencing_date = use_signal(String::new);

    // Load available cases for the selector
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

    // Load available defendants for the selector
    let defendants_for_select = use_resource(move || {
        let court = ctx.court_id.read().clone();
        async move {
            match server::api::list_all_defendants(court, None, Some(1), Some(100)).await {
                Ok(json) => serde_json::from_str::<PaginatedResponse<DefendantResponse>>(&json)
                    .ok()
                    .map(|r| r.data),
                Err(_) => None,
            }
        }
    });

    // Load available judges for the selector
    let judges_for_select = use_resource(move || {
        let court = ctx.court_id.read().clone();
        async move {
            match server::api::list_judges(court).await {
                Ok(json) => serde_json::from_str::<Vec<shared_types::Judge>>(&json).ok(),
                Err(_) => None,
            }
        }
    });

    // Load sentencing records
    let mut data = use_resource(move || {
        let court = ctx.court_id.read().clone();
        let q = search_query.read().clone();
        let p = *page.read();
        async move {
            let search = if q.is_empty() { None } else { Some(q) };
            match server::api::list_all_sentencing(court, search, Some(p), Some(20)).await {
                Ok(json) => {
                    serde_json::from_str::<PaginatedResponse<SentencingResponse>>(&json).ok()
                }
                Err(_) => None,
            }
        }
    });

    let mut reset_form = move || {
        form_case_id.set(String::new());
        form_defendant_id.set(String::new());
        form_judge_id.set(String::new());
        form_custody_months.set(String::new());
        form_probation_months.set(String::new());
        form_supervised_release.set(String::new());
        form_fine.set(String::new());
        form_restitution.set(String::new());
        form_special_assessment.set(String::new());
        form_base_offense.set(String::new());
        form_total_offense.set(String::new());
        form_history_category.set(String::new());
        form_departure_type.set("None".to_string());
        form_sentencing_date.set(String::new());
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
        let defendant_id_str = form_defendant_id.read().clone();
        let judge_id_str = form_judge_id.read().clone();

        if case_id_str.is_empty() {
            toast.error("Case is required.".to_string(), ToastOptions::new());
            return;
        }
        if defendant_id_str.is_empty() {
            toast.error("Defendant is required.".to_string(), ToastOptions::new());
            return;
        }
        if judge_id_str.is_empty() {
            toast.error("Judge is required.".to_string(), ToastOptions::new());
            return;
        }

        let body = serde_json::json!({
            "case_id": case_id_str,
            "defendant_id": defendant_id_str,
            "judge_id": judge_id_str,
            "custody_months": parse_opt_i32(&form_custody_months.read()),
            "probation_months": parse_opt_i32(&form_probation_months.read()),
            "supervised_release_months": parse_opt_i32(&form_supervised_release.read()),
            "fine_amount": parse_opt_f64(&form_fine.read()),
            "restitution_amount": parse_opt_f64(&form_restitution.read()),
            "special_assessment": parse_opt_f64(&form_special_assessment.read()),
            "base_offense_level": parse_opt_i32(&form_base_offense.read()),
            "total_offense_level": parse_opt_i32(&form_total_offense.read()),
            "criminal_history_category": opt_str(&form_history_category.read()),
            "departure_type": opt_str(&form_departure_type.read()),
            "sentencing_date": opt_date(&form_sentencing_date.read()),
        });

        spawn(async move {
            match server::api::create_sentencing(court, body.to_string()).await {
                Ok(_) => {
                    data.restart();
                    show_sheet.set(false);
                    toast.success(
                        "Sentencing record created successfully".to_string(),
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
                PageTitle { "Sentencing" }
                PageActions {
                    Button {
                        variant: ButtonVariant::Primary,
                        onclick: open_create,
                        "New Sentencing Record"
                    }
                }
            }

            SearchBar {
                Input {
                    value: search_input.read().clone(),
                    placeholder: "Search by case number...",
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
                    SentencingTable { records: resp.data.clone() }
                    PaginationControls { meta: resp.meta.clone(), page: page }
                },
                Some(None) => rsx! {
                    Card {
                        CardContent {
                            p { "No sentencing records found for this court district." }
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

            // Create sentencing Sheet
            Sheet {
                open: show_sheet(),
                on_close: move |_| show_sheet.set(false),
                side: SheetSide::Right,
                SheetContent {
                    SheetHeader {
                        SheetTitle { "New Sentencing Record" }
                        SheetDescription {
                            "Create a sentencing record for a defendant."
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

                            // Defendant selector
                            label { class: "input-label", "Defendant *" }
                            select {
                                class: "input",
                                value: form_defendant_id.read().clone(),
                                onchange: move |e: FormEvent| form_defendant_id.set(e.value().to_string()),
                                option { value: "", "-- Select a defendant --" }
                                {match &*defendants_for_select.read() {
                                    Some(Some(defs)) => rsx! {
                                        for d in defs.iter() {
                                            option {
                                                value: "{d.id}",
                                                "{d.name}"
                                            }
                                        }
                                    },
                                    _ => rsx! {
                                        option { value: "", disabled: true, "Loading defendants..." }
                                    },
                                }}
                            }

                            // Judge selector
                            label { class: "input-label", "Judge *" }
                            select {
                                class: "input",
                                value: form_judge_id.read().clone(),
                                onchange: move |e: FormEvent| form_judge_id.set(e.value().to_string()),
                                option { value: "", "-- Select a judge --" }
                                {match &*judges_for_select.read() {
                                    Some(Some(judges)) => rsx! {
                                        for j in judges.iter() {
                                            option {
                                                value: "{j.id}",
                                                "{j.name} — {j.title}"
                                            }
                                        }
                                    },
                                    _ => rsx! {
                                        option { value: "", disabled: true, "Loading judges..." }
                                    },
                                }}
                            }

                            Input {
                                label: "Sentencing Date",
                                input_type: "date",
                                value: form_sentencing_date.read().clone(),
                                on_input: move |e: FormEvent| form_sentencing_date.set(e.value().to_string()),
                            }

                            Input {
                                label: "Custody Months",
                                input_type: "number",
                                value: form_custody_months.read().clone(),
                                on_input: move |e: FormEvent| form_custody_months.set(e.value().to_string()),
                                placeholder: "e.g., 60",
                            }

                            Input {
                                label: "Probation Months",
                                input_type: "number",
                                value: form_probation_months.read().clone(),
                                on_input: move |e: FormEvent| form_probation_months.set(e.value().to_string()),
                            }

                            Input {
                                label: "Supervised Release Months",
                                input_type: "number",
                                value: form_supervised_release.read().clone(),
                                on_input: move |e: FormEvent| form_supervised_release.set(e.value().to_string()),
                            }

                            Input {
                                label: "Fine Amount ($)",
                                input_type: "number",
                                value: form_fine.read().clone(),
                                on_input: move |e: FormEvent| form_fine.set(e.value().to_string()),
                                placeholder: "e.g., 10000.00",
                            }

                            Input {
                                label: "Restitution Amount ($)",
                                input_type: "number",
                                value: form_restitution.read().clone(),
                                on_input: move |e: FormEvent| form_restitution.set(e.value().to_string()),
                            }

                            Input {
                                label: "Special Assessment ($)",
                                input_type: "number",
                                value: form_special_assessment.read().clone(),
                                on_input: move |e: FormEvent| form_special_assessment.set(e.value().to_string()),
                            }

                            Input {
                                label: "Base Offense Level",
                                input_type: "number",
                                value: form_base_offense.read().clone(),
                                on_input: move |e: FormEvent| form_base_offense.set(e.value().to_string()),
                            }

                            Input {
                                label: "Total Offense Level",
                                input_type: "number",
                                value: form_total_offense.read().clone(),
                                on_input: move |e: FormEvent| form_total_offense.set(e.value().to_string()),
                            }

                            label { class: "input-label", "Criminal History Category" }
                            select {
                                class: "input",
                                value: form_history_category.read().clone(),
                                onchange: move |e: FormEvent| form_history_category.set(e.value().to_string()),
                                option { value: "", "-- Select category --" }
                                for cat in CRIMINAL_HISTORY_CATEGORIES.iter() {
                                    option { value: *cat, "{cat}" }
                                }
                            }

                            label { class: "input-label", "Departure Type" }
                            select {
                                class: "input",
                                value: form_departure_type.read().clone(),
                                onchange: move |e: FormEvent| form_departure_type.set(e.value().to_string()),
                                for dep in DEPARTURE_TYPES.iter() {
                                    option { value: *dep, "{dep}" }
                                }
                            }
                        }

                        Separator {}

                        SheetFooter {
                            div {
                                class: "sheet-footer-actions",
                                SheetClose { on_close: move |_| show_sheet.set(false) }
                                Button {
                                    variant: ButtonVariant::Primary,
                                    "Create Record"
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
fn SentencingTable(records: Vec<SentencingResponse>) -> Element {
    if records.is_empty() {
        return rsx! {
            Card {
                CardContent {
                    p { "No sentencing records found for this court district." }
                }
            }
        };
    }

    rsx! {
        DataTable {
            DataTableHeader {
                DataTableColumn { "Case" }
                DataTableColumn { "Defendant" }
                DataTableColumn { "Custody" }
                DataTableColumn { "Departure" }
                DataTableColumn { "Date" }
            }
            DataTableBody {
                for record in records {
                    SentencingRow { record: record }
                }
            }
        }
    }
}

#[component]
fn SentencingRow(record: SentencingResponse) -> Element {
    let id = record.id.clone();
    let case_id_short = truncate_id(&record.case_id);
    let defendant_id_short = truncate_id(&record.defendant_id);
    let custody_display = record
        .custody_months
        .map(|m| format!("{} mo", m))
        .unwrap_or_else(|| "--".to_string());
    let departure_display = record
        .departure_type
        .clone()
        .unwrap_or_else(|| "None".to_string());
    let date_display = record
        .sentencing_date
        .as_deref()
        .map(|d| d.chars().take(10).collect::<String>())
        .unwrap_or_else(|| "--".to_string());

    // HoverCard details
    let guidelines_low = record.guidelines_range_low_months;
    let guidelines_high = record.guidelines_range_high_months;
    let guidelines_display = match (guidelines_low, guidelines_high) {
        (Some(low), Some(high)) => format!("{} - {} months", low, high),
        _ => "--".to_string(),
    };
    let offense_level_display = record
        .total_offense_level
        .map(|l| l.to_string())
        .unwrap_or_else(|| "--".to_string());
    let history_cat_display = record
        .criminal_history_category
        .clone()
        .unwrap_or_else(|| "--".to_string());
    let fine_display = record
        .fine_amount
        .map(|a| format!("${:.2}", a))
        .unwrap_or_else(|| "--".to_string());
    let supervised_display = record
        .supervised_release_months
        .map(|m| format!("{} mo", m))
        .unwrap_or_else(|| "--".to_string());
    let departure_reason_display = record
        .departure_reason
        .clone()
        .unwrap_or_else(|| "--".to_string());

    let departure_variant = departure_badge_variant(&departure_display);

    rsx! {
        DataTableRow {
            onclick: move |_| {
                let nav = navigator();
                nav.push(Route::SentencingDetail { id: id.clone() });
            },
            DataTableCell {
                HoverCard {
                    HoverCardTrigger {
                        span { class: "attorney-name-link", "{case_id_short}" }
                    }
                    HoverCardContent {
                        div { class: "hover-card-body",
                            div { class: "hover-card-details",
                                span { class: "hover-card-name", "Sentencing Details" }
                                span { class: "hover-card-id", "Offense Level: {offense_level_display}" }
                                span { class: "hover-card-id", "History Category: {history_cat_display}" }
                                span { class: "hover-card-id", "Guidelines: {guidelines_display}" }
                                span { class: "hover-card-id", "Fine: {fine_display}" }
                                span { class: "hover-card-id", "Supervised Release: {supervised_display}" }
                                if departure_display != "None" {
                                    span { class: "hover-card-id", "Departure: {departure_display}" }
                                    span { class: "hover-card-id", "Reason: {departure_reason_display}" }
                                }
                                div { class: "hover-card-meta",
                                    Badge { variant: departure_variant,
                                        "Departure: {departure_display}"
                                    }
                                    if record.appeal_waiver {
                                        Badge { variant: BadgeVariant::Outline, "Appeal Waived" }
                                    }
                                }
                            }
                        }
                    }
                }
            }
            DataTableCell { "{defendant_id_short}" }
            DataTableCell { "{custody_display}" }
            DataTableCell {
                Badge { variant: departure_variant, "{departure_display}" }
            }
            DataTableCell { "{date_display}" }
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

/// Map departure type to a badge variant.
fn departure_badge_variant(departure: &str) -> BadgeVariant {
    match departure {
        "Upward" => BadgeVariant::Destructive,
        "Downward" => BadgeVariant::Primary,
        "None" => BadgeVariant::Secondary,
        _ => BadgeVariant::Outline,
    }
}

/// Truncate a UUID string for table display.
fn truncate_id(id: &str) -> String {
    if id.len() > 8 {
        format!("{}...", &id[..8])
    } else {
        id.to_string()
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

/// Return `serde_json::Value::Null` for empty date strings, or the date string value.
fn opt_date(s: &str) -> serde_json::Value {
    if s.trim().is_empty() {
        serde_json::Value::Null
    } else {
        serde_json::Value::String(s.to_string())
    }
}

/// Parse an optional i32 from a string, returning null for empty/invalid values.
fn parse_opt_i32(s: &str) -> serde_json::Value {
    if s.trim().is_empty() {
        serde_json::Value::Null
    } else {
        match s.trim().parse::<i32>() {
            Ok(n) => serde_json::json!(n),
            Err(_) => serde_json::Value::Null,
        }
    }
}

/// Parse an optional f64 from a string, returning null for empty/invalid values.
fn parse_opt_f64(s: &str) -> serde_json::Value {
    if s.trim().is_empty() {
        serde_json::Value::Null
    } else {
        match s.trim().parse::<f64>() {
            Ok(n) => serde_json::json!(n),
            Err(_) => serde_json::Value::Null,
        }
    }
}
