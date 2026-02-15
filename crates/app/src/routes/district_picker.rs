use crate::auth::use_auth;
use crate::{CourtContext, COURT_OPTIONS};
use dioxus::prelude::*;
use shared_ui::components::{
    Button, ButtonVariant, DialogContent, DialogDescription, DialogRoot, DialogTitle, FormSelect,
};
use shared_ui::{
    use_toast, DropdownMenu, DropdownMenuContent, DropdownMenuItem, DropdownMenuSeparator,
    DropdownMenuTrigger, ToastOptions,
};

/// District picker dropdown — shows in the sidebar header.
///
/// Reads `AuthUser.court_roles` to determine accessible courts:
/// - Admin users see all `COURT_OPTIONS`
/// - Regular users see only courts present in their `court_roles`
///
/// Selecting a court sets `CourtContext.court_id`, causing all pages to
/// reactively refresh data for the newly selected district.
#[component]
pub fn DistrictPicker() -> Element {
    let auth = use_auth();
    let mut ctx = use_context::<CourtContext>();
    let mut show_request_dialog = use_signal(|| false);

    // Build list of courts the user can access
    let accessible_courts: Vec<(&str, &str)> = {
        let user = auth.current_user.read();
        match user.as_ref() {
            Some(u) if u.role == "admin" => COURT_OPTIONS.to_vec(),
            Some(u) => COURT_OPTIONS
                .iter()
                .filter(|(id, _)| u.court_roles.contains_key(*id))
                .copied()
                .collect(),
            None => vec![],
        }
    };

    let current_court_id = ctx.court_id.read().clone();

    // Find the display name for the current court
    let current_name = COURT_OPTIONS
        .iter()
        .find(|(id, _)| *id == current_court_id.as_str())
        .map(|(_, name)| *name)
        .unwrap_or(&current_court_id);

    // Abbreviation for the icon badge
    let abbrev: String = current_court_id.chars().take(2).collect::<String>().to_uppercase();

    rsx! {
        document::Link { rel: "stylesheet", href: asset!("./district_picker.css") }

        div { class: "district-picker",
            DropdownMenu {
                DropdownMenuTrigger {
                    div { class: "district-picker-trigger",
                        span { class: "district-picker-icon", "{abbrev}" }
                        span { class: "district-picker-label", "{current_name}" }
                        span { class: "district-picker-chevron", "\u{25BC}" }
                    }
                }
                DropdownMenuContent {
                    for (idx , (court_id , court_name)) in accessible_courts.iter().enumerate() {
                        {
                            let cid = court_id.to_string();
                            let is_current = cid == current_court_id;
                            rsx! {
                                DropdownMenuItem::<String> {
                                    value: cid.clone(),
                                    index: idx,
                                    on_select: move |val: String| {
                                        ctx.court_id.set(val);
                                    },
                                    div { class: "district-item",
                                        span { class: "district-item-check",
                                            if is_current { "\u{2713}" } else { "" }
                                        }
                                        span { class: "district-item-name", "{court_name}" }
                                    }
                                }
                            }
                        }
                    }

                    DropdownMenuSeparator {}

                    DropdownMenuItem::<String> {
                        value: "__request_access__".to_string(),
                        index: accessible_courts.len(),
                        on_select: move |_: String| {
                            show_request_dialog.set(true);
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
/// Filters `COURT_OPTIONS` to exclude courts the user already has access to.
/// On submit, calls `request_court_admission`. If auto-approved (uscourts.gov
/// email match), refreshes auth state so the new court appears immediately.
#[component]
fn RequestAccessDialog(open: bool, on_close: EventHandler<()>) -> Element {
    let mut auth = use_auth();
    let mut ctx = use_context::<CourtContext>();
    let toast = use_toast();

    let mut selected_court = use_signal(String::new);
    let mut selected_role = use_signal(|| "attorney".to_string());
    let mut submitting = use_signal(|| false);
    let mut error_msg: Signal<Option<String>> = use_signal(|| None);

    // Courts the user does NOT yet have access to
    let available_courts: Vec<(&str, &str)> = {
        let user = auth.current_user.read();
        match user.as_ref() {
            Some(u) => COURT_OPTIONS
                .iter()
                .filter(|(id, _)| !u.court_roles.contains_key(*id))
                .copied()
                .collect(),
            None => vec![],
        }
    };

    let handle_submit = move |_| {
        let court = selected_court.read().clone();
        let role = selected_role.read().clone();

        if court.is_empty() {
            error_msg.set(Some("Please select a district.".to_string()));
            return;
        }

        spawn(async move {
            submitting.set(true);
            error_msg.set(None);

            match server::api::request_court_admission(court.clone(), role).await {
                Ok(result) if result == "approved" => {
                    // Auto-approved — refresh auth to pick up new court_roles
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
                    // Pending — admin review required
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
                            option { value: *court_id, "{court_name}" }
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
