use dioxus::prelude::*;
use shared_types::{
    CaseSearchResponse, JudicialOpinionResponse, OPINION_DISPOSITIONS, OPINION_STATUSES,
    OPINION_TYPES,
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

/// Unified create/edit form for opinions, rendered inside a Sheet.
#[component]
pub fn OpinionFormSheet(
    mode: FormMode,
    initial: Option<JudicialOpinionResponse>,
    open: bool,
    on_close: EventHandler<()>,
    on_saved: EventHandler<()>,
) -> Element {
    let ctx = use_context::<CourtContext>();
    let toast = use_toast();

    // --- Form field signals ---
    let mut title = use_signal(String::new);
    let mut opinion_type = use_signal(|| OPINION_TYPES[0].to_string());
    let mut disposition = use_signal(String::new);
    let mut status = use_signal(|| "Draft".to_string());
    let mut case_id = use_signal(String::new);
    let mut case_name = use_signal(String::new);
    let mut docket_number = use_signal(String::new);
    let mut judge_id = use_signal(String::new);
    let mut judge_name = use_signal(String::new);
    let mut syllabus = use_signal(String::new);
    let mut content = use_signal(String::new);
    let mut keywords_text = use_signal(String::new);

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
                title.set(data.title.clone());
                opinion_type.set(data.opinion_type.clone());
                disposition.set(data.disposition.clone());
                status.set(data.status.clone());
                case_id.set(data.case_id.clone());
                case_name.set(data.case_name.clone());
                docket_number.set(data.docket_number.clone());
                judge_id.set(data.author_judge_id.clone());
                judge_name.set(data.author_judge_name.clone());
                syllabus.set(data.syllabus.clone());
                content.set(data.content.clone());
                keywords_text.set(data.keywords.join(", "));
            }
        } else if mode == FormMode::Create && hydrated_id.read().is_empty() {
            // Already at defaults for create
        } else if mode == FormMode::Create {
            hydrated_id.set(String::new());
            title.set(String::new());
            opinion_type.set(OPINION_TYPES[0].to_string());
            disposition.set(String::new());
            status.set("Draft".to_string());
            case_id.set(String::new());
            case_name.set(String::new());
            docket_number.set(String::new());
            judge_id.set(String::new());
            judge_name.set(String::new());
            syllabus.set(String::new());
            content.set(String::new());
            keywords_text.set(String::new());
        }
    });

    // --- Dirty state tracking ---
    let mut initial_snapshot = use_signal(String::new);

    use_effect(move || {
        if open {
            initial_snapshot.set(snapshot(
                &title,
                &opinion_type,
                &disposition,
                &status,
                &syllabus,
                &content,
                &keywords_text,
            ));
        }
    });

    let is_dirty = move || {
        let current = snapshot(
            &title,
            &opinion_type,
            &disposition,
            &status,
            &syllabus,
            &content,
            &keywords_text,
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

        let keywords: Vec<String> = keywords_text
            .read()
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();

        match mode {
            FormMode::Create => {
                if title.read().trim().is_empty() {
                    toast.error("Title is required.".to_string(), ToastOptions::new());
                    return;
                }
                if case_id.read().is_empty() {
                    toast.error("Case is required.".to_string(), ToastOptions::new());
                    return;
                }
                if judge_id.read().is_empty() {
                    toast.error(
                        "Author judge is required.".to_string(),
                        ToastOptions::new(),
                    );
                    return;
                }

                let body = serde_json::json!({
                    "case_id": case_id.read().clone(),
                    "case_name": case_name.read().clone(),
                    "docket_number": docket_number.read().clone(),
                    "author_judge_id": judge_id.read().clone(),
                    "author_judge_name": judge_name.read().clone(),
                    "opinion_type": opinion_type.read().clone(),
                    "title": title.read().trim().to_string(),
                    "content": content.read().clone(),
                    "disposition": opt_str(&disposition.read()),
                    "syllabus": opt_str(&syllabus.read()),
                    "status": status.read().clone(),
                    "keywords": keywords,
                });

                spawn(async move {
                    in_flight.set(true);
                    match server::api::create_opinion(court, body.to_string()).await {
                        Ok(_) => {
                            on_saved.call(());
                            on_close.call(());
                            toast.success(
                                "Opinion created successfully".to_string(),
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
                    if title.read().trim() != init.title {
                        body.insert(
                            "title".into(),
                            serde_json::Value::String(title.read().trim().to_string()),
                        );
                    }
                    if *content.read() != init.content {
                        body.insert(
                            "content".into(),
                            serde_json::Value::String(content.read().clone()),
                        );
                    }
                    if *status.read() != init.status {
                        body.insert(
                            "status".into(),
                            serde_json::Value::String(status.read().clone()),
                        );
                    }
                    if *disposition.read() != init.disposition {
                        body.insert("disposition".into(), opt_str(&disposition.read()));
                    }
                    if *syllabus.read() != init.syllabus {
                        body.insert("syllabus".into(), opt_str(&syllabus.read()));
                    }
                    let kw_changed = keywords != init.keywords;
                    if kw_changed {
                        body.insert(
                            "keywords".into(),
                            serde_json::Value::Array(
                                keywords
                                    .iter()
                                    .map(|k| serde_json::Value::String(k.clone()))
                                    .collect(),
                            ),
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
                    match server::api::update_opinion(court, id, payload).await {
                        Ok(_) => {
                            on_saved.call(());
                            on_close.call(());
                            toast.success(
                                "Opinion updated successfully".to_string(),
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
        FormMode::Create => "New Opinion",
        FormMode::Edit => "Edit Opinion",
    };
    let description = match mode {
        FormMode::Create => "Draft a new judicial opinion.",
        FormMode::Edit => "Modify opinion details.",
    };
    let submit_label = match mode {
        FormMode::Create => "Create Opinion",
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
                            label: "Title *",
                            value: title.read().clone(),
                            on_input: move |e: FormEvent| title.set(e.value()),
                            placeholder: "e.g., United States v. Smith",
                        }

                        FormSelect {
                            label: "Opinion Type *",
                            value: "{opinion_type}",
                            onchange: move |e: Event<FormData>| opinion_type.set(e.value()),
                            for ot in OPINION_TYPES.iter() {
                                option { value: *ot, "{ot}" }
                            }
                        }

                        FormSelect {
                            label: "Status",
                            value: "{status}",
                            onchange: move |e: Event<FormData>| status.set(e.value()),
                            for s in OPINION_STATUSES.iter() {
                                option { value: *s, "{s}" }
                            }
                        }

                        FormSelect {
                            label: "Disposition",
                            value: "{disposition}",
                            onchange: move |e: Event<FormData>| disposition.set(e.value()),
                            option { value: "", "-- None --" }
                            for d in OPINION_DISPOSITIONS.iter() {
                                option { value: *d, "{d}" }
                            }
                        }

                        if mode == FormMode::Create {
                            // Case selector (only for create; case can't change on edit)
                            label { class: "input-label", "Case *" }
                            select {
                                class: "input",
                                value: case_id.read().clone(),
                                onchange: move |e: FormEvent| {
                                    let selected_id = e.value().to_string();
                                    case_id.set(selected_id.clone());
                                    if let Some(Some(cases)) = &*cases_for_select.read() {
                                        if let Some(c) = cases.iter().find(|c| c.id == selected_id) {
                                            case_name.set(c.title.clone());
                                            docket_number.set(c.case_number.clone());
                                        }
                                    }
                                },
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

                            // Judge selector (only for create)
                            label { class: "input-label", "Author Judge *" }
                            select {
                                class: "input",
                                value: judge_id.read().clone(),
                                onchange: move |e: FormEvent| {
                                    let selected_id = e.value().to_string();
                                    judge_id.set(selected_id.clone());
                                    if let Some(Some(judges)) = &*judges_for_select.read() {
                                        if let Some(j) = judges.iter().find(|j| {
                                            j["id"].as_str().unwrap_or("") == selected_id
                                        }) {
                                            judge_name.set(
                                                j["name"].as_str().unwrap_or("").to_string(),
                                            );
                                        }
                                    }
                                },
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

                        Separator {}

                        label { class: "input-label", "Syllabus" }
                        textarea {
                            class: "input",
                            rows: 3,
                            value: syllabus.read().clone(),
                            oninput: move |e: FormEvent| syllabus.set(e.value().to_string()),
                            placeholder: "Brief summary of the opinion...",
                        }

                        label { class: "input-label", "Content" }
                        textarea {
                            class: "input",
                            rows: 6,
                            value: content.read().clone(),
                            oninput: move |e: FormEvent| content.set(e.value().to_string()),
                            placeholder: "Opinion content...",
                        }

                        Input {
                            label: "Keywords (comma-separated)",
                            value: keywords_text.read().clone(),
                            on_input: move |e: FormEvent| keywords_text.set(e.value()),
                            placeholder: "e.g., due process, Fourth Amendment",
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
    title: &Signal<String>,
    opinion_type: &Signal<String>,
    disposition: &Signal<String>,
    status: &Signal<String>,
    syllabus: &Signal<String>,
    content: &Signal<String>,
    keywords_text: &Signal<String>,
) -> String {
    serde_json::json!({
        "title": title.read().clone(),
        "opinion_type": opinion_type.read().clone(),
        "disposition": disposition.read().clone(),
        "status": status.read().clone(),
        "syllabus": syllabus.read().clone(),
        "content": content.read().clone(),
        "keywords_text": keywords_text.read().clone(),
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
