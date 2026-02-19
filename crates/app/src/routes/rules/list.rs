use dioxus::prelude::*;
use shared_types::RuleResponse;
use shared_ui::components::{
    Badge, BadgeVariant, Button, ButtonVariant, Card, CardContent, DataTable, DataTableBody,
    DataTableCell, DataTableColumn, DataTableHeader, DataTableRow, Form, FormSelect, Input,
    PageActions, PageHeader, PageTitle, SearchBar, Separator, Sheet, SheetClose, SheetContent,
    SheetDescription, SheetFooter, SheetHeader, SheetSide, SheetTitle, Skeleton,
};
use shared_ui::{use_toast, HoverCard, HoverCardContent, HoverCardTrigger, ToastOptions};

use crate::routes::Route;
use crate::CourtContext;

/// Rule source options for the create form.
const RULE_SOURCES: &[&str] = &["FRCP", "LocalRule", "Statute"];

/// Rule status options for the create form.
const RULE_STATUSES: &[&str] = &["Active", "Superseded", "Repealed"];

#[component]
pub fn RuleListPage() -> Element {
    let ctx = use_context::<CourtContext>();
    let toast = use_toast();

    let mut search_input = use_signal(String::new);

    // Sheet state for creating rules
    let mut show_sheet = use_signal(|| false);
    let mut form_name = use_signal(String::new);
    let mut form_description = use_signal(String::new);
    let mut form_source = use_signal(|| "FRCP".to_string());
    let mut form_category = use_signal(String::new);
    let mut form_priority = use_signal(|| "0".to_string());
    let mut form_status = use_signal(|| "Active".to_string());
    let mut form_jurisdiction = use_signal(String::new);
    let mut form_citation = use_signal(String::new);

    let mut data = use_resource(move || {
        let court = ctx.court_id.read().clone();
        async move {
            match server::api::list_rules(court).await {
                Ok(json) => serde_json::from_str::<Vec<RuleResponse>>(&json).ok(),
                Err(_) => None,
            }
        }
    });

    let mut reset_form = move || {
        form_name.set(String::new());
        form_description.set(String::new());
        form_source.set("FRCP".to_string());
        form_category.set(String::new());
        form_priority.set("0".to_string());
        form_status.set("Active".to_string());
        form_jurisdiction.set(String::new());
        form_citation.set(String::new());
    };

    let open_create = move |_| {
        reset_form();
        show_sheet.set(true);
    };

    let handle_clear = move |_| {
        search_input.set(String::new());
    };

    let handle_save = move |_: FormEvent| {
        let court = ctx.court_id.read().clone();

        let priority_val: i32 = form_priority.read().parse().unwrap_or(0);

        let body = serde_json::json!({
            "name": form_name.read().clone(),
            "description": form_description.read().clone(),
            "source": form_source.read().clone(),
            "category": form_category.read().clone(),
            "priority": priority_val,
            "status": form_status.read().clone(),
            "jurisdiction": opt_str(&form_jurisdiction.read()),
            "citation": opt_str(&form_citation.read()),
        });

        spawn(async move {
            match server::api::create_rule(court, body.to_string()).await {
                Ok(_) => {
                    data.restart();
                    show_sheet.set(false);
                    toast.success(
                        "Rule created successfully".to_string(),
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
                PageTitle { "Rules" }
                PageActions {
                    Button {
                        variant: ButtonVariant::Primary,
                        onclick: open_create,
                        "New Rule"
                    }
                }
            }

            SearchBar {
                Input {
                    value: search_input.read().clone(),
                    placeholder: "Filter by name, source, category, or citation...",
                    label: "",
                    on_input: move |evt: FormEvent| search_input.set(evt.value().to_string()),
                }
                if !search_input.read().is_empty() {
                    Button {
                        variant: ButtonVariant::Secondary,
                        onclick: handle_clear,
                        "Clear"
                    }
                }
            }

            match &*data.read() {
                Some(Some(rules)) => {
                    let query = search_input.read().to_lowercase();
                    let filtered: Vec<RuleResponse> = if query.is_empty() {
                        rules.clone()
                    } else {
                        rules
                            .iter()
                            .filter(|r| {
                                r.name.to_lowercase().contains(&query)
                                    || r.source.to_lowercase().contains(&query)
                                    || r.category.to_lowercase().contains(&query)
                                    || r.citation
                                        .as_deref()
                                        .unwrap_or("")
                                        .to_lowercase()
                                        .contains(&query)
                                    || r.status.to_lowercase().contains(&query)
                            })
                            .cloned()
                            .collect()
                    };

                    rsx! {
                        RuleTable { rules: filtered }
                    }
                },
                Some(None) => rsx! {
                    Card {
                        CardContent {
                            p { "No rules found for this court district." }
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

            // Create rule Sheet
            Sheet {
                open: show_sheet(),
                on_close: move |_| show_sheet.set(false),
                side: SheetSide::Right,
                SheetContent {
                    SheetHeader {
                        SheetTitle { "New Rule" }
                        SheetDescription {
                            "Create a new court rule or local rule."
                        }
                        SheetClose { on_close: move |_| show_sheet.set(false) }
                    }

                    Form {
                        onsubmit: handle_save,

                        div {
                            class: "sheet-form",

                            Input {
                                label: "Name *",
                                value: form_name.read().clone(),
                                on_input: move |e: FormEvent| form_name.set(e.value().to_string()),
                                placeholder: "e.g., Local Rule 7.1",
                            }

                            Input {
                                label: "Description *",
                                value: form_description.read().clone(),
                                on_input: move |e: FormEvent| form_description.set(e.value().to_string()),
                                placeholder: "Rule description",
                            }

                            FormSelect {
                                label: "Source *",
                                value: "{form_source}",
                                onchange: move |e: Event<FormData>| form_source.set(e.value()),
                                for src in RULE_SOURCES.iter() {
                                    option { value: *src, "{src}" }
                                }
                            }

                            Input {
                                label: "Category *",
                                value: form_category.read().clone(),
                                on_input: move |e: FormEvent| form_category.set(e.value().to_string()),
                                placeholder: "e.g., Discovery, Motions, Filing",
                            }

                            Input {
                                label: "Priority",
                                input_type: "number",
                                value: form_priority.read().clone(),
                                on_input: move |e: FormEvent| form_priority.set(e.value().to_string()),
                                placeholder: "0 = lowest",
                            }

                            FormSelect {
                                label: "Status",
                                value: "{form_status}",
                                onchange: move |e: Event<FormData>| form_status.set(e.value()),
                                for s in RULE_STATUSES.iter() {
                                    option { value: *s, "{s}" }
                                }
                            }

                            Input {
                                label: "Jurisdiction",
                                value: form_jurisdiction.read().clone(),
                                on_input: move |e: FormEvent| form_jurisdiction.set(e.value().to_string()),
                                placeholder: "e.g., Eastern District of Texas",
                            }

                            Input {
                                label: "Citation",
                                value: form_citation.read().clone(),
                                on_input: move |e: FormEvent| form_citation.set(e.value().to_string()),
                                placeholder: "e.g., Fed. R. Civ. P. 26",
                            }
                        }

                        Separator {}

                        SheetFooter {
                            div {
                                class: "sheet-footer-actions",
                                SheetClose { on_close: move |_| show_sheet.set(false) }
                                Button {
                                    variant: ButtonVariant::Primary,
                                    "Create Rule"
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
fn RuleTable(rules: Vec<RuleResponse>) -> Element {
    if rules.is_empty() {
        return rsx! {
            Card {
                CardContent {
                    p { "No rules match the current filter." }
                }
            }
        };
    }

    rsx! {
        DataTable {
            DataTableHeader {
                DataTableColumn { "Name" }
                DataTableColumn { "Source" }
                DataTableColumn { "Category" }
                DataTableColumn { "Priority" }
                DataTableColumn { "Status" }
            }
            DataTableBody {
                for rule in rules {
                    RuleRow { rule: rule }
                }
            }
        }
    }
}

#[component]
fn RuleRow(rule: RuleResponse) -> Element {
    let id = rule.id.clone();
    let source_variant = source_badge_variant(&rule.source);
    let status_variant = status_badge_variant(&rule.status);
    let description_display = rule
        .description
        .clone()
        .unwrap_or_else(|| "No description".to_string());
    let citation_display = rule
        .citation
        .clone()
        .unwrap_or_else(|| "--".to_string());
    let jurisdiction_display = rule
        .jurisdiction
        .clone()
        .unwrap_or_else(|| "--".to_string());

    rsx! {
        DataTableRow {
            onclick: move |_| {
                let nav = navigator();
                nav.push(Route::RuleDetail { id: id.clone() });
            },
            DataTableCell {
                HoverCard {
                    HoverCardTrigger {
                        span { class: "rule-name-link", "{rule.name}" }
                    }
                    HoverCardContent {
                        div { class: "hover-card-body",
                            div { class: "hover-card-details",
                                span { class: "hover-card-name", "{rule.name}" }
                                span { class: "hover-card-id", "{description_display}" }
                                if citation_display != "--" {
                                    span { class: "hover-card-username", "Citation: {citation_display}" }
                                }
                                if jurisdiction_display != "--" {
                                    span { class: "hover-card-id", "Jurisdiction: {jurisdiction_display}" }
                                }
                                div { class: "hover-card-meta",
                                    Badge { variant: source_variant, "{rule.source}" }
                                    Badge { variant: status_variant, "{rule.status}" }
                                }
                            }
                        }
                    }
                }
            }
            DataTableCell {
                Badge { variant: source_variant, "{rule.source}" }
            }
            DataTableCell { "{rule.category}" }
            DataTableCell { "{rule.priority}" }
            DataTableCell {
                Badge { variant: status_variant, "{rule.status}" }
            }
        }
    }
}

/// Map rule source to an appropriate badge variant.
fn source_badge_variant(source: &str) -> BadgeVariant {
    match source {
        "FRCP" => BadgeVariant::Primary,
        "LocalRule" => BadgeVariant::Secondary,
        "Statute" => BadgeVariant::Outline,
        _ => BadgeVariant::Secondary,
    }
}

/// Map rule status to an appropriate badge variant.
fn status_badge_variant(status: &str) -> BadgeVariant {
    match status {
        "Active" => BadgeVariant::Primary,
        "Superseded" => BadgeVariant::Secondary,
        "Repealed" => BadgeVariant::Destructive,
        _ => BadgeVariant::Secondary,
    }
}

/// Convert an empty string to JSON null, otherwise wrap in a JSON string.
fn opt_str(s: &str) -> serde_json::Value {
    if s.trim().is_empty() {
        serde_json::Value::Null
    } else {
        serde_json::Value::String(s.to_string())
    }
}
