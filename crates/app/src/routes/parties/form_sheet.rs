use dioxus::prelude::*;
use shared_types::{
    CaseSearchResponse, PartyResponse, VALID_ENTITY_TYPES, VALID_PARTY_ROLES, VALID_PARTY_STATUSES,
    VALID_PARTY_TYPES,
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

/// Unified create/edit form for parties, rendered inside a Sheet.
#[component]
pub fn PartyFormSheet(
    mode: FormMode,
    initial: Option<PartyResponse>,
    open: bool,
    on_close: EventHandler<()>,
    on_saved: EventHandler<()>,
) -> Element {
    let ctx = use_context::<CourtContext>();
    let toast = use_toast();

    // --- Form field signals ---
    let mut name = use_signal(String::new);
    let mut first_name = use_signal(String::new);
    let mut last_name = use_signal(String::new);
    let mut middle_name = use_signal(String::new);
    let mut party_type = use_signal(|| "Defendant".to_string());
    let mut party_role = use_signal(|| "Lead".to_string());
    let mut entity_type = use_signal(|| "Individual".to_string());
    let mut organization_name = use_signal(String::new);
    let mut email = use_signal(String::new);
    let mut phone = use_signal(String::new);
    let mut status = use_signal(|| "Active".to_string());
    let mut case_id = use_signal(String::new);
    let mut pro_se = use_signal(|| false);

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
                first_name.set(data.first_name.clone().unwrap_or_default());
                last_name.set(data.last_name.clone().unwrap_or_default());
                middle_name.set(data.middle_name.clone().unwrap_or_default());
                party_type.set(data.party_type.clone());
                party_role.set(data.party_role.clone());
                entity_type.set(data.entity_type.clone());
                organization_name.set(data.organization_name.clone().unwrap_or_default());
                email.set(data.email.clone().unwrap_or_default());
                phone.set(data.phone.clone().unwrap_or_default());
                status.set(data.status.clone());
                case_id.set(data.case_id.clone());
                pro_se.set(data.pro_se);
            }
        } else if mode == FormMode::Create && hydrated_id.read().is_empty() {
            // Already at defaults
        } else if mode == FormMode::Create {
            hydrated_id.set(String::new());
            name.set(String::new());
            first_name.set(String::new());
            last_name.set(String::new());
            middle_name.set(String::new());
            party_type.set("Defendant".to_string());
            party_role.set("Lead".to_string());
            entity_type.set("Individual".to_string());
            organization_name.set(String::new());
            email.set(String::new());
            phone.set(String::new());
            status.set("Active".to_string());
            case_id.set(String::new());
            pro_se.set(false);
        }
    });

    // --- Dirty state ---
    let mut initial_snapshot = use_signal(String::new);

    use_effect(move || {
        if open {
            initial_snapshot.set(snapshot(
                &name,
                &first_name,
                &last_name,
                &party_type,
                &party_role,
                &entity_type,
                &email,
                &phone,
                &status,
                &pro_se,
            ));
        }
    });

    let is_dirty = move || {
        let current = snapshot(
            &name,
            &first_name,
            &last_name,
            &party_type,
            &party_role,
            &entity_type,
            &email,
            &phone,
            &status,
            &pro_se,
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

        match mode {
            FormMode::Create => {
                if case_id.read().is_empty() {
                    toast.error("Case is required.".to_string(), ToastOptions::new());
                    return;
                }

                let body = serde_json::json!({
                    "case_id": case_id.read().clone(),
                    "name": name.read().trim().to_string(),
                    "party_type": party_type.read().clone(),
                    "entity_type": entity_type.read().clone(),
                    "party_role": opt_str(&party_role.read()),
                    "first_name": opt_str(&first_name.read()),
                    "last_name": opt_str(&last_name.read()),
                    "middle_name": opt_str(&middle_name.read()),
                    "organization_name": opt_str(&organization_name.read()),
                    "email": opt_str(&email.read()),
                    "phone": opt_str(&phone.read()),
                    "pro_se": *pro_se.read(),
                });

                spawn(async move {
                    in_flight.set(true);
                    match server::api::create_party(court, body.to_string()).await {
                        Ok(_) => {
                            on_saved.call(());
                            on_close.call(());
                            toast.success(
                                "Party created successfully".to_string(),
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
                let body = serde_json::json!({
                    "name": name.read().trim().to_string(),
                    "party_type": party_type.read().clone(),
                    "party_role": opt_str(&party_role.read()),
                    "entity_type": entity_type.read().clone(),
                    "first_name": opt_str(&first_name.read()),
                    "last_name": opt_str(&last_name.read()),
                    "middle_name": opt_str(&middle_name.read()),
                    "organization_name": opt_str(&organization_name.read()),
                    "email": opt_str(&email.read()),
                    "phone": opt_str(&phone.read()),
                    "status": status.read().clone(),
                    "pro_se": *pro_se.read(),
                });

                spawn(async move {
                    in_flight.set(true);
                    match server::api::update_party(court, id, body.to_string()).await {
                        Ok(_) => {
                            on_saved.call(());
                            on_close.call(());
                            toast.success(
                                "Party updated successfully".to_string(),
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
        FormMode::Create => "New Party",
        FormMode::Edit => "Edit Party",
    };
    let description = match mode {
        FormMode::Create => "Add a party to an existing case.",
        FormMode::Edit => "Modify party information.",
    };
    let submit_label = match mode {
        FormMode::Create => "Create Party",
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
                            label: "First Name",
                            value: first_name.read().clone(),
                            on_input: move |e: FormEvent| first_name.set(e.value()),
                        }

                        Input {
                            label: "Last Name",
                            value: last_name.read().clone(),
                            on_input: move |e: FormEvent| last_name.set(e.value()),
                        }

                        Input {
                            label: "Middle Name",
                            value: middle_name.read().clone(),
                            on_input: move |e: FormEvent| middle_name.set(e.value()),
                        }

                        FormSelect {
                            label: "Party Type",
                            value: "{party_type}",
                            onchange: move |e: Event<FormData>| party_type.set(e.value()),
                            for pt in VALID_PARTY_TYPES.iter() {
                                option { value: *pt, "{pt}" }
                            }
                        }

                        FormSelect {
                            label: "Party Role",
                            value: "{party_role}",
                            onchange: move |e: Event<FormData>| party_role.set(e.value()),
                            for pr in VALID_PARTY_ROLES.iter() {
                                option { value: *pr, "{pr}" }
                            }
                        }

                        FormSelect {
                            label: "Entity Type",
                            value: "{entity_type}",
                            onchange: move |e: Event<FormData>| entity_type.set(e.value()),
                            for et in VALID_ENTITY_TYPES.iter() {
                                option { value: *et, "{et}" }
                            }
                        }

                        Input {
                            label: "Organization Name",
                            value: organization_name.read().clone(),
                            on_input: move |e: FormEvent| organization_name.set(e.value()),
                            placeholder: "For entity types like Corporation, LLC",
                        }

                        Separator {}

                        Input {
                            label: "Email",
                            input_type: "email",
                            value: email.read().clone(),
                            on_input: move |e: FormEvent| email.set(e.value()),
                        }

                        Input {
                            label: "Phone",
                            input_type: "tel",
                            value: phone.read().clone(),
                            on_input: move |e: FormEvent| phone.set(e.value()),
                        }

                        if mode == FormMode::Edit {
                            FormSelect {
                                label: "Status",
                                value: "{status}",
                                onchange: move |e: Event<FormData>| status.set(e.value()),
                                for s in VALID_PARTY_STATUSES.iter() {
                                    option { value: *s, "{s}" }
                                }
                            }
                        }

                        div { class: "checkbox-row",
                            style: "display: flex; align-items: center; gap: var(--space-sm);",
                            input {
                                r#type: "checkbox",
                                checked: *pro_se.read(),
                                onchange: move |e: FormEvent| pro_se.set(e.value() == "true"),
                            }
                            label { "Pro Se (self-represented)" }
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
    first_name: &Signal<String>,
    last_name: &Signal<String>,
    party_type: &Signal<String>,
    party_role: &Signal<String>,
    entity_type: &Signal<String>,
    email: &Signal<String>,
    phone: &Signal<String>,
    status: &Signal<String>,
    pro_se: &Signal<bool>,
) -> String {
    serde_json::json!({
        "name": name.read().clone(),
        "first_name": first_name.read().clone(),
        "last_name": last_name.read().clone(),
        "party_type": party_type.read().clone(),
        "party_role": party_role.read().clone(),
        "entity_type": entity_type.read().clone(),
        "email": email.read().clone(),
        "phone": phone.read().clone(),
        "status": status.read().clone(),
        "pro_se": *pro_se.read(),
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
