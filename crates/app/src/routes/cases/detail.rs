use dioxus::prelude::*;
use shared_types::CaseResponse;
use shared_ui::components::{
    AlertDialogAction, AlertDialogActions, AlertDialogCancel, AlertDialogContent,
    AlertDialogDescription, AlertDialogRoot, AlertDialogTitle, Button, ButtonVariant, Card,
    CardContent, PageActions, PageHeader, PageTitle, Skeleton, TabContent, TabList, TabTrigger,
    Tabs,
};

use super::form_sheet::{CaseFormSheet, FormMode};
use super::tabs::{
    docket_container::DocketContainerTab, overview::OverviewTab, parties::PartiesTab,
    scheduling::SchedulingTab, sentencing::SentencingTab,
};
use crate::auth::{can, use_user_role, Action};
use crate::routes::Route;
use crate::CourtContext;

#[component]
pub fn CaseDetailPage(id: String) -> Element {
    let ctx = use_context::<CourtContext>();
    let court_id = ctx.court_id.read().clone();
    let case_id = id.clone();

    let mut data = use_resource(move || {
        let court = court_id.clone();
        let cid = case_id.clone();
        async move {
            match server::api::get_case(court, cid).await {
                Ok(json) => serde_json::from_str::<CaseResponse>(&json).ok(),
                Err(_) => None,
            }
        }
    });

    let mut show_edit = use_signal(|| false);

    rsx! {
        div { class: "container",
            match &*data.read() {
                Some(Some(c)) => rsx! {
                    CaseDetailView { case_item: c.clone(), id: id.clone(), show_edit: show_edit }
                    CaseFormSheet {
                        mode: FormMode::Edit,
                        initial: Some(c.clone()),
                        open: show_edit(),
                        on_close: move |_| show_edit.set(false),
                        on_saved: move |_| data.restart(),
                    }
                },
                Some(None) => rsx! {
                    PageHeader {
                        PageTitle { "Case Not Found" }
                        PageActions {
                            Link { to: Route::CaseList {},
                                Button { variant: ButtonVariant::Secondary, "Back to List" }
                            }
                        }
                    }
                    Card {
                        CardContent {
                            p { "The requested case could not be found." }
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
fn CaseDetailView(case_item: CaseResponse, id: String, show_edit: Signal<bool>) -> Element {
    let ctx = use_context::<CourtContext>();
    let role = use_user_role();

    let mut show_delete_confirm = use_signal(|| false);
    let mut deleting = use_signal(|| false);

    let detail_id = id.clone();
    let handle_delete = move |_: MouseEvent| {
        let court = ctx.court_id.read().clone();
        let did = detail_id.clone();
        spawn(async move {
            deleting.set(true);
            match server::api::delete_case(court, did).await {
                Ok(_) => {
                    navigator().push(Route::CaseList {});
                }
                Err(_) => {
                    deleting.set(false);
                    show_delete_confirm.set(false);
                }
            }
        });
    };

    rsx! {
        PageHeader {
            PageTitle { "{case_item.title}" }
            PageActions {
                Link { to: Route::Dashboard {},
                    Button { variant: ButtonVariant::Secondary, "Queue" }
                }
                Link { to: Route::CaseList {},
                    Button { variant: ButtonVariant::Secondary, "Cases" }
                }
                if can(&role, Action::Edit) {
                    Button {
                        variant: ButtonVariant::Primary,
                        onclick: move |_| show_edit.set(true),
                        "Edit"
                    }
                }
                Button {
                    variant: ButtonVariant::Destructive,
                    onclick: move |_| show_delete_confirm.set(true),
                    "Delete"
                }
            }
        }

        // Delete confirmation dialog
        AlertDialogRoot {
            open: show_delete_confirm(),
            on_open_change: move |v| show_delete_confirm.set(v),
            AlertDialogContent {
                AlertDialogTitle { "Delete Case" }
                AlertDialogDescription {
                    "Are you sure you want to delete this case? This action cannot be undone."
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

        Tabs { default_value: "overview", horizontal: true,
            TabList {
                TabTrigger { value: "overview", index: 0usize, "Overview" }
                TabTrigger { value: "docket", index: 1usize, "Docket" }
                TabTrigger { value: "parties", index: 2usize, "Parties" }
                TabTrigger { value: "scheduling", index: 3usize, "Scheduling" }
                TabTrigger { value: "sentencing", index: 4usize, "Sentencing" }
            }
            TabContent { value: "overview", index: 0usize,
                OverviewTab { case_item: case_item.clone() }
            }
            TabContent { value: "docket", index: 1usize,
                DocketContainerTab { case_id: id.clone() }
            }
            TabContent { value: "parties", index: 2usize,
                PartiesTab { case_id: id.clone() }
            }
            TabContent { value: "scheduling", index: 3usize,
                SchedulingTab { case_id: id.clone() }
            }
            TabContent { value: "sentencing", index: 4usize,
                SentencingTab { case_id: id.clone() }
            }
        }
    }
}
