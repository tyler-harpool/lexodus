use crate::auth::use_can_manage_memberships;
use crate::{CourtContext, COURT_OPTIONS};
use dioxus::prelude::*;
use dioxus_free_icons::icons::ld_icons::LdEllipsis;
use dioxus_free_icons::Icon;
use server::api::{
    approve_court_request, create_user, delete_user, deny_court_request,
    list_pending_court_requests, list_users_with_memberships, remove_user_court_role,
    set_user_court_role, update_user,
};
use shared_types::UserWithMembership;
use shared_ui::{
    use_toast, AlertDialogAction, AlertDialogActions, AlertDialogCancel, AlertDialogContent,
    AlertDialogDescription, AlertDialogRoot, AlertDialogTitle, Avatar, AvatarFallback, Badge,
    BadgeVariant, Button, ButtonVariant, Checkbox, CheckboxIndicator, CheckboxState, ContextMenu,
    ContextMenuContent, ContextMenuTrigger, DialogContent, DialogDescription, DialogRoot,
    DialogTitle, DropdownMenu, DropdownMenuContent, DropdownMenuItem, DropdownMenuSeparator,
    DropdownMenuTrigger, Input, Label, SelectContent, SelectItem, SelectItemIndicator, SelectRoot,
    SelectTrigger, SelectValue, Separator, ToastOptions, Toolbar, ToolbarButton, ToolbarSeparator,
};
use shared_ui::components::FormSelect;

/// Extract the first two characters of a name as uppercase initials.
fn initials(name: &str) -> String {
    name.chars().take(2).collect::<String>().to_uppercase()
}

/// Map a court role string to its badge variant.
fn court_role_badge_variant(role: &str) -> BadgeVariant {
    match role.to_lowercase().as_str() {
        "clerk" => BadgeVariant::Primary,
        "judge" => BadgeVariant::Destructive,
        "attorney" => BadgeVariant::Secondary,
        _ => BadgeVariant::Outline,
    }
}

/// Format a court role string for display (capitalized).
fn court_role_display(role: &str) -> &str {
    match role.to_lowercase().as_str() {
        "attorney" => "Attorney",
        "clerk" => "Clerk",
        "judge" => "Judge",
        _ => "",
    }
}

/// Users management page with CRUD operations and court role management.
#[component]
pub fn Users() -> Element {
    let ctx = use_context::<CourtContext>();
    let toast = use_toast();
    let can_manage = use_can_manage_memberships();

    // Fetch users with their membership for the current court (re-fetches on court change)
    let mut users = use_resource(move || {
        let court = ctx.court_id.read().clone();
        async move { list_users_with_memberships(court).await }
    });

    // Toggle between "Court Members", "All Users", and "Pending Requests"
    let mut view_mode = use_signal(|| "members".to_string()); // "members" | "all" | "requests"

    // Pending requests resource (only fetched when view_mode is "requests")
    let mut pending_requests = use_resource(move || {
        let court = ctx.court_id.read().clone();
        async move { list_pending_court_requests(court).await }
    });

    let mut show_create_dialog = use_signal(|| false);
    let mut editing_user: Signal<Option<UserWithMembership>> = use_signal(|| None);
    let mut show_delete_confirm = use_signal(|| false);
    let mut selected_ids: Signal<Vec<i64>> = use_signal(Vec::new);
    let mut form_username = use_signal(String::new);
    let mut form_display_name = use_signal(String::new);

    // "Assign to Court" dialog state
    let mut show_assign_dialog = use_signal(|| false);
    let mut assign_user_id: Signal<Option<i64>> = use_signal(|| None);
    let mut assign_user_name = use_signal(String::new);
    let mut assign_court = use_signal(String::new);
    let mut assign_role = use_signal(|| "attorney".to_string());

    // Deny request dialog state
    let mut show_deny_dialog = use_signal(|| false);
    let mut deny_request_id = use_signal(String::new);
    let mut deny_reason = use_signal(String::new);

    let has_selection = !selected_ids.read().is_empty();

    // Handle form save (create or update)
    let handle_save = move |_: MouseEvent| {
        let username = form_username.read().clone();
        let display_name = form_display_name.read().clone();
        let editing = editing_user.read().clone();

        spawn(async move {
            let result = if let Some(user) = editing {
                update_user(user.id, username, display_name).await
            } else {
                create_user(username, display_name).await
            };

            match result {
                Ok(_) => {
                    let msg = if editing_user.read().is_some() {
                        "User updated"
                    } else {
                        "User created"
                    };
                    toast.success(msg.to_string(), ToastOptions::new());
                    show_create_dialog.set(false);
                    editing_user.set(None);
                    users.restart();
                }
                Err(err) => {
                    toast.error(
                        shared_types::AppError::friendly_message(&err.to_string()),
                        ToastOptions::new(),
                    );
                }
            }
        });
    };

    // Handle delete of selected users
    let handle_delete_selected = move |_: MouseEvent| {
        let ids = selected_ids.read().clone();

        spawn(async move {
            let mut had_error = false;
            for id in &ids {
                if let Err(err) = delete_user(*id).await {
                    toast.error(
                        format!(
                            "Failed to delete user {id}: {}",
                            shared_types::AppError::friendly_message(&err.to_string())
                        ),
                        ToastOptions::new(),
                    );
                    had_error = true;
                }
            }
            if !had_error {
                let count = ids.len();
                toast.success(format!("{count} user(s) deleted"), ToastOptions::new());
            }
            selected_ids.set(Vec::new());
            show_delete_confirm.set(false);
            users.restart();
        });
    };

    // Filter based on view mode: "members" shows only users with a role in the current court
    let mode = view_mode.read().clone();
    let filtered_users: Option<Vec<UserWithMembership>> = {
        let data = users.read();
        data.as_ref().and_then(|r| r.as_ref().ok()).map(|all| {
            if mode == "members" {
                all.iter().filter(|u| !u.court_role.is_empty()).cloned().collect()
            } else {
                all.to_vec()
            }
        })
    };

    rsx! {
        document::Link { rel: "stylesheet", href: asset!("./users.css") }

        div {
            class: "users-page",

            // Toolbar
            Toolbar {
                aria_label: "User actions",
                ToolbarButton {
                    index: 0usize,
                    on_click: move |_| {
                        editing_user.set(None);
                        form_username.set(String::new());
                        form_display_name.set(String::new());
                        show_create_dialog.set(true);
                    },
                    "Add User"
                }
                ToolbarSeparator {}
                ToolbarButton {
                    index: 1usize,
                    disabled: !has_selection,
                    on_click: move |_| {
                        show_delete_confirm.set(true);
                    },
                    "Delete Selected"
                }
            }

            // View mode toggle
            div { class: "users-view-toggle",
                button {
                    class: if *view_mode.read() == "members" { "toggle-tab active" } else { "toggle-tab" },
                    onclick: move |_| view_mode.set("members".to_string()),
                    "Court Members"
                }
                button {
                    class: if *view_mode.read() == "all" { "toggle-tab active" } else { "toggle-tab" },
                    onclick: move |_| view_mode.set("all".to_string()),
                    "All Users"
                }
                if can_manage {
                    button {
                        class: if *view_mode.read() == "requests" { "toggle-tab active" } else { "toggle-tab" },
                        onclick: move |_| {
                            view_mode.set("requests".to_string());
                            pending_requests.restart();
                        },
                        "Pending Requests"
                    }
                }
            }

            // Pending Requests View
            if mode == "requests" && can_manage {
                div { class: "users-list",
                    {
                        let requests_data = pending_requests.read();
                        let requests_result = requests_data.as_ref();

                        match requests_result {
                            Some(Ok(requests)) if requests.is_empty() => {
                                rsx! {
                                    div { class: "users-empty",
                                        "No pending access requests for this court."
                                    }
                                }
                            }
                            Some(Ok(requests)) => {
                                rsx! {
                                    for req in requests.iter() {
                                        {
                                            let req_id = req.id.clone();
                                            let req_id_for_deny = req.id.clone();
                                            let display_name = req.user_display_name.clone().unwrap_or_else(|| format!("User #{}", req.user_id));
                                            let email = req.user_email.clone().unwrap_or_default();
                                            let role = req.requested_role.clone();
                                            let notes = req.notes.clone();
                                            let requested_at = req.requested_at.clone();

                                            rsx! {
                                                div { class: "request-card",
                                                    div { class: "request-info",
                                                        span { class: "request-name", "{display_name}" }
                                                        if !email.is_empty() {
                                                            span { class: "request-email", "{email}" }
                                                        }
                                                    }
                                                    div { class: "request-meta",
                                                        Badge {
                                                            variant: court_role_badge_variant(&role),
                                                            "{court_role_display(&role)}"
                                                        }
                                                        span { class: "request-date", "{requested_at}" }
                                                    }
                                                    if let Some(reason) = notes {
                                                        div { class: "request-reason",
                                                            span { class: "request-reason-label", "Reason:" }
                                                            " {reason}"
                                                        }
                                                    }
                                                    div { class: "request-actions",
                                                        Button {
                                                            variant: ButtonVariant::Primary,
                                                            onclick: move |_| {
                                                                let rid = req_id.clone();
                                                                spawn(async move {
                                                                    match approve_court_request(rid).await {
                                                                        Ok(()) => {
                                                                            toast.success("Request approved".to_string(), ToastOptions::new());
                                                                            pending_requests.restart();
                                                                            users.restart();
                                                                        }
                                                                        Err(err) => {
                                                                            toast.error(
                                                                                shared_types::AppError::friendly_message(&err.to_string()),
                                                                                ToastOptions::new(),
                                                                            );
                                                                        }
                                                                    }
                                                                });
                                                            },
                                                            "Approve"
                                                        }
                                                        Button {
                                                            variant: ButtonVariant::Destructive,
                                                            onclick: move |_| {
                                                                deny_request_id.set(req_id_for_deny.clone());
                                                                deny_reason.set(String::new());
                                                                show_deny_dialog.set(true);
                                                            },
                                                            "Deny"
                                                        }
                                                    }
                                                }
                                                Separator {}
                                            }
                                        }
                                    }
                                }
                            }
                            Some(Err(err)) => {
                                let msg = shared_types::AppError::friendly_message(&err.to_string());
                                rsx! {
                                    div { class: "users-empty", "Error loading requests: {msg}" }
                                }
                            }
                            None => {
                                rsx! {
                                    div { class: "users-empty", "Loading requests..." }
                                }
                            }
                        }
                    }
                }
            }

            // User List
            if mode != "requests" {
                div {
                    class: "users-list",

                    if let Some(user_vec) = filtered_users.as_ref() {
                        if user_vec.is_empty() {
                            div {
                                class: "users-empty",
                                if *view_mode.read() == "members" {
                                    "No members in this court. Switch to \"All Users\" to assign roles."
                                } else {
                                    "No users found. Click \"Add User\" to create one."
                                }
                            }
                        } else {
                            for user in user_vec.iter() {
                                {
                                    let user_id = user.id;
                                    let user_clone = user.clone();
                                    let user_for_edit = user.clone();
                                    let user_for_ctx_edit = user.clone();
                                    let user_for_assign = user.clone();
                                    let user_for_info = user.clone();
                                    let display_initials = initials(&user.display_name);
                                    let is_checked = selected_ids.read().contains(&user_id);

                                    rsx! {
                                        ContextMenu {
                                            ContextMenuTrigger {
                                                div {
                                                    class: "user-row",

                                                    Checkbox {
                                                        default_checked: if is_checked { CheckboxState::Checked } else { CheckboxState::Unchecked },
                                                        on_checked_change: move |state: CheckboxState| {
                                                            let mut ids = selected_ids.write();
                                                            match state {
                                                                CheckboxState::Checked => {
                                                                    if !ids.contains(&user_id) {
                                                                        ids.push(user_id);
                                                                    }
                                                                }
                                                                _ => {
                                                                    ids.retain(|&id| id != user_id);
                                                                }
                                                            }
                                                        },
                                                        CheckboxIndicator {
                                                            span { "\u{2713}" }
                                                        }
                                                    }

                                                    Avatar {
                                                        AvatarFallback { "{display_initials}" }
                                                    }

                                                    div {
                                                        class: "user-info",
                                                        span {
                                                            class: "user-display-name",
                                                            "{user_clone.display_name}"
                                                        }
                                                        span {
                                                            class: "user-username",
                                                            "@{user_clone.username}"
                                                        }
                                                    }

                                                    // Court Role column
                                                    {
                                                        let court_role = user_clone.court_role.clone();
                                                        let row_user_id = user_id;
                                                        let current_court = ctx.court_id.read().clone();
                                                        rsx! {
                                                            div {
                                                                class: "user-court-role",
                                                                if can_manage {
                                                                    {
                                                                        let current_role = court_role.clone();
                                                                        let court_for_change = current_court.clone();
                                                                        rsx! {
                                                                            SelectRoot::<String> {
                                                                                default_value: current_role.clone(),
                                                                                placeholder: "No Role",
                                                                                on_value_change: move |val: Option<String>| {
                                                                                    let new_role = val.unwrap_or_default();
                                                                                    let court = court_for_change.clone();
                                                                                    spawn(async move {
                                                                                        let result = if new_role.is_empty() {
                                                                                            remove_user_court_role(row_user_id, court).await
                                                                                        } else {
                                                                                            set_user_court_role(row_user_id, court, new_role.clone()).await
                                                                                        };
                                                                                        match result {
                                                                                            Ok(_) => {
                                                                                                let label = if new_role.is_empty() {
                                                                                                    "No Role"
                                                                                                } else {
                                                                                                    court_role_display(&new_role)
                                                                                                };
                                                                                                toast.success(
                                                                                                    format!("Court role updated to {label}"),
                                                                                                    ToastOptions::new(),
                                                                                                );
                                                                                                users.restart();
                                                                                            }
                                                                                            Err(err) => {
                                                                                                toast.error(
                                                                                                    format!("Failed to update court role: {}", shared_types::AppError::friendly_message(&err.to_string())),
                                                                                                    ToastOptions::new(),
                                                                                                );
                                                                                            }
                                                                                        }
                                                                                    });
                                                                                },
                                                                                SelectTrigger {
                                                                                    aria_label: "Change court role",
                                                                                    SelectValue {}
                                                                                }
                                                                                SelectContent {
                                                                                    aria_label: "Court role options",
                                                                                    SelectItem::<String> {
                                                                                        value: "",
                                                                                        index: 0usize,
                                                                                        "No Role"
                                                                                        SelectItemIndicator { "\u{2713}" }
                                                                                    }
                                                                                    SelectItem::<String> {
                                                                                        value: "attorney",
                                                                                        index: 1usize,
                                                                                        "Attorney"
                                                                                        SelectItemIndicator { "\u{2713}" }
                                                                                    }
                                                                                    SelectItem::<String> {
                                                                                        value: "clerk",
                                                                                        index: 2usize,
                                                                                        "Clerk"
                                                                                        SelectItemIndicator { "\u{2713}" }
                                                                                    }
                                                                                    SelectItem::<String> {
                                                                                        value: "judge",
                                                                                        index: 3usize,
                                                                                        "Judge"
                                                                                        SelectItemIndicator { "\u{2713}" }
                                                                                    }
                                                                                }
                                                                            }
                                                                        }
                                                                    }
                                                                } else if !court_role.is_empty() {
                                                                    Badge {
                                                                        variant: court_role_badge_variant(&court_role),
                                                                        "{court_role_display(&court_role)}"
                                                                    }
                                                                }
                                                            }
                                                        }
                                                    }

                                                    // Actions dropdown ("..." button)
                                                    DropdownMenu {
                                                        DropdownMenuTrigger {
                                                            div { class: "user-actions-trigger",
                                                                Icon::<LdEllipsis> { icon: LdEllipsis, width: 18, height: 18 }
                                                            }
                                                        }
                                                        DropdownMenuContent {
                                                            // Edit
                                                            DropdownMenuItem::<String> {
                                                                value: "edit".to_string(),
                                                                index: 0usize,
                                                                on_select: move |_: String| {
                                                                    let u = user_for_ctx_edit.clone();
                                                                    form_username.set(u.username.clone());
                                                                    form_display_name.set(u.display_name.clone());
                                                                    editing_user.set(Some(u));
                                                                    show_create_dialog.set(true);
                                                                },
                                                                "\u{270E}  Edit"
                                                            }

                                                            // Assign to Court
                                                            if can_manage {
                                                                DropdownMenuItem::<String> {
                                                                    value: "assign_court".to_string(),
                                                                    index: 1usize,
                                                                    on_select: move |_: String| {
                                                                        let u = user_for_assign.clone();
                                                                        assign_user_id.set(Some(u.id));
                                                                        assign_user_name.set(u.display_name.clone());
                                                                        assign_court.set(String::new());
                                                                        assign_role.set("attorney".to_string());
                                                                        show_assign_dialog.set(true);
                                                                    },
                                                                    "\u{2795}  Assign to Court"
                                                                }
                                                            }

                                                            // Court memberships — inline remove
                                                            if !user_for_edit.all_court_roles.is_empty() {
                                                                DropdownMenuSeparator {}
                                                                for (idx , (cid , crole)) in user_for_edit.all_court_roles.iter().enumerate() {
                                                                    {
                                                                        let cid_clone = cid.clone();
                                                                        let court_label = COURT_OPTIONS.iter()
                                                                            .find(|(id, _)| *id == cid.as_str())
                                                                            .map(|(_, name)| *name)
                                                                            .unwrap_or(cid.as_str());
                                                                        let role_label = court_role_display(crole);
                                                                        let label = format!("\u{2715}  {court_label} ({role_label})");
                                                                        rsx! {
                                                                            DropdownMenuItem::<String> {
                                                                                value: format!("remove_{cid_clone}"),
                                                                                index: 10 + idx,
                                                                                on_select: move |_: String| {
                                                                                    let uid = user_id;
                                                                                    let court = cid_clone.clone();
                                                                                    spawn(async move {
                                                                                        match remove_user_court_role(uid, court.clone()).await {
                                                                                            Ok(_) => {
                                                                                                toast.success(
                                                                                                    format!("Removed from {court}"),
                                                                                                    ToastOptions::new(),
                                                                                                );
                                                                                                users.restart();
                                                                                            }
                                                                                            Err(err) => {
                                                                                                toast.error(
                                                                                                    shared_types::AppError::friendly_message(&err.to_string()),
                                                                                                    ToastOptions::new(),
                                                                                                );
                                                                                            }
                                                                                        }
                                                                                    });
                                                                                },
                                                                                "{label}"
                                                                            }
                                                                        }
                                                                    }
                                                                }
                                                            }

                                                            DropdownMenuSeparator {}

                                                            // Delete
                                                            DropdownMenuItem::<String> {
                                                                value: "delete".to_string(),
                                                                index: 99usize,
                                                                on_select: move |_: String| {
                                                                    spawn(async move {
                                                                        match delete_user(user_id).await {
                                                                            Ok(()) => {
                                                                                toast.success("User deleted".to_string(), ToastOptions::new());
                                                                                selected_ids.write().retain(|&id| id != user_id);
                                                                                users.restart();
                                                                            }
                                                                            Err(err) => {
                                                                                toast.error(shared_types::AppError::friendly_message(&err.to_string()), ToastOptions::new());
                                                                            }
                                                                        }
                                                                    });
                                                                },
                                                                span { class: "user-action-destructive", "\u{1F5D1}  Delete" }
                                                            }
                                                        }
                                                    }
                                                }
                                            } // ContextMenuTrigger close

                                            // Right-click info card
                                            ContextMenuContent {
                                                div { class: "user-info-card",
                                                    div { class: "user-info-card-header",
                                                        Avatar {
                                                            AvatarFallback { "{initials(&user_for_info.display_name)}" }
                                                        }
                                                        div {
                                                            span { class: "user-info-card-name", "{user_for_info.display_name}" }
                                                            span { class: "user-info-card-username", "@{user_for_info.username}" }
                                                        }
                                                    }

                                                    if !user_for_info.email.is_empty() {
                                                        div { class: "user-info-card-field",
                                                            span { class: "user-info-card-label", "Email" }
                                                            span { "{user_for_info.email}" }
                                                        }
                                                    }

                                                    if let Some(phone) = &user_for_info.phone_number {
                                                        div { class: "user-info-card-field",
                                                            span { class: "user-info-card-label", "Phone" }
                                                            span { "{phone}" }
                                                        }
                                                    }

                                                    div { class: "user-info-card-field",
                                                        span { class: "user-info-card-label", "Platform Role" }
                                                        Badge { variant: court_role_badge_variant(&user_for_info.role), "{user_for_info.role}" }
                                                    }

                                                    if !user_for_info.all_court_roles.is_empty() {
                                                        div { class: "user-info-card-courts",
                                                            span { class: "user-info-card-label", "Court Assignments" }
                                                            for (cid , crole) in user_for_info.all_court_roles.iter() {
                                                                {
                                                                    let court_label = COURT_OPTIONS.iter()
                                                                        .find(|(id, _)| *id == cid.as_str())
                                                                        .map(|(_, name)| *name)
                                                                        .unwrap_or(cid.as_str());
                                                                    rsx! {
                                                                        div { class: "user-info-card-court-row",
                                                                            span { "{court_label}" }
                                                                            Badge { variant: court_role_badge_variant(crole), "{court_role_display(crole)}" }
                                                                        }
                                                                    }
                                                                }
                                                            }
                                                        }
                                                    } else {
                                                        div { class: "user-info-card-field",
                                                            span { class: "user-info-card-label", "Court Assignments" }
                                                            span { class: "user-info-card-none", "None" }
                                                        }
                                                    }
                                                }
                                            }
                                        } // ContextMenu close

                                        Separator {}
                                    }
                                }
                            }
                        }
                    } else {
                        div {
                            class: "users-empty",
                            "Loading users..."
                        }
                    }
                }
            }

            // Deny Request Dialog
            AlertDialogRoot {
                open: show_deny_dialog(),
                on_open_change: move |open: bool| show_deny_dialog.set(open),
                AlertDialogContent {
                    AlertDialogTitle { "Deny Access Request" }
                    AlertDialogDescription {
                        "Optionally provide a reason for denying this request."
                    }
                    div { class: "dialog-field",
                        label { class: "form-label", "Reason (optional)" }
                        textarea {
                            class: "input",
                            rows: 3,
                            placeholder: "Reason for denial...",
                            value: deny_reason(),
                            oninput: move |evt: FormEvent| deny_reason.set(evt.value()),
                        }
                    }
                    AlertDialogActions {
                        AlertDialogCancel { "Cancel" }
                        AlertDialogAction {
                            on_click: move |_: MouseEvent| {
                                let rid = deny_request_id.read().clone();
                                let reason_text = deny_reason.read().clone();
                                let notes = if reason_text.trim().is_empty() {
                                    None
                                } else {
                                    Some(reason_text)
                                };
                                spawn(async move {
                                    match deny_court_request(rid, notes).await {
                                        Ok(()) => {
                                            toast.success("Request denied".to_string(), ToastOptions::new());
                                            pending_requests.restart();
                                        }
                                        Err(err) => {
                                            toast.error(
                                                shared_types::AppError::friendly_message(&err.to_string()),
                                                ToastOptions::new(),
                                            );
                                        }
                                    }
                                });
                            },
                            "Deny Request"
                        }
                    }
                }
            }

            // Create / Edit Dialog
            DialogRoot {
                open: show_create_dialog(),
                on_open_change: move |open: bool| show_create_dialog.set(open),
                DialogContent {
                    DialogTitle {
                        if editing_user.read().is_some() { "Edit User" } else { "Add User" }
                    }
                    DialogDescription {
                        if editing_user.read().is_some() {
                            "Update the user details below."
                        } else {
                            "Fill in the details to create a new user."
                        }
                    }

                    div {
                        class: "dialog-form",

                        div {
                            class: "dialog-field",
                            Label { html_for: "username-field", "Username" }
                            Input {
                                value: form_username(),
                                placeholder: "Enter username",
                                label: "",
                                on_input: move |evt: FormEvent| form_username.set(evt.value()),
                            }
                        }

                        div {
                            class: "dialog-field",
                            Label { html_for: "display-name-field", "Display Name" }
                            Input {
                                value: form_display_name(),
                                placeholder: "Enter display name",
                                label: "",
                                on_input: move |evt: FormEvent| form_display_name.set(evt.value()),
                            }
                        }

                        div {
                            class: "dialog-actions",
                            Button {
                                variant: ButtonVariant::Ghost,
                                onclick: move |_| {
                                    show_create_dialog.set(false);
                                    editing_user.set(None);
                                },
                                "Cancel"
                            }
                            Button {
                                variant: ButtonVariant::Primary,
                                onclick: handle_save,
                                "Save"
                            }
                        }
                    }
                }
            }

            // Delete Confirmation Alert Dialog
            AlertDialogRoot {
                open: show_delete_confirm(),
                on_open_change: move |open: bool| show_delete_confirm.set(open),
                AlertDialogContent {
                    AlertDialogTitle { "Confirm Deletion" }
                    AlertDialogDescription {
                        {
                            let count = selected_ids.read().len();
                            format!("Are you sure you want to delete {count} selected user(s)? This action cannot be undone.")
                        }
                    }
                    AlertDialogActions {
                        AlertDialogCancel { "Cancel" }
                        AlertDialogAction {
                            on_click: handle_delete_selected,
                            "Delete"
                        }
                    }
                }
            }

            // Assign to Court Dialog
            DialogRoot {
                open: show_assign_dialog(),
                on_open_change: move |open: bool| show_assign_dialog.set(open),
                DialogContent {
                    DialogTitle { "Assign to Court" }
                    DialogDescription {
                        {format!("Grant {} access to a court district.", assign_user_name.read())}
                    }

                    div {
                        class: "dialog-form",

                        FormSelect {
                            label: "District Court".to_string(),
                            value: assign_court(),
                            onchange: move |evt: Event<FormData>| {
                                assign_court.set(evt.value());
                            },
                            option { value: "", disabled: true, "Select a district..." }
                            for (court_id , court_name) in COURT_OPTIONS.iter() {
                                option { value: *court_id, "{court_name}" }
                            }
                        }

                        FormSelect {
                            label: "Role".to_string(),
                            value: assign_role(),
                            onchange: move |evt: Event<FormData>| {
                                assign_role.set(evt.value());
                            },
                            option { value: "attorney", "Attorney" }
                            option { value: "clerk", "Clerk" }
                            option { value: "judge", "Judge" }
                        }

                        div {
                            class: "dialog-actions",
                            Button {
                                variant: ButtonVariant::Ghost,
                                onclick: move |_| show_assign_dialog.set(false),
                                "Cancel"
                            }
                            Button {
                                variant: ButtonVariant::Primary,
                                onclick: move |_| {
                                    let uid = assign_user_id.read().unwrap_or(0);
                                    let court = assign_court.read().clone();
                                    let role = assign_role.read().clone();

                                    if court.is_empty() {
                                        toast.error("Please select a district.".to_string(), ToastOptions::new());
                                        return;
                                    }

                                    spawn(async move {
                                        match set_user_court_role(uid, court.clone(), role.clone()).await {
                                            Ok(_) => {
                                                toast.success(
                                                    format!("Assigned {} role in {}", court_role_display(&role), court),
                                                    ToastOptions::new(),
                                                );
                                                show_assign_dialog.set(false);
                                                users.restart();
                                            }
                                            Err(err) => {
                                                toast.error(
                                                    shared_types::AppError::friendly_message(&err.to_string()),
                                                    ToastOptions::new(),
                                                );
                                            }
                                        }
                                    });
                                },
                                "Assign"
                            }
                        }
                    }
                }
            }
        }
    }
}
