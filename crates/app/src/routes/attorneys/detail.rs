use dioxus::prelude::*;
use shared_ui::components::{
    AlertDialogAction, AlertDialogActions, AlertDialogCancel, AlertDialogContent,
    AlertDialogDescription, AlertDialogRoot, AlertDialogTitle, Button, ButtonVariant, Card,
    CardContent, PageActions, PageHeader, PageTitle, Skeleton, TabContent, TabList, TabTrigger,
    Tabs,
};

use super::form_sheet::{AttorneyFormSheet, FormMode};
use super::tabs::{
    admissions::AdmissionsTab, cases::AttorneyCasesTab, cja::CjaTab, discipline::DisciplineTab,
    metrics::AttorneyMetricsTab, pro_hac_vice::ProHacViceTab, profile::ProfileTab,
};
use crate::auth::{can, use_user_role, Action};
use crate::routes::Route;
use crate::CourtContext;

#[component]
pub fn AttorneyDetailPage(id: String) -> Element {
    let ctx = use_context::<CourtContext>();
    let court_id = ctx.court_id.read().clone();
    let attorney_id = id.clone();

    let role = use_user_role();
    let mut show_edit = use_signal(|| false);
    let mut show_delete_confirm = use_signal(|| false);
    let mut deleting = use_signal(|| false);

    let mut data = use_resource(move || {
        let court = court_id.clone();
        let aid = attorney_id.clone();
        async move {
            server::api::get_attorney(court, aid).await.ok()
        }
    });

    let detail_id = id.clone();
    let handle_delete = move |_: MouseEvent| {
        let court = ctx.court_id.read().clone();
        let aid = detail_id.clone();
        spawn(async move {
            deleting.set(true);
            match server::api::delete_attorney(court, aid).await {
                Ok(()) => {
                    let nav = navigator();
                    nav.push(Route::AttorneyList {});
                }
                Err(_) => {
                    deleting.set(false);
                    show_delete_confirm.set(false);
                }
            }
        });
    };

    rsx! {
        div { class: "container",
            match &*data.read() {
                Some(Some(att)) => rsx! {
                    PageHeader {
                        PageTitle { "{att.last_name}, {att.first_name}" }
                        PageActions {
                            Link { to: Route::AttorneyList {},
                                Button { variant: ButtonVariant::Secondary, "Back to List" }
                            }
                            if can(&role, Action::ManageAttorneys) {
                                Button {
                                    variant: ButtonVariant::Primary,
                                    onclick: move |_| show_edit.set(true),
                                    "Edit"
                                }
                            }
                            if can(&role, Action::ManageAttorneys) {
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
                            AlertDialogTitle { "Delete Attorney" }
                            AlertDialogDescription {
                                "Are you sure you want to delete this attorney? This action cannot be undone."
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

                    Tabs { default_value: "profile", horizontal: true,
                        TabList {
                            TabTrigger { value: "profile", index: 0usize, "Profile" }
                            TabTrigger { value: "admissions", index: 1usize, "Admissions" }
                            TabTrigger { value: "cja", index: 2usize, "CJA" }
                            TabTrigger { value: "cases", index: 3usize, "Cases" }
                            TabTrigger { value: "metrics", index: 4usize, "Metrics" }
                            TabTrigger { value: "discipline", index: 5usize, "Discipline" }
                            TabTrigger { value: "pro-hac-vice", index: 6usize, "Pro Hac Vice" }
                        }
                        TabContent { value: "profile", index: 0usize,
                            ProfileTab { attorney: att.clone(), attorney_id: id.clone() }
                        }
                        TabContent { value: "admissions", index: 1usize,
                            AdmissionsTab { attorney_id: id.clone() }
                        }
                        TabContent { value: "cja", index: 2usize,
                            CjaTab { attorney_id: id.clone() }
                        }
                        TabContent { value: "cases", index: 3usize,
                            AttorneyCasesTab { attorney_id: id.clone() }
                        }
                        TabContent { value: "metrics", index: 4usize,
                            AttorneyMetricsTab { attorney: att.clone() }
                        }
                        TabContent { value: "discipline", index: 5usize,
                            DisciplineTab { attorney_id: id.clone() }
                        }
                        TabContent { value: "pro-hac-vice", index: 6usize,
                            ProHacViceTab { attorney_id: id.clone() }
                        }
                    }

                    AttorneyFormSheet {
                        mode: FormMode::Edit,
                        initial: Some(att.clone()),
                        open: show_edit(),
                        on_close: move |_| show_edit.set(false),
                        on_saved: move |_| data.restart(),
                    }
                },
                Some(None) => rsx! {
                    Card {
                        CardContent {
                            div { class: "empty-state",
                                h2 { "Attorney Not Found" }
                                p { "The attorney you're looking for doesn't exist in this court district." }
                                Link { to: Route::AttorneyList {},
                                    Button { "Back to List" }
                                }
                            }
                        }
                    }
                },
                None => rsx! {
                    div { class: "loading",
                        Skeleton {}
                        Skeleton {}
                        Skeleton {}
                    }
                },
            }
        }
    }
}
