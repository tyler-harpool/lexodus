use dioxus::prelude::*;
use shared_types::{CaseSearchResponse, EvidenceResponse, EVIDENCE_TYPES};
use shared_ui::components::{
    AlertDialogAction, AlertDialogActions, AlertDialogCancel, AlertDialogContent,
    AlertDialogDescription, AlertDialogRoot, AlertDialogTitle, Form, FormSelect, Input, Separator,
    Sheet, SheetClose, SheetContent, SheetDescription, SheetFooter, SheetHeader, SheetSide,
    SheetTitle,
};
use shared_ui::{use_toast, ToastOptions};

use crate::CourtContext;

/// Controls whether the form is in Create or Edit mode.
#[derive(Clone, Copy, PartialEq)]
pub enum FormMode {
    Create,
    Edit,
}

/// Unified create/edit form for evidence items, rendered inside a Sheet.
#[component]
pub fn EvidenceFormSheet(
    mode: FormMode,
    initial: Option<EvidenceResponse>,
    open: bool,
    on_close: EventHandler<()>,
    on_saved: EventHandler<()>,
) -> Element {
    let ctx = use_context::<CourtContext>();
    let toast = use_toast();

    // --- Form field signals ---
    let mut description = use_signal(String::new);
    let mut case_id = use_signal(String::new);
    let mut evidence_type = use_signal(|| "Physical".to_string());
    let mut seized_date = use_signal(String::new);
    let mut seized_by = use_signal(String::new);
    let mut location = use_signal(String::new);
    let mut is_sealed = use_signal(|| false);

    // --- Load cases for selector ---
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
                description.set(data.description.clone());
                case_id.set(data.case_id.clone());
                evidence_type.set(data.evidence_type.clone());
                seized_date.set(
                    data.seized_date
                        .as_deref()
                        .map(|d| d.chars().take(10).collect::<String>())
                        .unwrap_or_default(),
                );
                seized_by.set(data.seized_by.clone().unwrap_or_default());
                location.set(data.location.clone());
                is_sealed.set(data.is_sealed);
            }
        } else if mode == FormMode::Create && hydrated_id.read().is_empty() {
            // Already at defaults
        } else if mode == FormMode::Create {
            hydrated_id.set(String::new());
            description.set(String::new());
            case_id.set(String::new());
            evidence_type.set("Physical".to_string());
            seized_date.set(String::new());
            seized_by.set(String::new());
            location.set(String::new());
            is_sealed.set(false);
        }
    });

    // --- Dirty state ---
    let mut initial_snapshot = use_signal(String::new);

    use_effect(move || {
        if open {
            initial_snapshot.set(snapshot(
                &description,
                &evidence_type,
                &seized_date,
                &seized_by,
                &location,
                &is_sealed,
            ));
        }
    });

    let is_dirty = move || {
        let current = snapshot(
            &description,
            &evidence_type,
            &seized_date,
            &seized_by,
            &location,
            &is_sealed,
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

        if description.read().trim().is_empty() {
            toast.error("Description is required.".to_string(), ToastOptions::new());
            return;
        }

        match mode {
            FormMode::Create => {
                if case_id.read().is_empty() {
                    toast.error("Case is required.".to_string(), ToastOptions::new());
                    return;
                }

                let body = serde_json::json!({
                    "case_id": case_id.read().clone(),
                    "description": description.read().trim().to_string(),
                    "evidence_type": evidence_type.read().clone(),
                    "seized_date": opt_date(&seized_date.read()),
                    "seized_by": opt_str(&seized_by.read()),
                    "location": opt_str(&location.read()),
                    "is_sealed": *is_sealed.read(),
                });

                spawn(async move {
                    in_flight.set(true);
                    match server::api::create_evidence(court, body.to_string()).await {
                        Ok(_) => {
                            on_saved.call(());
                            on_close.call(());
                            toast.success(
                                "Evidence created successfully".to_string(),
                                ToastOptions::new(),
                            );
                        }
                        Err(e) => {
                            toast.error(format!("{e}"), ToastOptions::new());
                        }
                    }
                    in_flight.set(false);
                });
            }
            FormMode::Edit => {
                // PATCH: only send changed fields
                let mut body = serde_json::Map::new();
                if let Some(ref init) = initial.as_ref() {
                    if description.read().trim() != init.description {
                        body.insert(
                            "description".into(),
                            serde_json::Value::String(description.read().trim().to_string()),
                        );
                    }
                    if *evidence_type.read() != init.evidence_type {
                        body.insert(
                            "evidence_type".into(),
                            serde_json::Value::String(evidence_type.read().clone()),
                        );
                    }
                    let current_seized_date = seized_date.read().trim().to_string();
                    let init_seized_date = init
                        .seized_date
                        .as_deref()
                        .map(|d| d.chars().take(10).collect::<String>())
                        .unwrap_or_default();
                    if current_seized_date != init_seized_date {
                        body.insert("seized_date".into(), opt_date(&seized_date.read()));
                    }
                    if seized_by.read().trim() != init.seized_by.as_deref().unwrap_or("") {
                        body.insert("seized_by".into(), opt_str(&seized_by.read()));
                    }
                    if *location.read() != init.location {
                        body.insert(
                            "location".into(),
                            serde_json::Value::String(location.read().clone()),
                        );
                    }
                    if *is_sealed.read() != init.is_sealed {
                        body.insert(
                            "is_sealed".into(),
                            serde_json::Value::Bool(*is_sealed.read()),
                        );
                    }
                }

                if body.is_empty() {
                    on_close.call(());
                    return;
                }

                let payload = serde_json::Value::Object(body).to_string();
                spawn(async move {
                    in_flight.set(true);
                    match server::api::update_evidence(court, id, payload).await {
                        Ok(_) => {
                            on_saved.call(());
                            on_close.call(());
                            toast.success(
                                "Evidence updated successfully".to_string(),
                                ToastOptions::new(),
                            );
                        }
                        Err(e) => {
                            toast.error(format!("{e}"), ToastOptions::new());
                        }
                    }
                    in_flight.set(false);
                });
            }
        }
    };

    // --- Render ---
    let sheet_title = match mode {
        FormMode::Create => "New Evidence",
        FormMode::Edit => "Edit Evidence",
    };
    let desc = match mode {
        FormMode::Create => "Add an evidence item to an existing case.",
        FormMode::Edit => "Modify evidence details.",
    };
    let submit_label = match mode {
        FormMode::Create => "Create Evidence",
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
                    SheetDescription { "{desc}" }
                    SheetClose { on_close: try_close }
                }

                Form {
                    onsubmit: handle_save,

                    div {
                        class: "sheet-form",

                        if mode == FormMode::Create {
                            label { class: "input-label", "Case *" }
                            select {
                                class: "input",
                                value: case_id.read().clone(),
                                onchange: move |e: FormEvent| case_id.set(e.value().to_string()),
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
                        }

                        Input {
                            label: "Description *",
                            value: description.read().clone(),
                            on_input: move |e: FormEvent| description.set(e.value()),
                            placeholder: "e.g., Recovered laptop from suspect's residence",
                        }

                        FormSelect {
                            label: "Evidence Type",
                            value: "{evidence_type}",
                            onchange: move |e: Event<FormData>| evidence_type.set(e.value()),
                            for et in EVIDENCE_TYPES.iter() {
                                option { value: *et, "{et}" }
                            }
                        }

                        Separator {}

                        Input {
                            label: "Seized Date",
                            input_type: "date",
                            value: seized_date.read().clone(),
                            on_input: move |e: FormEvent| seized_date.set(e.value()),
                        }

                        Input {
                            label: "Seized By",
                            value: seized_by.read().clone(),
                            on_input: move |e: FormEvent| seized_by.set(e.value()),
                            placeholder: "e.g., Special Agent Smith",
                        }

                        Input {
                            label: "Storage Location",
                            value: location.read().clone(),
                            on_input: move |e: FormEvent| location.set(e.value()),
                            placeholder: "e.g., Evidence Locker B-12",
                        }

                        div { class: "checkbox-row",
                            style: "display: flex; align-items: center; gap: var(--space-sm);",
                            input {
                                r#type: "checkbox",
                                checked: *is_sealed.read(),
                                onchange: move |e: FormEvent| is_sealed.set(e.value() == "true"),
                            }
                            label { "Sealed" }
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
    description: &Signal<String>,
    evidence_type: &Signal<String>,
    seized_date: &Signal<String>,
    seized_by: &Signal<String>,
    location: &Signal<String>,
    is_sealed: &Signal<bool>,
) -> String {
    serde_json::json!({
        "description": description.read().clone(),
        "evidence_type": evidence_type.read().clone(),
        "seized_date": seized_date.read().clone(),
        "seized_by": seized_by.read().clone(),
        "location": location.read().clone(),
        "is_sealed": *is_sealed.read(),
    })
    .to_string()
}

fn opt_str(s: &str) -> serde_json::Value {
    if s.trim().is_empty() {
        serde_json::Value::Null
    } else {
        serde_json::Value::String(s.to_string())
    }
}

fn opt_date(s: &str) -> serde_json::Value {
    if s.trim().is_empty() {
        serde_json::Value::Null
    } else {
        serde_json::Value::String(s.to_string())
    }
}
