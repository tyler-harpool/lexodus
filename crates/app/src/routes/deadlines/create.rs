use dioxus::prelude::*;
use shared_ui::components::{
    Button, ButtonVariant, Card, CardContent, CardHeader, Input, PageActions, PageHeader, PageTitle,
    Textarea,
};

use crate::routes::Route;
use crate::CourtContext;

#[component]
pub fn DeadlineCreatePage() -> Element {
    let ctx = use_context::<CourtContext>();
    let court_id = ctx.court_id.read().clone();

    let mut title = use_signal(String::new);
    let mut due_at = use_signal(String::new);
    let mut rule_code = use_signal(String::new);
    let mut notes = use_signal(String::new);
    let mut error_msg = use_signal(|| None::<String>);
    let mut submitting = use_signal(|| false);

    let handle_submit = move |evt: Event<FormData>| {
        evt.prevent_default();
        let court = court_id.clone();
        let t = title.read().clone();
        let d = due_at.read().clone();
        let r = rule_code.read().clone();
        let n = notes.read().clone();

        spawn(async move {
            submitting.set(true);
            error_msg.set(None);

            if t.trim().is_empty() || d.trim().is_empty() {
                error_msg.set(Some("Title and due date are required.".to_string()));
                submitting.set(false);
                return;
            }

            // Convert HTML datetime-local to RFC3339
            let due_rfc3339 = format!("{}:00Z", d);

            let body = serde_json::json!({
                "title": t.trim(),
                "due_at": due_rfc3339,
                "rule_code": if r.is_empty() { None::<String> } else { Some(r) },
                "notes": if n.is_empty() { None::<String> } else { Some(n) },
            });

            match server::api::create_deadline(court, body.to_string()).await {
                Ok(_) => {
                    let nav = navigator();
                    nav.push(Route::DeadlineList {});
                }
                Err(e) => {
                    error_msg.set(Some(format!("Failed to create deadline: {}", e)));
                }
            }
            submitting.set(false);
        });
    };

    rsx! {
        div { class: "container",
            PageHeader {
                PageTitle { "New Deadline" }
                PageActions {
                    Link { to: Route::DeadlineList {},
                        Button { variant: ButtonVariant::Secondary, "Back to List" }
                    }
                }
            }

            Card {
                CardHeader { "Deadline Details" }
                CardContent {
                    if let Some(err) = &*error_msg.read() {
                        div { class: "error-message", "{err}" }
                    }

                    form { onsubmit: handle_submit,
                        div { class: "form-group",
                            Input {
                                label: "Title *",
                                value: title.read().clone(),
                                on_input: move |evt: FormEvent| title.set(evt.value().to_string()),
                                placeholder: "e.g., File Motion Response",
                            }
                        }

                        div { class: "form-group",
                            Input {
                                label: "Due Date *",
                                input_type: "datetime-local",
                                value: due_at.read().clone(),
                                on_input: move |evt: FormEvent| due_at.set(evt.value().to_string()),
                            }
                        }

                        div { class: "form-group",
                            Input {
                                label: "Rule Code",
                                value: rule_code.read().clone(),
                                on_input: move |evt: FormEvent| rule_code.set(evt.value().to_string()),
                                placeholder: "e.g., FRCP 12(b)",
                            }
                        }

                        div { class: "form-group",
                            Textarea {
                                label: "Notes",
                                value: notes.read().clone(),
                                on_input: move |evt: FormEvent| notes.set(evt.value().to_string()),
                                placeholder: "Optional notes...",
                            }
                        }

                        div { class: "form-actions",
                            button {
                                class: "button",
                                "data-style": "primary",
                                r#type: "submit",
                                disabled: *submitting.read(),
                                if *submitting.read() { "Creating..." } else { "Create Deadline" }
                            }
                        }
                    }
                }
            }
        }
    }
}
