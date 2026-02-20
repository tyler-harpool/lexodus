use dioxus::prelude::*;
use shared_types::DeadlineResponse;
use shared_ui::components::{
    AlertDialogAction, AlertDialogActions, AlertDialogCancel, AlertDialogContent,
    AlertDialogDescription, AlertDialogRoot, AlertDialogTitle, Badge, BadgeVariant, Button,
    ButtonVariant, Card, CardContent, CardHeader, CardTitle, DetailFooter, DetailGrid, DetailItem,
    DetailList, PageActions, PageHeader, PageTitle, Skeleton,
};

use super::form_sheet::{DeadlineFormSheet, FormMode};
use crate::auth::{can, use_user_role, Action};
use crate::routes::Route;
use crate::CourtContext;

#[component]
pub fn DeadlineDetailPage(id: String) -> Element {
    let ctx = use_context::<CourtContext>();
    let court_id = ctx.court_id.read().clone();
    let dl_id = id.clone();
    let role = use_user_role();
    let mut show_edit = use_signal(|| false);

    let mut data = use_resource(move || {
        let court = court_id.clone();
        let deadline_id = dl_id.clone();
        async move {
            server::api::get_deadline(court, deadline_id).await.ok()
        }
    });

    rsx! {
        div { class: "container",
            match &*data.read() {
                Some(Some(dl)) => rsx! {
                    DeadlineDetailView {
                        deadline: dl.clone(),
                        id: id.clone(),
                        role: role.clone(),
                        show_edit: show_edit,
                    }

                    DeadlineFormSheet {
                        mode: FormMode::Edit,
                        initial: Some(dl.clone()),
                        open: show_edit(),
                        on_close: move |_| show_edit.set(false),
                        on_saved: move |_| data.restart(),
                    }
                },
                Some(None) => rsx! {
                    PageHeader {
                        PageTitle { "Deadline Not Found" }
                        PageActions {
                            Link { to: Route::DeadlineList {},
                                Button { variant: ButtonVariant::Secondary, "Back to List" }
                            }
                        }
                    }
                    Card {
                        CardContent {
                            p { "The requested deadline could not be found." }
                        }
                    }
                },
                None => rsx! {
                    div { class: "loading",
                        Skeleton {}
                        Skeleton {}
                    }
                },
            }
        }
    }
}

#[component]
fn DeadlineDetailView(
    deadline: DeadlineResponse,
    id: String,
    role: shared_types::UserRole,
    show_edit: Signal<bool>,
) -> Element {
    let ctx = use_context::<CourtContext>();

    let mut show_delete_confirm = use_signal(|| false);
    let mut deleting = use_signal(|| false);

    let detail_id = id.clone();
    let handle_delete = move |_: MouseEvent| {
        let court = ctx.court_id.read().clone();
        let did = detail_id.clone();
        spawn(async move {
            deleting.set(true);
            match server::api::delete_deadline(court, did).await {
                Ok(_) => {
                    navigator().push(Route::DeadlineList {});
                }
                Err(_) => {
                    deleting.set(false);
                    show_delete_confirm.set(false);
                }
            }
        });
    };

    let badge_variant = match deadline.status.as_str() {
        "open" => BadgeVariant::Primary,
        "met" => BadgeVariant::Secondary,
        "extended" => BadgeVariant::Outline,
        "cancelled" | "expired" => BadgeVariant::Destructive,
        _ => BadgeVariant::Secondary,
    };

    let display_date = if deadline.due_at.len() >= 16 {
        deadline.due_at[..16].replace('T', " ")
    } else {
        deadline.due_at.clone()
    };

    rsx! {
        PageHeader {
            PageTitle { "{deadline.title}" }
            PageActions {
                Link { to: Route::DeadlineList {},
                    Button { variant: ButtonVariant::Secondary, "Back to List" }
                }
                if can(&role, Action::EditCase) {
                    Button {
                        variant: ButtonVariant::Primary,
                        onclick: move |_| show_edit.set(true),
                        "Edit"
                    }
                }
                if can(&role, Action::DeleteCase) {
                    Button {
                        variant: ButtonVariant::Destructive,
                        onclick: move |_| show_delete_confirm.set(true),
                        "Delete"
                    }
                }
            }
        }

        AlertDialogRoot {
            open: show_delete_confirm(),
            on_open_change: move |v| show_delete_confirm.set(v),
            AlertDialogContent {
                AlertDialogTitle { "Delete Deadline" }
                AlertDialogDescription {
                    "Are you sure you want to delete this deadline? This action cannot be undone."
                }
                AlertDialogActions {
                    AlertDialogCancel { "Cancel" }
                    AlertDialogAction {
                        on_click: handle_delete,
                        if *deleting.read() { "Deleting..." } else { "Delete" }
                    }
                }
            }
        }

        DetailGrid {
            Card {
                CardHeader { CardTitle { "Deadline Details" } }
                CardContent {
                    DetailList {
                        DetailItem { label: "Due Date", value: display_date }
                        DetailItem { label: "Status",
                            Badge { variant: badge_variant, "{deadline.status}" }
                        }
                        if let Some(ref rule) = deadline.rule_code {
                            DetailItem { label: "Rule Code", value: rule.clone() }
                        }
                        if let Some(ref case_id) = deadline.case_id {
                            DetailItem { label: "Case ID", value: case_id.clone() }
                        }
                    }
                }
            }

            Card {
                CardHeader { CardTitle { "Timestamps" } }
                CardContent {
                    DetailList {
                        DetailItem { label: "Created", value: format_date(&deadline.created_at) }
                        DetailItem { label: "Updated", value: format_date(&deadline.updated_at) }
                    }
                }
            }

            if let Some(ref notes) = deadline.notes {
                Card {
                    CardHeader { CardTitle { "Notes" } }
                    CardContent {
                        p { "{notes}" }
                    }
                }
            }
        }

        DetailFooter {
            span { "ID: {deadline.id}" }
        }
    }
}

fn format_date(date_str: &str) -> String {
    if date_str.len() >= 10 {
        date_str[..10].to_string()
    } else {
        date_str.to_string()
    }
}
