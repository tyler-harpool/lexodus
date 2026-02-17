use dioxus::prelude::*;
use shared_types::{
    CaseSearchResponse, DefendantResponse, BAIL_TYPES, CITIZENSHIP_STATUSES, CUSTODY_STATUSES,
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

/// Unified create/edit form for defendants, rendered inside a Sheet.
#[component]
pub fn DefendantFormSheet(
    mode: FormMode,
    initial: Option<DefendantResponse>,
    open: bool,
    on_close: EventHandler<()>,
    on_saved: EventHandler<()>,
) -> Element {
    let ctx = use_context::<CourtContext>();
    let toast = use_toast();

    // --- Form field signals ---
    let mut name = use_signal(String::new);
    let mut case_id = use_signal(String::new);
    let mut usm_number = use_signal(String::new);
    let mut fbi_number = use_signal(String::new);
    let mut date_of_birth = use_signal(String::new);
    let mut citizenship_status = use_signal(|| "Unknown".to_string());
    let mut custody_status = use_signal(|| "Released".to_string());
    let mut aliases_text = use_signal(String::new);
    let mut bail_type = use_signal(String::new);
    let mut bail_amount = use_signal(String::new);
    let mut surety_name = use_signal(String::new);

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
                name.set(data.name.clone());
                case_id.set(data.case_id.clone());
                usm_number.set(data.usm_number.clone().unwrap_or_default());
                fbi_number.set(data.fbi_number.clone().unwrap_or_default());
                date_of_birth.set(
                    data.date_of_birth
                        .clone()
                        .unwrap_or_default(),
                );
                citizenship_status.set(data.citizenship_status.clone());
                custody_status.set(data.custody_status.clone());
                aliases_text.set(data.aliases.join(", "));
                bail_type.set(data.bail_type.clone().unwrap_or_default());
                bail_amount.set(
                    data.bail_amount
                        .map(|a| format!("{:.2}", a))
                        .unwrap_or_default(),
                );
                surety_name.set(data.surety_name.clone().unwrap_or_default());
            }
        } else if mode == FormMode::Create && hydrated_id.read().is_empty() {
            // Already at defaults
        } else if mode == FormMode::Create {
            hydrated_id.set(String::new());
            name.set(String::new());
            case_id.set(String::new());
            usm_number.set(String::new());
            fbi_number.set(String::new());
            date_of_birth.set(String::new());
            citizenship_status.set("Unknown".to_string());
            custody_status.set("Released".to_string());
            aliases_text.set(String::new());
            bail_type.set(String::new());
            bail_amount.set(String::new());
            surety_name.set(String::new());
        }
    });

    // --- Dirty state ---
    let mut initial_snapshot = use_signal(String::new);

    use_effect(move || {
        if open {
            initial_snapshot.set(snapshot(
                &name,
                &usm_number,
                &fbi_number,
                &date_of_birth,
                &citizenship_status,
                &custody_status,
                &aliases_text,
                &bail_type,
                &bail_amount,
                &surety_name,
            ));
        }
    });

    let is_dirty = move || {
        let current = snapshot(
            &name,
            &usm_number,
            &fbi_number,
            &date_of_birth,
            &citizenship_status,
            &custody_status,
            &aliases_text,
            &bail_type,
            &bail_amount,
            &surety_name,
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

        if name.read().trim().is_empty() {
            toast.error("Name is required.".to_string(), ToastOptions::new());
            return;
        }

        let aliases: Vec<String> = aliases_text
            .read()
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();

        let bail_amt: Option<f64> = bail_amount
            .read()
            .trim()
            .parse::<f64>()
            .ok();

        match mode {
            FormMode::Create => {
                if case_id.read().is_empty() {
                    toast.error("Case is required.".to_string(), ToastOptions::new());
                    return;
                }

                let body = serde_json::json!({
                    "case_id": case_id.read().clone(),
                    "name": name.read().trim().to_string(),
                    "aliases": aliases,
                    "usm_number": opt_str(&usm_number.read()),
                    "fbi_number": opt_str(&fbi_number.read()),
                    "date_of_birth": opt_date(&date_of_birth.read()),
                    "citizenship_status": citizenship_status.read().clone(),
                    "custody_status": custody_status.read().clone(),
                    "bail_type": opt_str(&bail_type.read()),
                    "bail_amount": bail_amt,
                    "surety_name": opt_str(&surety_name.read()),
                });

                spawn(async move {
                    in_flight.set(true);
                    match server::api::create_defendant(court, body.to_string()).await {
                        Ok(_) => {
                            on_saved.call(());
                            on_close.call(());
                            toast.success(
                                "Defendant created successfully".to_string(),
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
                    if name.read().trim() != init.name {
                        body.insert(
                            "name".into(),
                            serde_json::Value::String(name.read().trim().to_string()),
                        );
                    }
                    if aliases != init.aliases {
                        body.insert(
                            "aliases".into(),
                            serde_json::Value::Array(
                                aliases
                                    .iter()
                                    .map(|a| serde_json::Value::String(a.clone()))
                                    .collect(),
                            ),
                        );
                    }
                    let usm_val = opt_str(&usm_number.read());
                    if usm_number.read().trim() != init.usm_number.as_deref().unwrap_or("") {
                        body.insert("usm_number".into(), usm_val);
                    }
                    let fbi_val = opt_str(&fbi_number.read());
                    if fbi_number.read().trim() != init.fbi_number.as_deref().unwrap_or("") {
                        body.insert("fbi_number".into(), fbi_val);
                    }
                    if date_of_birth.read().trim() != init.date_of_birth.as_deref().unwrap_or("") {
                        body.insert("date_of_birth".into(), opt_date(&date_of_birth.read()));
                    }
                    if *citizenship_status.read() != init.citizenship_status {
                        body.insert(
                            "citizenship_status".into(),
                            serde_json::Value::String(citizenship_status.read().clone()),
                        );
                    }
                    if *custody_status.read() != init.custody_status {
                        body.insert(
                            "custody_status".into(),
                            serde_json::Value::String(custody_status.read().clone()),
                        );
                    }
                    if bail_amt != init.bail_amount {
                        body.insert(
                            "bail_amount".into(),
                            bail_amt
                                .map(|a| serde_json::json!(a))
                                .unwrap_or(serde_json::Value::Null),
                        );
                    }
                    let bt_val = opt_str(&bail_type.read());
                    if bail_type.read().trim() != init.bail_type.as_deref().unwrap_or("") {
                        body.insert("bail_type".into(), bt_val);
                    }
                    let sn_val = opt_str(&surety_name.read());
                    if surety_name.read().trim() != init.surety_name.as_deref().unwrap_or("") {
                        body.insert("surety_name".into(), sn_val);
                    }
                }

                if body.is_empty() {
                    on_close.call(());
                    return;
                }

                let payload = serde_json::Value::Object(body).to_string();
                spawn(async move {
                    in_flight.set(true);
                    match server::api::update_defendant(court, id, payload).await {
                        Ok(_) => {
                            on_saved.call(());
                            on_close.call(());
                            toast.success(
                                "Defendant updated successfully".to_string(),
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
        FormMode::Create => "New Defendant",
        FormMode::Edit => "Edit Defendant",
    };
    let description = match mode {
        FormMode::Create => "Add a defendant to an existing case.",
        FormMode::Edit => "Modify defendant information.",
    };
    let submit_label = match mode {
        FormMode::Create => "Create Defendant",
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
                            label: "Full Name *",
                            value: name.read().clone(),
                            on_input: move |e: FormEvent| name.set(e.value()),
                            placeholder: "e.g., John Doe",
                        }

                        Input {
                            label: "Aliases (comma-separated)",
                            value: aliases_text.read().clone(),
                            on_input: move |e: FormEvent| aliases_text.set(e.value()),
                            placeholder: "e.g., Johnny D, JD",
                        }

                        Separator {}

                        Input {
                            label: "USM Number",
                            value: usm_number.read().clone(),
                            on_input: move |e: FormEvent| usm_number.set(e.value()),
                            placeholder: "e.g., 12345-001",
                        }

                        Input {
                            label: "FBI Number",
                            value: fbi_number.read().clone(),
                            on_input: move |e: FormEvent| fbi_number.set(e.value()),
                        }

                        Input {
                            label: "Date of Birth",
                            input_type: "date",
                            value: date_of_birth.read().clone(),
                            on_input: move |e: FormEvent| date_of_birth.set(e.value()),
                        }

                        FormSelect {
                            label: "Citizenship Status",
                            value: "{citizenship_status}",
                            onchange: move |e: Event<FormData>| citizenship_status.set(e.value()),
                            for cs in CITIZENSHIP_STATUSES.iter() {
                                option { value: *cs, "{cs}" }
                            }
                        }

                        FormSelect {
                            label: "Custody Status",
                            value: "{custody_status}",
                            onchange: move |e: Event<FormData>| custody_status.set(e.value()),
                            for cs in CUSTODY_STATUSES.iter() {
                                option { value: *cs, "{cs}" }
                            }
                        }

                        Separator {}

                        FormSelect {
                            label: "Bail Type",
                            value: "{bail_type}",
                            onchange: move |e: Event<FormData>| bail_type.set(e.value()),
                            option { value: "", "-- None --" }
                            for bt in BAIL_TYPES.iter() {
                                option { value: *bt, "{bt}" }
                            }
                        }

                        Input {
                            label: "Bail Amount",
                            input_type: "number",
                            value: bail_amount.read().clone(),
                            on_input: move |e: FormEvent| bail_amount.set(e.value()),
                            placeholder: "e.g., 50000",
                        }

                        Input {
                            label: "Surety Name",
                            value: surety_name.read().clone(),
                            on_input: move |e: FormEvent| surety_name.set(e.value()),
                            placeholder: "e.g., ABC Bail Bonds",
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
    name: &Signal<String>,
    usm_number: &Signal<String>,
    fbi_number: &Signal<String>,
    date_of_birth: &Signal<String>,
    citizenship_status: &Signal<String>,
    custody_status: &Signal<String>,
    aliases_text: &Signal<String>,
    bail_type: &Signal<String>,
    bail_amount: &Signal<String>,
    surety_name: &Signal<String>,
) -> String {
    serde_json::json!({
        "name": name.read().clone(),
        "usm_number": usm_number.read().clone(),
        "fbi_number": fbi_number.read().clone(),
        "date_of_birth": date_of_birth.read().clone(),
        "citizenship_status": citizenship_status.read().clone(),
        "custody_status": custody_status.read().clone(),
        "aliases_text": aliases_text.read().clone(),
        "bail_type": bail_type.read().clone(),
        "bail_amount": bail_amount.read().clone(),
        "surety_name": surety_name.read().clone(),
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
