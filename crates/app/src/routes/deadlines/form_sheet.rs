use dioxus::prelude::*;
use shared_types::{DeadlineResponse, DEADLINE_STATUSES};
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

/// Unified create/edit form for deadlines, rendered inside a Sheet.
/// Uses PUT semantics for updates.
#[component]
pub fn DeadlineFormSheet(
    mode: FormMode,
    initial: Option<DeadlineResponse>,
    open: bool,
    on_close: EventHandler<()>,
    on_saved: EventHandler<()>,
) -> Element {
    let ctx = use_context::<CourtContext>();
    let toast = use_toast();

    // --- Form field signals ---
    let mut title = use_signal(String::new);
    let mut due_at = use_signal(String::new);
    let mut status = use_signal(|| "open".to_string());
    let mut rule_code = use_signal(String::new);
    let mut notes = use_signal(String::new);

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
                // Convert RFC3339 to datetime-local format for input
                due_at.set(rfc3339_to_datetime_local(&data.due_at));
                status.set(data.status.clone());
                rule_code.set(data.rule_code.clone().unwrap_or_default());
                notes.set(data.notes.clone().unwrap_or_default());
            }
        } else if mode == FormMode::Create && hydrated_id.read().is_empty() {
            // Already at defaults
        } else if mode == FormMode::Create {
            hydrated_id.set(String::new());
            title.set(String::new());
            due_at.set(String::new());
            status.set("open".to_string());
            rule_code.set(String::new());
            notes.set(String::new());
        }
    });

    // --- Dirty state tracking ---
    let mut initial_snapshot = use_signal(String::new);

    use_effect(move || {
        if open {
            initial_snapshot.set(snapshot(&title, &due_at, &status, &rule_code, &notes));
        }
    });

    let is_dirty = move || {
        let current = snapshot(&title, &due_at, &status, &rule_code, &notes);
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

        let t = title.read().clone();
        let d = due_at.read().clone();
        let r = rule_code.read().clone();
        let n = notes.read().clone();
        let s = status.read().clone();

        if t.trim().is_empty() || d.trim().is_empty() {
            toast.error(
                "Title and due date are required.".to_string(),
                ToastOptions::new(),
            );
            return;
        }

        // Convert HTML datetime-local to RFC3339
        let due_rfc3339 = format!("{}:00Z", d);

        let body = match mode {
            FormMode::Create => serde_json::json!({
                "title": t.trim(),
                "due_at": due_rfc3339,
                "rule_code": opt_str(&r),
                "notes": opt_str(&n),
            }),
            FormMode::Edit => serde_json::json!({
                "title": t.trim(),
                "due_at": due_rfc3339,
                "status": s,
                "rule_code": opt_str(&r),
                "notes": opt_str(&n),
            }),
        };

        spawn(async move {
            in_flight.set(true);
            let result = match mode {
                FormMode::Create => server::api::create_deadline(court, body.to_string()).await,
                FormMode::Edit => server::api::update_deadline(court, id, body.to_string()).await,
            };
            match result {
                Ok(_) => {
                    on_saved.call(());
                    on_close.call(());
                    let msg = match mode {
                        FormMode::Create => "Deadline created successfully",
                        FormMode::Edit => "Deadline updated successfully",
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
        FormMode::Create => "New Deadline",
        FormMode::Edit => "Edit Deadline",
    };
    let sheet_desc = match mode {
        FormMode::Create => "Set a deadline with an associated rule and due date.",
        FormMode::Edit => "Update deadline information.",
    };
    let submit_label = match mode {
        FormMode::Create => "Create Deadline",
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
                            placeholder: "e.g., File Motion Response",
                        }

                        Input {
                            label: "Due Date *",
                            input_type: "datetime-local",
                            value: due_at.read().clone(),
                            on_input: move |e: FormEvent| due_at.set(e.value()),
                        }

                        if mode == FormMode::Edit {
                            FormSelect {
                                label: "Status",
                                value: status.read().clone(),
                                onchange: move |e: Event<FormData>| status.set(e.value()),
                                for s in DEADLINE_STATUSES {
                                    option { value: *s, "{s}" }
                                }
                            }
                        }

                        Separator {}

                        Input {
                            label: "Rule Code",
                            value: rule_code.read().clone(),
                            on_input: move |e: FormEvent| rule_code.set(e.value()),
                            placeholder: "e.g., FRCP 12(b)",
                        }

                        Textarea {
                            label: "Notes",
                            value: notes.read().clone(),
                            on_input: move |e: FormEvent| notes.set(e.value()),
                            placeholder: "Optional notes...",
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

/// Returns `Value::Null` for empty strings, otherwise the string value.
fn opt_str(s: &str) -> serde_json::Value {
    if s.trim().is_empty() {
        serde_json::Value::Null
    } else {
        serde_json::json!(s.trim())
    }
}

/// Converts RFC3339 timestamp to HTML datetime-local format (YYYY-MM-DDTHH:MM).
fn rfc3339_to_datetime_local(rfc: &str) -> String {
    if rfc.len() >= 16 {
        rfc[..16].to_string()
    } else {
        rfc.to_string()
    }
}

fn snapshot(
    title: &Signal<String>,
    due_at: &Signal<String>,
    status: &Signal<String>,
    rule_code: &Signal<String>,
    notes: &Signal<String>,
) -> String {
    serde_json::json!({
        "title": title.read().clone(),
        "due_at": due_at.read().clone(),
        "status": status.read().clone(),
        "rule_code": rule_code.read().clone(),
        "notes": notes.read().clone(),
    })
    .to_string()
}
