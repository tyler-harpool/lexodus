use dioxus::prelude::*;
use shared_types::AttorneyResponse;
use shared_ui::components::{
    AlertDialogAction, AlertDialogActions, AlertDialogCancel, AlertDialogContent,
    AlertDialogDescription, AlertDialogRoot, AlertDialogTitle, Form, Input, Separator, Sheet,
    SheetClose, SheetContent, SheetDescription, SheetFooter, SheetHeader, SheetSide, SheetTitle,
};
use shared_ui::{use_toast, ToastOptions};

use crate::CourtContext;

/// Controls whether the form is in Create or Edit mode.
#[derive(Clone, Copy, PartialEq)]
pub enum FormMode {
    Create,
    Edit,
}

/// Unified create/edit form for attorneys, rendered inside a Sheet.
///
/// - `mode`: Create or Edit
/// - `initial`: None for create, Some(AttorneyResponse) for edit (pre-populates fields)
/// - `open`: whether the sheet is visible
/// - `on_close`: called when user closes the sheet (after dirty check)
/// - `on_saved`: called after successful save (caller should `data.restart()`)
#[component]
pub fn AttorneyFormSheet(
    mode: FormMode,
    initial: Option<AttorneyResponse>,
    open: bool,
    on_close: EventHandler<()>,
    on_saved: EventHandler<()>,
) -> Element {
    let ctx = use_context::<CourtContext>();
    let toast = use_toast();

    // --- Form field signals ---
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

    // --- Hydration: sync signals from initial data ---
    let mut hydrated_id = use_signal(String::new);
    let initial_for_hydration = initial.clone();

    use_effect(move || {
        if !open {
            return;
        }
        if let Some(ref data) = initial_for_hydration {
            let id = data.id.clone();
            if *hydrated_id.read() != id {
                hydrated_id.set(id);
                bar_number.set(data.bar_number.clone());
                first_name.set(data.first_name.clone());
                last_name.set(data.last_name.clone());
                middle_name.set(data.middle_name.clone().unwrap_or_default());
                email.set(data.email.clone());
                phone.set(data.phone.clone());
                firm_name.set(data.firm_name.clone().unwrap_or_default());
                fax.set(data.fax.clone().unwrap_or_default());
                street1.set(data.address.street1.clone());
                street2.set(data.address.street2.clone().unwrap_or_default());
                city.set(data.address.city.clone());
                state.set(data.address.state.clone());
                zip_code.set(data.address.zip_code.clone());
                country.set(data.address.country.clone());
            }
        } else if mode == FormMode::Create && hydrated_id.read().is_empty() {
            // Already at defaults for create
        } else if mode == FormMode::Create {
            // Reset for a fresh create
            hydrated_id.set(String::new());
            bar_number.set(String::new());
            first_name.set(String::new());
            last_name.set(String::new());
            middle_name.set(String::new());
            email.set(String::new());
            phone.set(String::new());
            firm_name.set(String::new());
            fax.set(String::new());
            street1.set(String::new());
            street2.set(String::new());
            city.set(String::new());
            state.set(String::new());
            zip_code.set(String::new());
            country.set("US".to_string());
        }
    });

    // --- Dirty state tracking ---
    let mut initial_snapshot = use_signal(String::new);

    use_effect(move || {
        if open {
            let snap = snapshot(
                &bar_number, &first_name, &last_name, &middle_name, &email, &phone, &firm_name,
                &fax, &street1, &street2, &city, &state, &zip_code, &country,
            );
            initial_snapshot.set(snap);
        }
    });

    let is_dirty = move || {
        let current = snapshot(
            &bar_number, &first_name, &last_name, &middle_name, &email, &phone, &firm_name, &fax,
            &street1, &street2, &city, &state, &zip_code, &country,
        );
        *initial_snapshot.read() != current
    };

    let mut show_discard = use_signal(|| false);

    let try_close = move |_| {
        if is_dirty() {
            show_discard.set(true);
        } else {
            on_close.call(());
        }
    };

    // --- Submit ---
    let mut in_flight = use_signal(|| false);

    let handle_save = move |_: FormEvent| {
        if *in_flight.read() {
            return;
        }
        let court = ctx.court_id.read().clone();
        let id = initial.as_ref().map(|d| d.id.clone()).unwrap_or_default();

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
            in_flight.set(true);
            let result = match mode {
                FormMode::Create => {
                    server::api::create_attorney(court, body.to_string()).await
                }
                FormMode::Edit => {
                    server::api::update_attorney(court, id, body.to_string()).await
                }
            };
            match result {
                Ok(_) => {
                    on_saved.call(());
                    on_close.call(());
                    let msg = match mode {
                        FormMode::Create => "Attorney created successfully",
                        FormMode::Edit => "Attorney updated successfully",
                    };
                    toast.success(msg.to_string(), ToastOptions::new());
                }
                Err(e) => {
                    toast.error(format!("{e}"), ToastOptions::new());
                }
            }
            in_flight.set(false);
        });
    };

    // --- Render ---
    let title = match mode {
        FormMode::Create => "New Attorney",
        FormMode::Edit => "Edit Attorney",
    };
    let description = match mode {
        FormMode::Create => "Add a new attorney to this court district.",
        FormMode::Edit => "Modify attorney information.",
    };
    let submit_label = match mode {
        FormMode::Create => "Create Attorney",
        FormMode::Edit => "Save Changes",
    };

    rsx! {
        Sheet {
            open,
            on_close: try_close,
            side: SheetSide::Right,
            SheetContent {
                SheetHeader {
                    SheetTitle { "{title}" }
                    SheetDescription { "{description}" }
                    SheetClose { on_close: try_close }
                }

                Form {
                    onsubmit: handle_save,

                    div {
                        class: "sheet-form",

                        // Personal Information
                        Input {
                            label: "Bar Number *",
                            value: bar_number.read().clone(),
                            on_input: move |e: FormEvent| bar_number.set(e.value()),
                            placeholder: "e.g., NY-123456",
                        }
                        Input {
                            label: "First Name *",
                            value: first_name.read().clone(),
                            on_input: move |e: FormEvent| first_name.set(e.value()),
                        }
                        Input {
                            label: "Last Name *",
                            value: last_name.read().clone(),
                            on_input: move |e: FormEvent| last_name.set(e.value()),
                        }
                        Input {
                            label: "Middle Name",
                            value: middle_name.read().clone(),
                            on_input: move |e: FormEvent| middle_name.set(e.value()),
                        }
                        Input {
                            label: "Firm Name",
                            value: firm_name.read().clone(),
                            on_input: move |e: FormEvent| firm_name.set(e.value()),
                        }

                        Separator {}

                        // Contact
                        Input {
                            label: "Email *",
                            input_type: "email",
                            value: email.read().clone(),
                            on_input: move |e: FormEvent| email.set(e.value()),
                        }
                        Input {
                            label: "Phone *",
                            input_type: "tel",
                            value: phone.read().clone(),
                            on_input: move |e: FormEvent| phone.set(e.value()),
                        }
                        Input {
                            label: "Fax",
                            input_type: "tel",
                            value: fax.read().clone(),
                            on_input: move |e: FormEvent| fax.set(e.value()),
                        }

                        Separator {}

                        // Address
                        Input {
                            label: "Street Address *",
                            value: street1.read().clone(),
                            on_input: move |e: FormEvent| street1.set(e.value()),
                        }
                        Input {
                            label: "Street Address 2",
                            value: street2.read().clone(),
                            on_input: move |e: FormEvent| street2.set(e.value()),
                        }
                        Input {
                            label: "City *",
                            value: city.read().clone(),
                            on_input: move |e: FormEvent| city.set(e.value()),
                        }
                        Input {
                            label: "State *",
                            value: state.read().clone(),
                            on_input: move |e: FormEvent| state.set(e.value()),
                        }
                        Input {
                            label: "ZIP Code *",
                            value: zip_code.read().clone(),
                            on_input: move |e: FormEvent| zip_code.set(e.value()),
                        }
                        Input {
                            label: "Country *",
                            value: country.read().clone(),
                            on_input: move |e: FormEvent| country.set(e.value()),
                        }
                    }

                    Separator {}

                    SheetFooter {
                        div {
                            class: "sheet-footer-actions",
                            SheetClose { on_close: try_close }
                            button {
                                class: "button",
                                "data-style": "primary",
                                r#type: "submit",
                                disabled: *in_flight.read(),
                                if *in_flight.read() { "Saving..." } else { "{submit_label}" }
                            }
                        }
                    }
                }
            }
        }

        // Discard changes confirmation
        AlertDialogRoot {
            open: *show_discard.read(),
            on_open_change: move |open: bool| show_discard.set(open),
            AlertDialogContent {
                AlertDialogTitle { "Discard changes?" }
                AlertDialogDescription {
                    "You have unsaved changes. Are you sure you want to close without saving?"
                }
                AlertDialogActions {
                    AlertDialogCancel { "Keep Editing" }
                    AlertDialogAction {
                        on_click: move |_| {
                            show_discard.set(false);
                            on_close.call(());
                        },
                        "Discard"
                    }
                }
            }
        }
    }
}

/// Build a JSON snapshot string for dirty-state comparison.
fn snapshot(
    bar_number: &Signal<String>,
    first_name: &Signal<String>,
    last_name: &Signal<String>,
    middle_name: &Signal<String>,
    email: &Signal<String>,
    phone: &Signal<String>,
    firm_name: &Signal<String>,
    fax: &Signal<String>,
    street1: &Signal<String>,
    street2: &Signal<String>,
    city: &Signal<String>,
    state: &Signal<String>,
    zip_code: &Signal<String>,
    country: &Signal<String>,
) -> String {
    serde_json::json!({
        "bar_number": bar_number.read().clone(),
        "first_name": first_name.read().clone(),
        "last_name": last_name.read().clone(),
        "middle_name": middle_name.read().clone(),
        "email": email.read().clone(),
        "phone": phone.read().clone(),
        "firm_name": firm_name.read().clone(),
        "fax": fax.read().clone(),
        "street1": street1.read().clone(),
        "street2": street2.read().clone(),
        "city": city.read().clone(),
        "state": state.read().clone(),
        "zip_code": zip_code.read().clone(),
        "country": country.read().clone(),
    })
    .to_string()
}

/// Return `serde_json::Value::Null` for empty strings, or the string value.
fn opt_str(s: &str) -> serde_json::Value {
    if s.trim().is_empty() {
        serde_json::Value::Null
    } else {
        serde_json::Value::String(s.to_string())
    }
}
