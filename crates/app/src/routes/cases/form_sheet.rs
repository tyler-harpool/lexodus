use dioxus::prelude::*;
use shared_types::{CaseResponse, CASE_PRIORITIES, CASE_STATUSES, CRIME_TYPES};
use shared_ui::components::{
    AlertDialogAction, AlertDialogActions, AlertDialogCancel, AlertDialogContent,
    AlertDialogDescription, AlertDialogRoot, AlertDialogTitle, Form, FormSelect, Input, Separator,
    Sheet, SheetClose, SheetContent, SheetDescription, SheetFooter, SheetHeader, SheetSide,
    SheetTitle, Textarea,
};
use shared_ui::{use_toast, ToastOptions};

use crate::CourtContext;

/// Controls whether the form is in Create or Edit mode.
#[derive(Clone, Copy, PartialEq)]
pub enum FormMode {
    Create,
    Edit,
}

/// Unified create/edit form for cases, rendered inside a Sheet.
/// Uses PATCH semantics for edit: only sends changed fields.
#[component]
pub fn CaseFormSheet(
    mode: FormMode,
    initial: Option<CaseResponse>,
    open: bool,
    on_close: EventHandler<()>,
    on_saved: EventHandler<()>,
) -> Element {
    let ctx = use_context::<CourtContext>();
    let toast = use_toast();

    // --- Form field signals ---
    let mut title = use_signal(String::new);
    let mut description = use_signal(String::new);
    let mut crime_type = use_signal(|| "fraud".to_string());
    let mut status = use_signal(|| "filed".to_string());
    let mut priority = use_signal(|| "medium".to_string());
    let mut location = use_signal(String::new);
    let mut district_code = use_signal(String::new);

    // --- Initial values for PATCH diff ---
    let mut init_title = use_signal(String::new);
    let mut init_description = use_signal(String::new);
    let mut init_crime_type = use_signal(|| "fraud".to_string());
    let mut init_status = use_signal(|| "filed".to_string());
    let mut init_priority = use_signal(|| "medium".to_string());
    let mut init_location = use_signal(String::new);
    let mut init_district_code = use_signal(String::new);

    // --- Hydration ---
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
                title.set(data.title.clone());
                description.set(data.description.clone());
                crime_type.set(data.crime_type.clone());
                status.set(data.status.clone());
                priority.set(data.priority.clone());
                location.set(data.location.clone());
                district_code.set(data.district_code.clone());
                // Capture initial values for PATCH diff
                init_title.set(data.title.clone());
                init_description.set(data.description.clone());
                init_crime_type.set(data.crime_type.clone());
                init_status.set(data.status.clone());
                init_priority.set(data.priority.clone());
                init_location.set(data.location.clone());
                init_district_code.set(data.district_code.clone());
            }
        } else if mode == FormMode::Create && hydrated_id.read().is_empty() {
            // Already at defaults
        } else if mode == FormMode::Create {
            hydrated_id.set(String::new());
            title.set(String::new());
            description.set(String::new());
            crime_type.set("fraud".to_string());
            status.set("filed".to_string());
            priority.set("medium".to_string());
            location.set(String::new());
            district_code.set(String::new());
        }
    });

    // --- Dirty state tracking ---
    let mut initial_snapshot = use_signal(String::new);

    use_effect(move || {
        if open {
            initial_snapshot.set(snapshot(
                &title, &description, &crime_type, &status, &priority, &location, &district_code,
            ));
        }
    });

    let is_dirty = move || {
        let current = snapshot(
            &title, &description, &crime_type, &status, &priority, &location, &district_code,
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

        let body = match mode {
            FormMode::Create => serde_json::json!({
                "title": title.read().clone(),
                "description": description.read().clone(),
                "crime_type": crime_type.read().clone(),
                "priority": priority.read().clone(),
                "location": location.read().clone(),
                "district_code": district_code.read().clone(),
            }),
            FormMode::Edit => {
                // PATCH: only include changed fields
                let mut map = serde_json::Map::new();
                if *title.read() != *init_title.read() {
                    map.insert("title".into(), serde_json::json!(title.read().clone()));
                }
                if *description.read() != *init_description.read() {
                    map.insert(
                        "description".into(),
                        serde_json::json!(description.read().clone()),
                    );
                }
                if *crime_type.read() != *init_crime_type.read() {
                    map.insert(
                        "crime_type".into(),
                        serde_json::json!(crime_type.read().clone()),
                    );
                }
                if *status.read() != *init_status.read() {
                    map.insert("status".into(), serde_json::json!(status.read().clone()));
                }
                if *priority.read() != *init_priority.read() {
                    map.insert(
                        "priority".into(),
                        serde_json::json!(priority.read().clone()),
                    );
                }
                if *location.read() != *init_location.read() {
                    map.insert(
                        "location".into(),
                        serde_json::json!(location.read().clone()),
                    );
                }
                if *district_code.read() != *init_district_code.read() {
                    map.insert(
                        "district_code".into(),
                        serde_json::json!(district_code.read().clone()),
                    );
                }
                serde_json::Value::Object(map)
            }
        };

        spawn(async move {
            in_flight.set(true);
            let result = match mode {
                FormMode::Create => server::api::create_case(court, body.to_string()).await,
                FormMode::Edit => server::api::update_case(court, id, body.to_string()).await,
            };
            match result {
                Ok(_) => {
                    on_saved.call(());
                    on_close.call(());
                    let msg = match mode {
                        FormMode::Create => "Case created successfully",
                        FormMode::Edit => "Case updated successfully",
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
    let sheet_title = match mode {
        FormMode::Create => "New Case",
        FormMode::Edit => "Edit Case",
    };
    let sheet_desc = match mode {
        FormMode::Create => "File a new criminal case.",
        FormMode::Edit => "Update case information. Only changed fields are saved.",
    };
    let submit_label = match mode {
        FormMode::Create => "Create Case",
        FormMode::Edit => "Save Changes",
    };

    rsx! {
        Sheet {
            open,
            on_close: try_close,
            side: SheetSide::Right,
            SheetContent {
                SheetHeader {
                    SheetTitle { "{sheet_title}" }
                    SheetDescription { "{sheet_desc}" }
                    SheetClose { on_close: try_close }
                }

                Form {
                    onsubmit: handle_save,

                    div {
                        class: "sheet-form",

                        Input {
                            label: "Title *",
                            value: title.read().clone(),
                            on_input: move |e: FormEvent| title.set(e.value()),
                            placeholder: "e.g., United States v. Smith",
                        }

                        Textarea {
                            label: "Description",
                            value: description.read().clone(),
                            on_input: move |e: FormEvent| description.set(e.value()),
                            placeholder: "Brief case description...",
                        }

                        Separator {}

                        FormSelect {
                            label: "Crime Type *",
                            value: crime_type.read().clone(),
                            onchange: move |e: Event<FormData>| crime_type.set(e.value()),
                            for ct in CRIME_TYPES {
                                option { value: *ct, "{ct}" }
                            }
                        }

                        if mode == FormMode::Edit {
                            FormSelect {
                                label: "Status",
                                value: status.read().clone(),
                                onchange: move |e: Event<FormData>| status.set(e.value()),
                                for s in CASE_STATUSES {
                                    option { value: *s, "{s}" }
                                }
                            }
                        }

                        FormSelect {
                            label: "Priority",
                            value: priority.read().clone(),
                            onchange: move |e: Event<FormData>| priority.set(e.value()),
                            for p in CASE_PRIORITIES {
                                option { value: *p, "{p}" }
                            }
                        }

                        Separator {}

                        Input {
                            label: "Location",
                            value: location.read().clone(),
                            on_input: move |e: FormEvent| location.set(e.value()),
                            placeholder: "e.g., Federal Courthouse, Room 401",
                        }

                        Input {
                            label: "District Code *",
                            value: district_code.read().clone(),
                            on_input: move |e: FormEvent| district_code.set(e.value()),
                            placeholder: "e.g., SDNY",
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

fn snapshot(
    title: &Signal<String>,
    description: &Signal<String>,
    crime_type: &Signal<String>,
    status: &Signal<String>,
    priority: &Signal<String>,
    location: &Signal<String>,
    district_code: &Signal<String>,
) -> String {
    serde_json::json!({
        "title": title.read().clone(),
        "description": description.read().clone(),
        "crime_type": crime_type.read().clone(),
        "status": status.read().clone(),
        "priority": priority.read().clone(),
        "location": location.read().clone(),
        "district_code": district_code.read().clone(),
    })
    .to_string()
}
