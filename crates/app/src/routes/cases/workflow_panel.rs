use dioxus::prelude::*;
use shared_ui::components::{
    Badge, BadgeVariant, Button, ButtonVariant, Card, CardContent, CardHeader, CardTitle,
    Separator, Skeleton, Textarea,
};

use crate::routes::Route;
use crate::CourtContext;

/// Human-readable label for a pipeline step.
#[allow(dead_code)]
fn step_label(step: &str) -> &str {
    match step {
        "review" => "Review Filing",
        "docket" => "Create Docket Entry",
        "nef" => "Generate NEF",
        "route_judge" => "Route to Judge",
        "serve" => "Serve Parties",
        "completed" => "Completed",
        _ => step,
    }
}

/// Workflow panel shown at the top of case detail when processing a queue item.
#[component]
pub fn WorkflowPanel(queue_id: String) -> Element {
    let ctx = use_context::<CourtContext>();
    let court = ctx.court_id.read().clone();
    let _nav = use_navigator();

    let mut reject_reason = use_signal(|| String::new());
    let mut show_reject = use_signal(|| false);
    let mut action_loading = use_signal(|| false);

    let qid = queue_id.clone();
    let mut item_resource = use_resource(move || {
        let court = court.clone();
        let id = qid.clone();
        async move {
            server::api::search_queue(
                court, None, None, None, None, None, None, Some(50),
            )
            .await
            .ok()
            .and_then(|resp| resp.items.into_iter().find(|i| i.id == id))
        }
    });

    let court_advance = ctx.court_id.read().clone();
    let court_reject = ctx.court_id.read().clone();
    let qid_advance = queue_id.clone();
    let qid_reject = queue_id.clone();

    rsx! {
        document::Link { rel: "stylesheet", href: asset!("./workflow_panel.css") }
        match &*item_resource.read() {
            Some(Some(item)) => {
                let steps = shared_types::pipeline_steps(&item.queue_type);
                let current_idx = steps.iter().position(|&s| s == item.current_step).unwrap_or(0);
                let total_steps = steps.len();
                let is_completed = item.status == "completed" || item.status == "rejected";

                rsx! {
                    Card {
                        CardHeader {
                            div { class: "workflow-header",
                                CardTitle { "Processing: {item.title}" }
                                Badge {
                                    variant: if item.status == "completed" { BadgeVariant::Primary }
                                             else if item.status == "rejected" { BadgeVariant::Destructive }
                                             else { BadgeVariant::Secondary },
                                    "{item.status}"
                                }
                            }
                        }
                        CardContent {
                            // Pipeline step indicator
                            div { class: "workflow-steps",
                                for (idx, step) in steps.iter().enumerate() {
                                    div {
                                        class: if idx < current_idx { "workflow-step workflow-step-done" }
                                               else if idx == current_idx && !is_completed { "workflow-step workflow-step-active" }
                                               else if is_completed && idx <= current_idx { "workflow-step workflow-step-done" }
                                               else { "workflow-step" },
                                        div { class: "workflow-step-dot",
                                            if idx < current_idx || (is_completed && idx <= current_idx) {
                                                "\u{2713}"
                                            } else {
                                                "{idx + 1}"
                                            }
                                        }
                                        span { class: "workflow-step-label", "{step_label(step)}" }
                                    }
                                    if idx < steps.len() - 1 {
                                        div { class: if idx < current_idx { "workflow-step-line workflow-step-line-done" }
                                                     else { "workflow-step-line" } }
                                    }
                                }
                            }

                            Separator {}

                            // Current step info
                            if is_completed {
                                div { class: "workflow-complete",
                                    if item.status == "completed" {
                                        p { class: "workflow-complete-text", "All steps completed successfully." }
                                    } else {
                                        p { class: "workflow-complete-text workflow-rejected", "This item was rejected." }
                                    }
                                    Link { to: Route::Dashboard {},
                                        Button {
                                            variant: ButtonVariant::Secondary,
                                            "Return to Queue"
                                        }
                                    }
                                }
                            } else {
                                div { class: "workflow-current",
                                    p { class: "workflow-step-info",
                                        "Step {current_idx + 1} of {total_steps}: {step_label(&item.current_step)}"
                                    }

                                    if *show_reject.read() {
                                        div { class: "workflow-reject-form",
                                            Textarea {
                                                value: reject_reason.read().clone(),
                                                placeholder: "Reason for rejection...",
                                                on_input: move |evt: FormEvent| reject_reason.set(evt.value()),
                                            }
                                            div { class: "workflow-reject-actions",
                                                Button {
                                                    variant: ButtonVariant::Secondary,
                                                    onclick: move |_| show_reject.set(false),
                                                    "Cancel"
                                                }
                                                Button {
                                                    variant: ButtonVariant::Destructive,
                                                    disabled: reject_reason.read().trim().is_empty() || *action_loading.read(),
                                                    onclick: {
                                                        let court = court_reject.clone();
                                                        let id = qid_reject.clone();
                                                        move |_| {
                                                            let court = court.clone();
                                                            let id = id.clone();
                                                            let reason = reject_reason.read().clone();
                                                            spawn(async move {
                                                                action_loading.set(true);
                                                                let _ = server::api::reject_queue_item_fn(court, id, reason).await;
                                                                action_loading.set(false);
                                                                item_resource.restart();
                                                            });
                                                        }
                                                    },
                                                    "Confirm Reject"
                                                }
                                            }
                                        }
                                    } else {
                                        div { class: "workflow-actions",
                                            Button {
                                                variant: ButtonVariant::Destructive,
                                                onclick: move |_| show_reject.set(true),
                                                "Reject"
                                            }
                                            Button {
                                                variant: ButtonVariant::Primary,
                                                disabled: *action_loading.read(),
                                                onclick: {
                                                    let court = court_advance.clone();
                                                    let id = qid_advance.clone();
                                                    move |_| {
                                                        let court = court.clone();
                                                        let id = id.clone();
                                                        spawn(async move {
                                                            action_loading.set(true);
                                                            let _ = server::api::advance_queue_item_fn(court, id).await;
                                                            action_loading.set(false);
                                                            item_resource.restart();
                                                        });
                                                    }
                                                },
                                                if current_idx == total_steps - 1 { "Complete" } else { "Advance to Next Step" }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            },
            Some(None) => rsx! {
                Card {
                    CardContent {
                        p { "Queue item not found." }
                    }
                }
            },
            None => rsx! {
                Card {
                    CardContent {
                        Skeleton { style: "height: 4rem; width: 100%;" }
                    }
                }
            },
        }
    }
}
