use dioxus::prelude::*;
use shared_types::{
    CaseSearchResponse, JudicialOpinionResponse, PaginatedResponse, PaginationMeta,
    OPINION_STATUSES, OPINION_TYPES,
};
use shared_ui::components::{
    Badge, BadgeVariant, Button, ButtonVariant, Card, CardContent, DataTable, DataTableBody,
    DataTableCell, DataTableColumn, DataTableHeader, DataTableRow, Form, FormSelect, Input,
    PageActions, PageHeader, PageTitle, SearchBar, Separator, Sheet, SheetClose, SheetContent,
    SheetDescription, SheetFooter, SheetHeader, SheetSide, SheetTitle, Skeleton,
};
use shared_ui::{use_toast, HoverCard, HoverCardContent, HoverCardTrigger, ToastOptions};

use crate::routes::Route;
use crate::CourtContext;

#[component]
pub fn OpinionListPage() -> Element {
    let ctx = use_context::<CourtContext>();
    let toast = use_toast();

    let mut page = use_signal(|| 1i64);
    let mut search_query = use_signal(String::new);
    let mut search_input = use_signal(String::new);

    // Sheet state for creating a new opinion
    let mut show_sheet = use_signal(|| false);
    let mut form_title = use_signal(String::new);
    let mut form_opinion_type = use_signal(|| OPINION_TYPES[0].to_string());
    let mut form_case_id = use_signal(String::new);
    let mut form_case_name = use_signal(String::new);
    let mut form_docket_number = use_signal(String::new);
    let mut form_judge_id = use_signal(String::new);
    let mut form_judge_name = use_signal(String::new);
    let mut form_content = use_signal(String::new);
    let mut form_status = use_signal(|| "Draft".to_string());

    // Load cases for the case selector in the create form
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

    // Load judges for the judge selector
    let judges_for_select = use_resource(move || {
        let court = ctx.court_id.read().clone();
        async move {
            match server::api::list_judges(court).await {
                Ok(json) => serde_json::from_str::<Vec<serde_json::Value>>(&json).ok(),
                Err(_) => None,
            }
        }
    });

    // Load opinions data
    let mut data = use_resource(move || {
        let court = ctx.court_id.read().clone();
        let q = search_query.read().clone();
        let p = *page.read();
        async move {
            let search = if q.is_empty() { None } else { Some(q) };
            match server::api::list_all_opinions(court, search, Some(p), Some(20)).await {
                Ok(json) => {
                    serde_json::from_str::<PaginatedResponse<JudicialOpinionResponse>>(&json).ok()
                }
                Err(_) => None,
            }
        }
    });

    let mut reset_form = move || {
        form_title.set(String::new());
        form_opinion_type.set(OPINION_TYPES[0].to_string());
        form_case_id.set(String::new());
        form_case_name.set(String::new());
        form_docket_number.set(String::new());
        form_judge_id.set(String::new());
        form_judge_name.set(String::new());
        form_content.set(String::new());
        form_status.set("Draft".to_string());
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

        if form_title.read().trim().is_empty() {
            toast.error("Title is required.".to_string(), ToastOptions::new());
            return;
        }
        if form_case_id.read().is_empty() {
            toast.error("Case is required.".to_string(), ToastOptions::new());
            return;
        }
        if form_judge_id.read().is_empty() {
            toast.error("Author judge is required.".to_string(), ToastOptions::new());
            return;
        }

        let body = serde_json::json!({
            "case_id": form_case_id.read().clone(),
            "case_name": form_case_name.read().clone(),
            "docket_number": form_docket_number.read().clone(),
            "author_judge_id": form_judge_id.read().clone(),
            "author_judge_name": form_judge_name.read().clone(),
            "opinion_type": form_opinion_type.read().clone(),
            "title": form_title.read().trim().to_string(),
            "content": form_content.read().clone(),
            "status": form_status.read().clone(),
            "keywords": [],
        });

        spawn(async move {
            match server::api::create_opinion(court, body.to_string()).await {
                Ok(_) => {
                    data.restart();
                    show_sheet.set(false);
                    toast.success(
                        "Opinion created successfully".to_string(),
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
                PageTitle { "Opinions" }
                PageActions {
                    Button {
                        variant: ButtonVariant::Primary,
                        onclick: open_create,
                        "New Opinion"
                    }
                }
            }

            SearchBar {
                Input {
                    value: search_input.read().clone(),
                    placeholder: "Search by title, case name, or author...",
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
                    OpinionTable { opinions: resp.data.clone() }
                    PaginationControls { meta: resp.meta.clone(), page: page }
                },
                Some(None) => rsx! {
                    Card {
                        CardContent {
                            p { "No opinions found for this court district." }
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

            // Create opinion Sheet
            Sheet {
                open: show_sheet(),
                on_close: move |_| show_sheet.set(false),
                side: SheetSide::Right,
                SheetContent {
                    SheetHeader {
                        SheetTitle { "New Opinion" }
                        SheetDescription {
                            "Draft a new judicial opinion."
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

                            FormSelect {
                                label: "Opinion Type *",
                                value: "{form_opinion_type}",
                                onchange: move |e: Event<FormData>| form_opinion_type.set(e.value()),
                                for ot in OPINION_TYPES.iter() {
                                    option { value: *ot, "{ot}" }
                                }
                            }

                            // Case selector
                            label { class: "input-label", "Case *" }
                            select {
                                class: "input",
                                value: form_case_id.read().clone(),
                                onchange: move |e: FormEvent| {
                                    let selected_id = e.value().to_string();
                                    form_case_id.set(selected_id.clone());
                                    // Auto-fill case name and docket number from selected case
                                    if let Some(Some(cases)) = &*cases_for_select.read() {
                                        if let Some(c) = cases.iter().find(|c| c.id == selected_id) {
                                            form_case_name.set(c.title.clone());
                                            form_docket_number.set(c.case_number.clone());
                                        }
                                    }
                                },
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

                            // Judge selector
                            label { class: "input-label", "Author Judge *" }
                            select {
                                class: "input",
                                value: form_judge_id.read().clone(),
                                onchange: move |e: FormEvent| {
                                    let selected_id = e.value().to_string();
                                    form_judge_id.set(selected_id.clone());
                                    // Auto-fill judge name from selected judge
                                    if let Some(Some(judges)) = &*judges_for_select.read() {
                                        if let Some(j) = judges.iter().find(|j| {
                                            j["id"].as_str().unwrap_or("") == selected_id
                                        }) {
                                            form_judge_name.set(
                                                j["name"].as_str().unwrap_or("").to_string(),
                                            );
                                        }
                                    }
                                },
                                option { value: "", "-- Select a judge --" }
                                {match &*judges_for_select.read() {
                                    Some(Some(judges)) => rsx! {
                                        for j in judges.iter() {
                                            option {
                                                value: j["id"].as_str().unwrap_or(""),
                                                {j["name"].as_str().unwrap_or("Unknown")}
                                            }
                                        }
                                    },
                                    _ => rsx! {
                                        option { value: "", disabled: true, "Loading judges..." }
                                    },
                                }}
                            }

                            FormSelect {
                                label: "Status",
                                value: "{form_status}",
                                onchange: move |e: Event<FormData>| form_status.set(e.value()),
                                for s in OPINION_STATUSES.iter() {
                                    option { value: *s, "{s}" }
                                }
                            }

                            label { class: "input-label", "Content" }
                            textarea {
                                class: "input",
                                rows: 6,
                                value: form_content.read().clone(),
                                oninput: move |e: FormEvent| form_content.set(e.value().to_string()),
                                placeholder: "Opinion content...",
                            }
                        }

                        Separator {}

                        SheetFooter {
                            div {
                                class: "sheet-footer-actions",
                                SheetClose { on_close: move |_| show_sheet.set(false) }
                                Button {
                                    variant: ButtonVariant::Primary,
                                    "Create Opinion"
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
fn OpinionTable(opinions: Vec<JudicialOpinionResponse>) -> Element {
    if opinions.is_empty() {
        return rsx! {
            Card {
                CardContent {
                    p { "No opinions found for this court district." }
                }
            }
        };
    }

    rsx! {
        DataTable {
            DataTableHeader {
                DataTableColumn { "Title" }
                DataTableColumn { "Case" }
                DataTableColumn { "Author" }
                DataTableColumn { "Type" }
                DataTableColumn { "Status" }
            }
            DataTableBody {
                for opinion in opinions {
                    OpinionRow { opinion: opinion }
                }
            }
        }
    }
}

#[component]
fn OpinionRow(opinion: JudicialOpinionResponse) -> Element {
    let id = opinion.id.clone();
    let status_variant = status_badge_variant(&opinion.status);
    let type_variant = type_badge_variant(&opinion.opinion_type);
    let published_display = opinion
        .published_at
        .as_deref()
        .map(|d| d.chars().take(10).collect::<String>())
        .unwrap_or_else(|| "--".to_string());

    rsx! {
        DataTableRow {
            onclick: move |_| {
                let nav = navigator();
                nav.push(Route::OpinionDetail { id: id.clone() });
            },
            DataTableCell {
                HoverCard {
                    HoverCardTrigger {
                        span { class: "attorney-name-link", "{opinion.title}" }
                    }
                    HoverCardContent {
                        div { class: "hover-card-body",
                            div { class: "hover-card-details",
                                span { class: "hover-card-name", "{opinion.title}" }
                                span { class: "hover-card-id", "Case: {opinion.case_name}" }
                                span { class: "hover-card-id", "Author: {opinion.author_judge_name}" }
                                span { class: "hover-card-id", "Published: {published_display}" }
                                div { class: "hover-card-meta",
                                    Badge { variant: type_variant,
                                        "{opinion.opinion_type}"
                                    }
                                    Badge { variant: status_variant,
                                        "{opinion.status}"
                                    }
                                    if opinion.is_precedential {
                                        Badge { variant: BadgeVariant::Primary, "Precedential" }
                                    }
                                }
                            }
                        }
                    }
                }
            }
            DataTableCell { "{opinion.case_name}" }
            DataTableCell { "{opinion.author_judge_name}" }
            DataTableCell {
                Badge { variant: type_variant, "{opinion.opinion_type}" }
            }
            DataTableCell {
                Badge { variant: status_variant, "{opinion.status}" }
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

/// Map opinion status to an appropriate badge variant.
fn status_badge_variant(status: &str) -> BadgeVariant {
    match status {
        "Draft" => BadgeVariant::Outline,
        "Under Review" | "Circulated" => BadgeVariant::Secondary,
        "Filed" => BadgeVariant::Primary,
        "Published" => BadgeVariant::Primary,
        "Withdrawn" => BadgeVariant::Destructive,
        "Superseded" => BadgeVariant::Secondary,
        _ => BadgeVariant::Outline,
    }
}

/// Map opinion type to an appropriate badge variant.
fn type_badge_variant(opinion_type: &str) -> BadgeVariant {
    match opinion_type {
        "Majority" => BadgeVariant::Primary,
        "Concurrence" => BadgeVariant::Secondary,
        "Dissent" => BadgeVariant::Destructive,
        "Per Curiam" => BadgeVariant::Primary,
        "Memorandum" => BadgeVariant::Outline,
        "En Banc" => BadgeVariant::Primary,
        "Summary" => BadgeVariant::Secondary,
        _ => BadgeVariant::Outline,
    }
}
