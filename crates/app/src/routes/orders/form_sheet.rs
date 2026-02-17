use dioxus::prelude::*;
use shared_types::{CaseSearchResponse, JudicialOrderResponse, ORDER_STATUSES, ORDER_TYPES};
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

/// Unified create/edit form for judicial orders, rendered inside a Sheet.
/// Uses PATCH semantics for edit: only sends changed fields.
#[component]
pub fn OrderFormSheet(
    mode: FormMode,
    initial: Option<JudicialOrderResponse>,
    open: bool,
    on_close: EventHandler<()>,
    on_saved: EventHandler<()>,
) -> Element {
    let ctx = use_context::<CourtContext>();
    let toast = use_toast();

    // --- Form field signals ---
    let mut title = use_signal(String::new);
    let mut order_type = use_signal(|| ORDER_TYPES[0].to_string());
    let mut case_id = use_signal(String::new);
    let mut judge_id = use_signal(String::new);
    let mut content = use_signal(String::new);
    let mut status = use_signal(|| "Draft".to_string());

    // --- Initial values for PATCH diff ---
    let mut init_title = use_signal(String::new);
    let mut init_content = use_signal(String::new);
    let mut init_status = use_signal(|| "Draft".to_string());

    // --- Load cases and judges for selectors ---
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

    let judges_for_select = use_resource(move || {
        let court = ctx.court_id.read().clone();
        async move {
            match server::api::list_judges(court).await {
                Ok(json) => serde_json::from_str::<Vec<serde_json::Value>>(&json).ok(),
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
                title.set(data.title.clone());
                order_type.set(data.order_type.clone());
                case_id.set(data.case_id.clone());
                judge_id.set(data.judge_id.clone());
                content.set(data.content.clone());
                status.set(data.status.clone());
                // Capture initial values for PATCH diff
                init_title.set(data.title.clone());
                init_content.set(data.content.clone());
                init_status.set(data.status.clone());
            }
        } else if mode == FormMode::Create && hydrated_id.read().is_empty() {
            // Already at defaults
        } else if mode == FormMode::Create {
            hydrated_id.set(String::new());
            title.set(String::new());
            order_type.set(ORDER_TYPES[0].to_string());
            case_id.set(String::new());
            judge_id.set(String::new());
            content.set(String::new());
            status.set("Draft".to_string());
        }
    });

    // --- Dirty state tracking ---
    let mut initial_snapshot = use_signal(String::new);

    use_effect(move || {
        if open {
            initial_snapshot.set(snapshot(
                &title, &order_type, &case_id, &judge_id, &content, &status,
            ));
        }
    });

    let is_dirty = move || {
        let current = snapshot(&title, &order_type, &case_id, &judge_id, &content, &status);
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
                "case_id": case_id.read().clone(),
                "judge_id": judge_id.read().clone(),
                "order_type": order_type.read().clone(),
                "title": title.read().trim().to_string(),
                "content": content.read().clone(),
                "status": status.read().clone(),
            }),
            FormMode::Edit => {
                // PATCH: only include changed fields
                let mut map = serde_json::Map::new();
                if *title.read() != *init_title.read() {
                    map.insert(
                        "title".into(),
                        serde_json::json!(title.read().trim().to_string()),
                    );
                }
                if *content.read() != *init_content.read() {
                    map.insert("content".into(), serde_json::json!(content.read().clone()));
                }
                if *status.read() != *init_status.read() {
                    map.insert("status".into(), serde_json::json!(status.read().clone()));
                }
                serde_json::Value::Object(map)
            }
        };

        spawn(async move {
            in_flight.set(true);
            let result = match mode {
                FormMode::Create => server::api::create_order(court, body.to_string()).await,
                FormMode::Edit => server::api::update_order(court, id, body.to_string()).await,
            };
            match result {
                Ok(_) => {
                    on_saved.call(());
                    on_close.call(());
                    let msg = match mode {
                        FormMode::Create => "Order created successfully",
                        FormMode::Edit => "Order updated successfully",
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
        FormMode::Create => "New Order",
        FormMode::Edit => "Edit Order",
    };
    let sheet_desc = match mode {
        FormMode::Create => "Create a new judicial order.",
        FormMode::Edit => "Update order information. Only changed fields are saved.",
    };
    let submit_label = match mode {
        FormMode::Create => "Create Order",
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
                            placeholder: "e.g., Scheduling Order",
                        }

                        FormSelect {
                            label: "Order Type *",
                            value: order_type.read().clone(),
                            onchange: move |e: Event<FormData>| order_type.set(e.value()),
                            for ot in ORDER_TYPES.iter() {
                                option { value: *ot, "{ot}" }
                            }
                        }

                        Separator {}

                        // Case selector (only in create mode — case_id is immutable)
                        if mode == FormMode::Create {
                            label { class: "input-label", "Case *" }
                            select {
                                class: "input",
                                value: case_id.read().clone(),
                                onchange: move |e: FormEvent| case_id.set(e.value()),
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

                            label { class: "input-label", "Judge *" }
                            select {
                                class: "input",
                                value: judge_id.read().clone(),
                                onchange: move |e: FormEvent| judge_id.set(e.value()),
                                option { value: "", "-- Select a judge --" }
                                {match &*judges_for_select.read() {
                                    Some(Some(judges)) => rsx! {
                                        for j in judges.iter() {
                                            option {
                                                value: j["id"].as_str().unwrap_or(""),
                                                {j["name"].as_str().unwrap_or("Unknown")}
                                            }
                                        }
                                    },
                                    _ => rsx! {
                                        option { value: "", disabled: true, "Loading judges..." }
                                    },
                                }}
                            }
                        }

                        FormSelect {
                            label: "Status",
                            value: status.read().clone(),
                            onchange: move |e: Event<FormData>| status.set(e.value()),
                            for s in ORDER_STATUSES.iter() {
                                option { value: *s, "{s}" }
                            }
                        }

                        Separator {}

                        Textarea {
                            label: "Content",
                            value: content.read().clone(),
                            on_input: move |e: FormEvent| content.set(e.value()),
                            placeholder: "Order content...",
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
    order_type: &Signal<String>,
    case_id: &Signal<String>,
    judge_id: &Signal<String>,
    content: &Signal<String>,
    status: &Signal<String>,
) -> String {
    serde_json::json!({
        "title": title.read().clone(),
        "order_type": order_type.read().clone(),
        "case_id": case_id.read().clone(),
        "judge_id": judge_id.read().clone(),
        "content": content.read().clone(),
        "status": status.read().clone(),
    })
    .to_string()
}
