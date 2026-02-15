use dioxus::prelude::*;
use shared_ui::components::{
    Button, ButtonVariant, Card, CardContent, CardHeader, FormSelect, Input, PageActions,
    PageHeader, PageTitle, Textarea,
};

use crate::routes::Route;
use crate::CourtContext;

#[component]
pub fn CaseCreatePage() -> Element {
    let ctx = use_context::<CourtContext>();
    let court_id = ctx.court_id.read().clone();

    let mut title = use_signal(String::new);
    let mut description = use_signal(String::new);
    let mut crime_type = use_signal(|| "fraud".to_string());
    let mut priority = use_signal(|| "medium".to_string());
    let mut district_code = use_signal(String::new);
    let mut location = use_signal(String::new);
    let mut error_msg = use_signal(|| None::<String>);
    let mut submitting = use_signal(|| false);

    let handle_submit = move |evt: Event<FormData>| {
        evt.prevent_default();
        let court = court_id.clone();
        let t = title.read().clone();
        let d = description.read().clone();
        let ct = crime_type.read().clone();
        let p = priority.read().clone();
        let dc = district_code.read().clone();
        let loc = location.read().clone();

        spawn(async move {
            submitting.set(true);
            error_msg.set(None);

            if t.trim().is_empty() {
                error_msg.set(Some("Title is required.".to_string()));
                submitting.set(false);
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
                    navigator().push(Route::CaseList {});
                }
                Err(e) => {
                    error_msg.set(Some(format!("Failed to create case: {}", e)));
                }
            }
            submitting.set(false);
        });
    };

    rsx! {
        div { class: "container",
            PageHeader {
                PageTitle { "New Case" }
                PageActions {
                    Link { to: Route::CaseList {},
                        Button { variant: ButtonVariant::Secondary, "Back to List" }
                    }
                }
            }

            Card {
                CardHeader { "Case Details" }
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
                                placeholder: "e.g., United States v. Smith",
                            }
                        }

                        div { class: "form-group",
                            Textarea {
                                label: "Description",
                                value: description.read().clone(),
                                on_input: move |evt: FormEvent| description.set(evt.value().to_string()),
                                placeholder: "Case description...",
                            }
                        }

                        div { class: "form-row",
                            div { class: "form-group",
                                FormSelect {
                                    label: "Crime Type *",
                                    value: "{crime_type}",
                                    onchange: move |evt: Event<FormData>| crime_type.set(evt.value().to_string()),
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
                            }

                            div { class: "form-group",
                                FormSelect {
                                    label: "Priority",
                                    value: "{priority}",
                                    onchange: move |evt: Event<FormData>| priority.set(evt.value().to_string()),
                                    option { value: "low", "Low" }
                                    option { value: "medium", "Medium" }
                                    option { value: "high", "High" }
                                    option { value: "critical", "Critical" }
                                }
                            }
                        }

                        div { class: "form-row",
                            div { class: "form-group",
                                Input {
                                    label: "District Code",
                                    value: district_code.read().clone(),
                                    on_input: move |evt: FormEvent| district_code.set(evt.value().to_string()),
                                    placeholder: "Defaults to current court",
                                }
                            }

                            div { class: "form-group",
                                Input {
                                    label: "Location",
                                    value: location.read().clone(),
                                    on_input: move |evt: FormEvent| location.set(evt.value().to_string()),
                                    placeholder: "e.g., Federal Courthouse Room 301",
                                }
                            }
                        }

                        div { class: "form-actions",
                            button {
                                class: "button",
                                "data-style": "primary",
                                r#type: "submit",
                                disabled: *submitting.read(),
                                if *submitting.read() { "Creating..." } else { "Create Case" }
                            }
                        }
                    }
                }
            }
        }
    }
}
