use dioxus::prelude::*;
use shared_types::RuleResponse;
use shared_ui::components::{
    AlertDialogAction, AlertDialogActions, AlertDialogCancel, AlertDialogContent,
    AlertDialogDescription, AlertDialogRoot, AlertDialogTitle, Badge, BadgeVariant, Button,
    ButtonVariant, Card, CardContent, CardHeader, CardTitle, DetailFooter, DetailGrid, DetailItem,
    DetailList, PageActions, PageHeader, PageTitle, Skeleton, TabContent, TabList, TabTrigger,
    Tabs,
};
use shared_ui::{use_toast, ToastOptions};

use crate::routes::Route;
use crate::CourtContext;

#[component]
pub fn RuleDetailPage(id: String) -> Element {
    let ctx = use_context::<CourtContext>();
    let court_id = ctx.court_id.read().clone();
    let rule_id = id.clone();
    let toast = use_toast();

    let mut show_delete_confirm = use_signal(|| false);
    let mut deleting = use_signal(|| false);

    let data = use_resource(move || {
        let court = court_id.clone();
        let rid = rule_id.clone();
        async move {
            server::api::get_rule(court, rid).await.ok()
        }
    });

    let detail_id = id.clone();
    let handle_delete = move |_: MouseEvent| {
        let court = ctx.court_id.read().clone();
        let rid = detail_id.clone();
        spawn(async move {
            deleting.set(true);
            match server::api::delete_rule(court, rid).await {
                Ok(()) => {
                    toast.success(
                        "Rule deleted successfully".to_string(),
                        ToastOptions::new(),
                    );
                    let nav = navigator();
                    nav.push(Route::RuleList {});
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
                Some(Some(rule)) => rsx! {
                    PageHeader {
                        PageTitle { "{rule.name}" }
                        PageActions {
                            Link { to: Route::RuleList {},
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
                            AlertDialogTitle { "Delete Rule" }
                            AlertDialogDescription {
                                "Are you sure you want to delete this rule? This action cannot be undone."
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

                    Tabs { default_value: "definition", horizontal: true,
                        TabList {
                            TabTrigger { value: "definition", index: 0usize, "Definition" }
                            TabTrigger { value: "metadata", index: 1usize, "Metadata" }
                            TabTrigger { value: "evaluation", index: 2usize, "Evaluation" }
                        }
                        TabContent { value: "definition", index: 0usize,
                            DefinitionTab { rule: rule.clone() }
                        }
                        TabContent { value: "metadata", index: 1usize,
                            MetadataTab { rule: rule.clone() }
                        }
                        TabContent { value: "evaluation", index: 2usize,
                            EvaluationTab { rule: rule.clone() }
                        }
                    }

                    DetailFooter {
                        span { "ID: {rule.id}" }
                    }
                },
                Some(None) => rsx! {
                    Card {
                        CardContent {
                            div { class: "empty-state",
                                h2 { "Rule Not Found" }
                                p { "The rule you're looking for doesn't exist in this court district." }
                                Link { to: Route::RuleList {},
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

/// Definition tab: name, description, source, category, status, priority, citation.
#[component]
fn DefinitionTab(rule: RuleResponse) -> Element {
    let description_display = rule
        .description
        .clone()
        .unwrap_or_else(|| "--".to_string());
    let citation_display = rule
        .citation
        .clone()
        .unwrap_or_else(|| "--".to_string());

    rsx! {
        DetailGrid {
            Card {
                CardHeader { CardTitle { "Rule Definition" } }
                CardContent {
                    DetailList {
                        DetailItem { label: "Name", value: rule.name.clone() }
                        DetailItem { label: "Description", value: description_display }
                        DetailItem { label: "Source",
                            Badge {
                                variant: source_badge_variant(&rule.source),
                                "{rule.source}"
                            }
                        }
                        DetailItem { label: "Category",
                            Badge {
                                variant: BadgeVariant::Secondary,
                                "{rule.category}"
                            }
                        }
                        DetailItem { label: "Status",
                            Badge {
                                variant: status_badge_variant(&rule.status),
                                "{rule.status}"
                            }
                        }
                        DetailItem { label: "Priority", value: format!("{}", rule.priority) }
                        DetailItem { label: "Citation", value: citation_display }
                    }
                }
            }
        }
    }
}

/// Metadata tab: jurisdiction, dates, supersedes, timestamps, conditions/actions JSON.
#[component]
fn MetadataTab(rule: RuleResponse) -> Element {
    let jurisdiction_display = rule
        .jurisdiction
        .clone()
        .unwrap_or_else(|| "--".to_string());
    let effective_display = rule
        .effective_date
        .as_deref()
        .map(|d| format_date(d))
        .unwrap_or_else(|| "--".to_string());
    let expiration_display = rule
        .expiration_date
        .as_deref()
        .map(|d| format_date(d))
        .unwrap_or_else(|| "--".to_string());
    let supersedes_display = rule
        .supersedes_rule_id
        .clone()
        .unwrap_or_else(|| "--".to_string());

    let conditions_formatted = serde_json::to_string_pretty(&rule.conditions)
        .unwrap_or_else(|_| "{}".to_string());
    let actions_formatted = serde_json::to_string_pretty(&rule.actions)
        .unwrap_or_else(|_| "{}".to_string());

    rsx! {
        DetailGrid {
            Card {
                CardHeader { CardTitle { "Metadata" } }
                CardContent {
                    DetailList {
                        DetailItem { label: "Jurisdiction", value: jurisdiction_display }
                        DetailItem { label: "Effective Date", value: effective_display }
                        DetailItem { label: "Expiration Date", value: expiration_display }
                        DetailItem { label: "Supersedes Rule", value: supersedes_display }
                        DetailItem { label: "Created", value: format_date(&rule.created_at) }
                        DetailItem { label: "Updated", value: format_date(&rule.updated_at) }
                    }
                }
            }

            Card {
                CardHeader { CardTitle { "Conditions" } }
                CardContent {
                    pre { class: "json-block", "{conditions_formatted}" }
                }
            }

            Card {
                CardHeader { CardTitle { "Actions" } }
                CardContent {
                    pre { class: "json-block", "{actions_formatted}" }
                }
            }
        }
    }
}

/// Evaluation tab: formatted display of conditions and actions JSON.
#[component]
fn EvaluationTab(rule: RuleResponse) -> Element {
    let conditions_formatted = serde_json::to_string_pretty(&rule.conditions)
        .unwrap_or_else(|_| "{}".to_string());
    let actions_formatted = serde_json::to_string_pretty(&rule.actions)
        .unwrap_or_else(|_| "{}".to_string());

    let has_conditions = !rule.conditions.is_null()
        && rule.conditions != serde_json::Value::Object(serde_json::Map::new());
    let has_actions = !rule.actions.is_null()
        && rule.actions != serde_json::Value::Object(serde_json::Map::new());

    rsx! {
        DetailGrid {
            Card {
                CardHeader { CardTitle { "Rule Conditions" } }
                CardContent {
                    if has_conditions {
                        p { class: "text-muted",
                            "Conditions that trigger this rule:"
                        }
                        pre { class: "json-block", "{conditions_formatted}" }
                    } else {
                        p { class: "text-muted", "No conditions defined for this rule." }
                    }
                }
            }

            Card {
                CardHeader { CardTitle { "Rule Actions" } }
                CardContent {
                    if has_actions {
                        p { class: "text-muted",
                            "Actions performed when this rule matches:"
                        }
                        pre { class: "json-block", "{actions_formatted}" }
                    } else {
                        p { class: "text-muted", "No actions defined for this rule." }
                    }
                }
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

/// Format an ISO date string to just the date portion.
fn format_date(date_str: &str) -> String {
    if date_str.len() >= 10 {
        date_str[..10].to_string()
    } else {
        date_str.to_string()
    }
}
