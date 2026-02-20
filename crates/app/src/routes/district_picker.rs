use crate::auth::use_auth;
use crate::CourtContext;
use dioxus::prelude::*;
use shared_ui::components::{
    Button, ButtonVariant, DialogContent, DialogDescription, DialogRoot, DialogTitle, FormSelect,
};
use shared_ui::{use_toast, ToastOptions};

/// Fetch all courts from the API, returning (id, name) pairs sorted by name.
fn use_all_courts() -> Resource<Vec<(String, String)>> {
    use_resource(|| async move {
        match server::api::list_courts().await {
            Ok(courts) => courts.into_iter().map(|c| (c.id, c.name)).collect(),
            Err(_) => vec![],
        }
    })
}

/// District picker dropdown — shows in the sidebar header.
///
/// Fetches all courts from the database. Admin users see all courts;
/// regular users see only courts present in their `court_roles`.
///
/// Selecting a court sets `CourtContext.court_id`, causing all pages to
/// reactively refresh data for the newly selected district.
#[component]
pub fn DistrictPicker() -> Element {
    let auth = use_auth();
    let mut ctx = use_context::<CourtContext>();
    let mut show_request_dialog = use_signal(|| false);
    let mut open = use_signal(|| false);
    let mut search_filter = use_signal(String::new);
    let all_courts = use_all_courts();

    let current_court_id = ctx.court_id.read().clone();

    // Build list of courts the user can access, filtered by search
    let accessible_courts: Vec<(String, String)> = {
        let courts = match &*all_courts.read() {
            Some(c) => c.clone(),
            None => vec![],
        };
        let user = auth.current_user.read();
        let role_filtered: Vec<(String, String)> = match user.as_ref() {
            Some(u) if u.role == "admin" => courts,
            Some(u) => courts
                .into_iter()
                .filter(|(id, _)| u.court_roles.contains_key(id.as_str()))
                .collect(),
            None => vec![],
        };
        let q = search_filter.read().to_lowercase();
        if q.is_empty() {
            role_filtered
        } else {
            role_filtered
                .into_iter()
                .filter(|(id, name)| {
                    name.to_lowercase().contains(&q) || id.to_lowercase().contains(&q)
                })
                .collect()
        }
    };

    // Find the display name for the current court from all loaded courts
    let current_name = match &*all_courts.read() {
        Some(courts) => courts
            .iter()
            .find(|(id, _)| *id == current_court_id)
            .map(|(_, name)| name.clone())
            .unwrap_or(current_court_id.clone()),
        None => current_court_id.clone(),
    };

    // Abbreviation for the icon badge
    let abbrev: String = current_court_id
        .chars()
        .take(2)
        .collect::<String>()
        .to_uppercase();

    let is_open = *open.read();

    rsx! {
        document::Link { rel: "stylesheet", href: asset!("./district_picker.css") }

        div { class: "district-picker",
            // Trigger button
            div {
                class: "district-picker-trigger",
                onclick: move |_| open.set(!is_open),
                span { class: "district-picker-icon", "{abbrev}" }
                span { class: "district-picker-label", "{current_name}" }
                span { class: "district-picker-chevron",
                    if is_open { "\u{25B2}" } else { "\u{25BC}" }
                }
            }

            // Popover content
            if is_open {
                // Backdrop to close on outside click
                div {
                    class: "district-backdrop",
                    onclick: move |_| {
                        open.set(false);
                        search_filter.set(String::new());
                    },
                }

                div { class: "district-popover",
                    // Search input
                    div { class: "district-search-wrap",
                        input {
                            class: "district-search-input",
                            r#type: "text",
                            placeholder: "Search districts...",
                            value: search_filter.read().clone(),
                            oninput: move |evt: FormEvent| search_filter.set(evt.value()),
                        }
                    }

                    // Scrollable court list
                    div { class: "district-list",
                        for (court_id , court_name) in accessible_courts.iter() {
                            {
                                let cid = court_id.clone();
                                let is_current = *court_id == current_court_id;
                                rsx! {
                                    div {
                                        class: if is_current { "district-item district-item-active" } else { "district-item" },
                                        onclick: move |_| {
                                            ctx.court_id.set(cid.clone());
                                            open.set(false);
                                            search_filter.set(String::new());
                                        },
                                        span { class: "district-item-check",
                                            if is_current { "\u{2713}" } else { "" }
                                        }
                                        span { class: "district-item-name", "{court_name}" }
                                    }
                                }
                            }
                        }

                        if accessible_courts.is_empty() && !search_filter.read().is_empty() {
                            div { class: "district-no-results", "No matching districts" }
                        }
                    }

                    // Separator + Request Access
                    div { class: "district-separator" }
                    div {
                        class: "district-item",
                        onclick: move |_| {
                            show_request_dialog.set(true);
                            open.set(false);
                            search_filter.set(String::new());
                        },
                        span { class: "district-request-link", "Request Access..." }
                    }
                }
            }
        }

        RequestAccessDialog {
            open: show_request_dialog(),
            on_close: move |_: ()| show_request_dialog.set(false),
        }
    }
}

/// Dialog for requesting access to a new district court.
///
/// Fetches all courts from the database and filters to exclude courts
/// the user already has access to.
/// On submit, calls `request_court_admission`. If auto-approved (uscourts.gov
/// email match), refreshes auth state so the new court appears immediately.
#[component]
fn RequestAccessDialog(open: bool, on_close: EventHandler<()>) -> Element {
    let mut auth = use_auth();
    let mut ctx = use_context::<CourtContext>();
    let toast = use_toast();
    let all_courts = use_all_courts();

    let mut selected_court = use_signal(String::new);
    let mut selected_role = use_signal(|| "attorney".to_string());
    let mut reason = use_signal(String::new);
    let mut submitting = use_signal(|| false);
    let mut error_msg: Signal<Option<String>> = use_signal(|| None);

    // Courts the user does NOT yet have access to
    let available_courts: Vec<(String, String)> = {
        let courts = match &*all_courts.read() {
            Some(c) => c.clone(),
            None => vec![],
        };
        let user = auth.current_user.read();
        match user.as_ref() {
            Some(u) => courts
                .into_iter()
                .filter(|(id, _)| !u.court_roles.contains_key(id.as_str()))
                .collect(),
            None => vec![],
        }
    };

    let handle_submit = move |_| {
        let court = selected_court.read().clone();
        let role = selected_role.read().clone();
        let reason_text = reason.read().clone();
        let notes = if reason_text.trim().is_empty() {
            None
        } else {
            Some(reason_text)
        };

        if court.is_empty() {
            error_msg.set(Some("Please select a district.".to_string()));
            return;
        }

        spawn(async move {
            submitting.set(true);
            error_msg.set(None);

            match server::api::request_court_admission(court.clone(), role, notes).await {
                Ok(result) if result == "approved" => {
                    match server::api::get_current_user().await {
                        Ok(Some(user)) => auth.set_user(user),
                        _ => {}
                    }
                    ctx.court_id.set(court);
                    toast.success(
                        "Access granted! Switched to your new district.".to_string(),
                        ToastOptions::new(),
                    );
                    on_close.call(());
                }
                Ok(_) => {
                    toast.info(
                        "Request submitted. Awaiting admin review.".to_string(),
                        ToastOptions::new(),
                    );
                    on_close.call(());
                }
                Err(e) => {
                    error_msg.set(Some(
                        shared_types::AppError::friendly_message(&e.to_string()),
                    ));
                }
            }

            submitting.set(false);
        });
    };

    rsx! {
        DialogRoot {
            open: open,
            on_open_change: move |is_open: bool| {
                if !is_open {
                    on_close.call(());
                }
            },
            DialogContent {
                DialogTitle { "Request Court Access" }
                DialogDescription {
                    "Select a district court and role to request access. "
                    "If your email matches the court's .uscourts.gov domain, access is granted immediately."
                }

                div { class: "request-access-form",
                    if let Some(err) = error_msg() {
                        p { class: "form-error", "{err}" }
                    }

                    FormSelect {
                        label: "District Court".to_string(),
                        value: selected_court(),
                        onchange: move |evt: Event<FormData>| {
                            selected_court.set(evt.value());
                        },
                        option { value: "", disabled: true, "Select a district..." }
                        for (court_id , court_name) in available_courts.iter() {
                            option { value: court_id.as_str(), "{court_name}" }
                        }
                    }

                    FormSelect {
                        label: "Role".to_string(),
                        value: selected_role(),
                        onchange: move |evt: Event<FormData>| {
                            selected_role.set(evt.value());
                        },
                        option { value: "attorney", "Attorney" }
                        option { value: "clerk", "Clerk" }
                        option { value: "judge", "Judge" }
                    }

                    div { class: "dialog-field",
                        label { class: "form-label", "Reason for access (optional)" }
                        textarea {
                            class: "input",
                            rows: 3,
                            placeholder: "Briefly describe why you need access to this court...",
                            value: reason(),
                            oninput: move |evt: FormEvent| reason.set(evt.value()),
                        }
                    }

                    div { class: "request-access-actions",
                        Button {
                            variant: ButtonVariant::Ghost,
                            onclick: move |_| on_close.call(()),
                            "Cancel"
                        }
                        Button {
                            variant: ButtonVariant::Primary,
                            disabled: submitting(),
                            onclick: handle_submit,
                            if submitting() { "Submitting..." } else { "Request Access" }
                        }
                    }
                }
            }
        }
    }
}
