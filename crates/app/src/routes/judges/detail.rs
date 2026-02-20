use dioxus::prelude::*;
use shared_types::JudgeResponse;
use shared_ui::components::{
    Badge, BadgeVariant, Button, ButtonVariant, Card, CardContent, PageActions, PageHeader,
    PageTitle, Skeleton, TabContent, TabList, TabTrigger, Tabs,
};

use super::form_sheet::{JudgeFormSheet, FormMode};
use super::tabs::{
    calendar::JudgeCalendarTab, caseload::CaseloadTab, conflicts::ConflictsTab,
    opinions::OpinionsTab, profile::JudgeProfileTab, vacation::VacationTab,
    workload::WorkloadTab,
};
use crate::auth::{can, use_user_role, Action};
use crate::routes::Route;
use crate::CourtContext;

#[component]
pub fn JudgeDetailPage(id: String) -> Element {
    let ctx = use_context::<CourtContext>();
    let court_id = ctx.court_id.read().clone();
    let judge_id = id.clone();
    let role = use_user_role();
    let mut show_edit = use_signal(|| false);

    let mut data = use_resource(move || {
        let court = court_id.clone();
        let jid = judge_id.clone();
        async move { server::api::get_judge(court, jid).await.ok() }
    });

    rsx! {
        div { class: "container",
            match &*data.read() {
                Some(Some(judge)) => {
                    rsx! {
                        JudgeDetailView {
                            judge: judge.clone(),
                            id: id.clone(),
                            role: role.clone(),
                            show_edit: show_edit,
                        }

                        JudgeFormSheet {
                            mode: FormMode::Edit,
                            initial: Some(judge.clone()),
                            open: show_edit(),
                            on_close: move |_| show_edit.set(false),
                            on_saved: move |_| data.restart(),
                        }
                    }
                },
                Some(None) => rsx! {
                    PageHeader {
                        PageTitle { "Judge Not Found" }
                        PageActions {
                            Link { to: Route::JudgeList {},
                                Button { variant: ButtonVariant::Secondary, "Back to List" }
                            }
                        }
                    }
                    Card {
                        CardContent {
                            p { "The requested judge could not be found." }
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
fn JudgeDetailView(
    judge: JudgeResponse,
    id: String,
    role: shared_types::UserRole,
    show_edit: Signal<bool>,
) -> Element {
    let name = judge.name.clone();
    let title = judge.title.clone();
    let status = judge.status.clone();

    rsx! {
        PageHeader {
            PageTitle { "{name}" }
            PageActions {
                Badge {
                    variant: status_badge_variant(&status),
                    "{title} — {status}"
                }
                Link { to: Route::JudgeList {},
                    Button { variant: ButtonVariant::Secondary, "Back to List" }
                }
                if can(&role, Action::ManageJudges) {
                    Button {
                        variant: ButtonVariant::Primary,
                        onclick: move |_| show_edit.set(true),
                        "Edit"
                    }
                }
            }
        }

        Tabs { default_value: "profile", horizontal: true,
            TabList {
                TabTrigger { value: "profile", index: 0usize, "Profile" }
                TabTrigger { value: "caseload", index: 1usize, "Caseload" }
                TabTrigger { value: "calendar", index: 2usize, "Calendar" }
                TabTrigger { value: "opinions", index: 3usize, "Opinions" }
                TabTrigger { value: "conflicts", index: 4usize, "Conflicts" }
                TabTrigger { value: "workload", index: 5usize, "Workload" }
                TabTrigger { value: "vacation", index: 6usize, "Vacation" }
            }
            TabContent { value: "profile", index: 0usize,
                JudgeProfileTab { judge: judge.clone() }
            }
            TabContent { value: "caseload", index: 1usize,
                CaseloadTab { judge_id: id.clone() }
            }
            TabContent { value: "calendar", index: 2usize,
                JudgeCalendarTab { judge_id: id.clone() }
            }
            TabContent { value: "opinions", index: 3usize,
                OpinionsTab { judge_id: id.clone() }
            }
            TabContent { value: "conflicts", index: 4usize,
                ConflictsTab { judge_id: id.clone() }
            }
            TabContent { value: "workload", index: 5usize,
                WorkloadTab { judge_id: id.clone(), judge: judge }
            }
            TabContent { value: "vacation", index: 6usize,
                VacationTab { judge_id: id.clone() }
            }
        }
    }
}

fn status_badge_variant(status: &str) -> BadgeVariant {
    match status {
        "Active" => BadgeVariant::Primary,
        "Senior" => BadgeVariant::Secondary,
        "Inactive" => BadgeVariant::Outline,
        "Retired" => BadgeVariant::Outline,
        "Deceased" => BadgeVariant::Destructive,
        _ => BadgeVariant::Secondary,
    }
}
