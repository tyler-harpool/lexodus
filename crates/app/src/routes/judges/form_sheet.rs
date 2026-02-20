use dioxus::prelude::*;
use shared_types::{
    CreateJudgeRequest, JudgeResponse, UpdateJudgeRequest, JUDGE_STATUSES, JUDGE_TITLES,
};
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

/// Unified create/edit form for judges, rendered inside a Sheet.
#[component]
pub fn JudgeFormSheet(
    mode: FormMode,
    initial: Option<JudgeResponse>,
    open: bool,
    on_close: EventHandler<()>,
    on_saved: EventHandler<()>,
) -> Element {
    let ctx = use_context::<CourtContext>();
    let toast = use_toast();

    // --- Form field signals ---
    let mut name = use_signal(String::new);
    let mut title = use_signal(|| "Judge".to_string());
    let mut district = use_signal(String::new);
    let mut status = use_signal(|| "Active".to_string());
    let mut courtroom = use_signal(String::new);
    let mut max_caseload = use_signal(|| "150".to_string());
    let mut specializations = use_signal(String::new);

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
                name.set(data.name.clone());
                title.set(data.title.clone());
                district.set(data.district.clone());
                status.set(data.status.clone());
                courtroom.set(data.courtroom.clone().unwrap_or_default());
                max_caseload.set(data.max_caseload.to_string());
                specializations.set(data.specializations.join(", "));
            }
        } else if mode == FormMode::Create && hydrated_id.read().is_empty() {
            // Already at defaults
        } else if mode == FormMode::Create {
            hydrated_id.set(String::new());
            name.set(String::new());
            title.set("Judge".to_string());
            district.set(String::new());
            status.set("Active".to_string());
            courtroom.set(String::new());
            max_caseload.set("150".to_string());
            specializations.set(String::new());
        }
    });

    // --- Dirty state tracking ---
    let mut initial_snapshot = use_signal(String::new);

    use_effect(move || {
        if open {
            initial_snapshot.set(snapshot(
                &name, &title, &district, &status, &courtroom, &max_caseload, &specializations,
            ));
        }
    });

    let is_dirty = move || {
        let current = snapshot(
            &name, &title, &district, &status, &courtroom, &max_caseload, &specializations,
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

        let parsed_caseload = max_caseload
            .read()
            .parse::<i32>()
            .unwrap_or(150);
        let parsed_specs: Vec<String> = specializations
            .read()
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();

        let courtroom_val = opt_string(&courtroom.read());

        spawn(async move {
            in_flight.set(true);
            let result = match mode {
                FormMode::Create => {
                    let req = CreateJudgeRequest {
                        name: name.read().clone(),
                        title: title.read().clone(),
                        district: district.read().clone(),
                        appointed_date: None,
                        status: None,
                        senior_status_date: None,
                        courtroom: courtroom_val,
                        max_caseload: Some(parsed_caseload),
                        specializations: parsed_specs,
                    };
                    server::api::create_judge(court, req).await
                }
                FormMode::Edit => {
                    let req = UpdateJudgeRequest {
                        name: Some(name.read().clone()),
                        title: Some(title.read().clone()),
                        district: Some(district.read().clone()),
                        appointed_date: None,
                        status: Some(status.read().clone()),
                        senior_status_date: None,
                        courtroom: courtroom_val,
                        max_caseload: Some(parsed_caseload),
                        specializations: Some(parsed_specs),
                    };
                    server::api::update_judge(court, id, req).await
                }
            };
            match result {
                Ok(_) => {
                    on_saved.call(());
                    on_close.call(());
                    let msg = match mode {
                        FormMode::Create => "Judge created successfully",
                        FormMode::Edit => "Judge updated successfully",
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
        FormMode::Create => "New Judge",
        FormMode::Edit => "Edit Judge",
    };
    let description = match mode {
        FormMode::Create => "Add a new judge to this court district.",
        FormMode::Edit => "Modify judge information.",
    };
    let submit_label = match mode {
        FormMode::Create => "Create Judge",
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
                    SheetDescription { "{description}" }
                    SheetClose { on_close: try_close }
                }

                Form {
                    onsubmit: handle_save,

                    div {
                        class: "sheet-form",

                        Input {
                            label: "Full Name *",
                            value: name.read().clone(),
                            on_input: move |e: FormEvent| name.set(e.value()),
                            placeholder: "e.g., Hon. Jane Smith",
                        }

                        FormSelect {
                            label: "Title *",
                            value: title.read().clone(),
                            onchange: move |e: Event<FormData>| title.set(e.value()),
                            for t in JUDGE_TITLES {
                                option { value: *t, "{t}" }
                            }
                        }

                        Input {
                            label: "District *",
                            value: district.read().clone(),
                            on_input: move |e: FormEvent| district.set(e.value()),
                            placeholder: "e.g., Southern District of New York",
                        }

                        if mode == FormMode::Edit {
                            FormSelect {
                                label: "Status",
                                value: status.read().clone(),
                                onchange: move |e: Event<FormData>| status.set(e.value()),
                                for s in JUDGE_STATUSES {
                                    option { value: *s, "{s}" }
                                }
                            }
                        }

                        Separator {}

                        Input {
                            label: "Courtroom",
                            value: courtroom.read().clone(),
                            on_input: move |e: FormEvent| courtroom.set(e.value()),
                            placeholder: "e.g., Courtroom 12B",
                        }

                        Input {
                            label: "Max Caseload",
                            input_type: "number",
                            value: max_caseload.read().clone(),
                            on_input: move |e: FormEvent| max_caseload.set(e.value()),
                        }

                        Input {
                            label: "Specializations",
                            value: specializations.read().clone(),
                            on_input: move |e: FormEvent| specializations.set(e.value()),
                            placeholder: "Comma-separated, e.g., Criminal, Civil Rights",
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
    name: &Signal<String>,
    title: &Signal<String>,
    district: &Signal<String>,
    status: &Signal<String>,
    courtroom: &Signal<String>,
    max_caseload: &Signal<String>,
    specializations: &Signal<String>,
) -> String {
    serde_json::json!({
        "name": name.read().clone(),
        "title": title.read().clone(),
        "district": district.read().clone(),
        "status": status.read().clone(),
        "courtroom": courtroom.read().clone(),
        "max_caseload": max_caseload.read().clone(),
        "specializations": specializations.read().clone(),
    })
    .to_string()
}

/// Converts an empty-or-whitespace string to `None`, otherwise `Some(trimmed)`.
fn opt_string(s: &str) -> Option<String> {
    let trimmed = s.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.to_string())
    }
}
