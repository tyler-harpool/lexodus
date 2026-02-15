use dioxus::prelude::*;
use shared_types::AttorneyResponse;
use shared_ui::components::{
    Button, ButtonVariant, Card, CardContent, CardHeader, CardTitle, Form, Input, PageActions,
    PageHeader, PageTitle,
};

use crate::routes::Route;
use crate::CourtContext;

#[component]
pub fn AttorneyCreatePage() -> Element {
    let ctx = use_context::<CourtContext>();

    let mut bar_number = use_signal(String::new);
    let mut first_name = use_signal(String::new);
    let mut last_name = use_signal(String::new);
    let mut middle_name = use_signal(String::new);
    let mut email = use_signal(String::new);
    let mut phone = use_signal(String::new);
    let mut firm_name = use_signal(String::new);
    let mut fax = use_signal(String::new);

    let mut street1 = use_signal(String::new);
    let mut street2 = use_signal(String::new);
    let mut city = use_signal(String::new);
    let mut state = use_signal(String::new);
    let mut zip_code = use_signal(String::new);
    let mut country = use_signal(|| "US".to_string());

    let mut error_msg = use_signal(|| None::<String>);
    let mut submitting = use_signal(|| false);

    let handle_submit = move |_evt: FormEvent| {
        let court_id = ctx.court_id.read().clone();
        let body = serde_json::json!({
            "bar_number": bar_number.read().clone(),
            "first_name": first_name.read().clone(),
            "last_name": last_name.read().clone(),
            "middle_name": opt_str(&middle_name.read()),
            "firm_name": opt_str(&firm_name.read()),
            "email": email.read().clone(),
            "phone": phone.read().clone(),
            "fax": opt_str(&fax.read()),
            "address": {
                "street1": street1.read().clone(),
                "street2": opt_str(&street2.read()),
                "city": city.read().clone(),
                "state": state.read().clone(),
                "zip_code": zip_code.read().clone(),
                "country": country.read().clone(),
            }
        });

        spawn(async move {
            submitting.set(true);
            error_msg.set(None);

            match server::api::create_attorney(court_id, body.to_string()).await {
                Ok(json) => {
                    if let Ok(att) = serde_json::from_str::<AttorneyResponse>(&json) {
                        let nav = navigator();
                        nav.push(Route::AttorneyDetail { id: att.id });
                    }
                }
                Err(e) => {
                    error_msg.set(Some(format!("{}", e)));
                }
            }
            submitting.set(false);
        });
    };

    rsx! {
        div { class: "container",
            PageHeader {
                PageTitle { "Create Attorney" }
                PageActions {
                    Link { to: Route::AttorneyList {},
                        Button { variant: ButtonVariant::Secondary, "Back to List" }
                    }
                }
            }

            if let Some(err) = error_msg.read().as_ref() {
                div { class: "alert alert-error", "{err}" }
            }

            Card {
                Form { onsubmit: handle_submit,
                    CardHeader {
                        CardTitle { "Personal Information" }
                    }
                    CardContent {
                        div { class: "form-row",
                            Input {
                                label: "Bar Number *",
                                value: bar_number.read().clone(),
                                on_input: move |e: FormEvent| bar_number.set(e.value().to_string()),
                            }
                            Input {
                                label: "First Name *",
                                value: first_name.read().clone(),
                                on_input: move |e: FormEvent| first_name.set(e.value().to_string()),
                            }
                            Input {
                                label: "Last Name *",
                                value: last_name.read().clone(),
                                on_input: move |e: FormEvent| last_name.set(e.value().to_string()),
                            }
                        }
                        div { class: "form-row",
                            Input {
                                label: "Middle Name",
                                value: middle_name.read().clone(),
                                on_input: move |e: FormEvent| middle_name.set(e.value().to_string()),
                            }
                            Input {
                                label: "Firm Name",
                                value: firm_name.read().clone(),
                                on_input: move |e: FormEvent| firm_name.set(e.value().to_string()),
                            }
                        }
                    }

                    CardHeader {
                        CardTitle { "Contact Information" }
                    }
                    CardContent {
                        div { class: "form-row",
                            Input {
                                label: "Email *",
                                input_type: "email",
                                value: email.read().clone(),
                                on_input: move |e: FormEvent| email.set(e.value().to_string()),
                            }
                            Input {
                                label: "Phone *",
                                input_type: "tel",
                                value: phone.read().clone(),
                                on_input: move |e: FormEvent| phone.set(e.value().to_string()),
                            }
                            Input {
                                label: "Fax",
                                input_type: "tel",
                                value: fax.read().clone(),
                                on_input: move |e: FormEvent| fax.set(e.value().to_string()),
                            }
                        }
                    }

                    CardHeader {
                        CardTitle { "Address" }
                    }
                    CardContent {
                        div { class: "form-row",
                            div { class: "form-group-wide",
                                Input {
                                    label: "Street Address *",
                                    value: street1.read().clone(),
                                    on_input: move |e: FormEvent| street1.set(e.value().to_string()),
                                }
                            }
                            div { class: "form-group-wide",
                                Input {
                                    label: "Street Address 2",
                                    value: street2.read().clone(),
                                    on_input: move |e: FormEvent| street2.set(e.value().to_string()),
                                }
                            }
                        }
                        div { class: "form-row",
                            Input {
                                label: "City *",
                                value: city.read().clone(),
                                on_input: move |e: FormEvent| city.set(e.value().to_string()),
                            }
                            Input {
                                label: "State *",
                                value: state.read().clone(),
                                on_input: move |e: FormEvent| state.set(e.value().to_string()),
                            }
                            Input {
                                label: "ZIP Code *",
                                value: zip_code.read().clone(),
                                on_input: move |e: FormEvent| zip_code.set(e.value().to_string()),
                            }
                            Input {
                                label: "Country *",
                                value: country.read().clone(),
                                on_input: move |e: FormEvent| country.set(e.value().to_string()),
                            }
                        }
                    }

                    div { class: "form-actions",
                        button {
                            class: "button",
                            "data-style": "primary",
                            r#type: "submit",
                            disabled: *submitting.read(),
                            if *submitting.read() { "Creating..." } else { "Create Attorney" }
                        }
                        Link { to: Route::AttorneyList {},
                            Button { variant: ButtonVariant::Secondary, "Cancel" }
                        }
                    }
                }
            }
        }
    }
}

fn opt_str(s: &str) -> serde_json::Value {
    if s.trim().is_empty() {
        serde_json::Value::Null
    } else {
        serde_json::Value::String(s.to_string())
    }
}
