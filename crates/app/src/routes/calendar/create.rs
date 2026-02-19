use dioxus::prelude::*;
use shared_types::{CalendarEntryResponse, CaseSearchResponse};
use shared_ui::components::{
    Button, ButtonVariant, Card, CardContent, CardHeader, CardTitle, Form, FormSelect, Input,
    PageActions, PageHeader, PageTitle,
};

use crate::routes::Route;
use crate::CourtContext;

#[component]
pub fn CalendarCreatePage() -> Element {
    let ctx = use_context::<CourtContext>();

    let mut case_id = use_signal(String::new);
    let mut judge_id = use_signal(String::new);

    // Load cases and judges for dropdown selectors
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
    let mut event_type = use_signal(|| "motion_hearing".to_string());
    let mut scheduled_date = use_signal(String::new);
    let mut duration_minutes = use_signal(|| "60".to_string());
    let mut courtroom = use_signal(String::new);
    let mut description = use_signal(String::new);
    let mut participants = use_signal(String::new);
    let mut is_public = use_signal(|| true);

    let mut error_msg = use_signal(|| None::<String>);
    let mut submitting = use_signal(|| false);

    let handle_submit = move |_evt: FormEvent| {
        let court_id = ctx.court_id.read().clone();

        let participants_vec: Vec<String> = participants
            .read()
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();

        let body = serde_json::json!({
            "case_id": case_id.read().clone(),
            "judge_id": judge_id.read().clone(),
            "event_type": event_type.read().clone(),
            "scheduled_date": scheduled_date.read().clone(),
            "duration_minutes": duration_minutes.read().parse::<i32>().unwrap_or(60),
            "courtroom": courtroom.read().clone(),
            "description": description.read().clone(),
            "participants": participants_vec,
            "is_public": *is_public.read(),
        });

        spawn(async move {
            submitting.set(true);
            error_msg.set(None);

            match server::api::schedule_calendar_event(court_id, body.to_string()).await {
                Ok(json) => {
                    if let Ok(entry) = serde_json::from_str::<CalendarEntryResponse>(&json) {
                        let nav = navigator();
                        nav.push(Route::CalendarDetail { id: entry.id });
                    }
                }
                Err(e) => {
                    error_msg.set(Some(format!("{}", e)));
                }
            }
            submitting.set(false);
        });
    };

    rsx! {
        div { class: "container",
            PageHeader {
                PageTitle { "Schedule Event" }
                PageActions {
                    Link { to: Route::CalendarList {},
                        Button { variant: ButtonVariant::Secondary, "Back to Calendar" }
                    }
                }
            }

            if let Some(err) = error_msg.read().as_ref() {
                div { class: "alert alert-error", "{err}" }
            }

            Card {
                Form { onsubmit: handle_submit,
                    CardHeader {
                        CardTitle { "Event Details" }
                    }
                    CardContent {
                        div { class: "form-row",
                            div { class: "form-group",
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
                            div { class: "form-group",
                                label { class: "input-label", "Judge *" }
                                select {
                                    class: "input",
                                    value: judge_id.read().clone(),
                                    onchange: move |e: FormEvent| judge_id.set(e.value().to_string()),
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
                        }
                        div { class: "form-row",
                            div { class: "form-group",
                                FormSelect {
                                    label: "Event Type *",
                                    value: "{event_type}",
                                    onchange: move |evt: Event<FormData>| event_type.set(evt.value().to_string()),
                                    option { value: "initial_appearance", "Initial Appearance" }
                                    option { value: "arraignment", "Arraignment" }
                                    option { value: "bail_hearing", "Bail Hearing" }
                                    option { value: "plea_hearing", "Plea Hearing" }
                                    option { value: "trial_date", "Trial Date" }
                                    option { value: "sentencing", "Sentencing" }
                                    option { value: "violation_hearing", "Violation Hearing" }
                                    option { value: "status_conference", "Status Conference" }
                                    option { value: "scheduling_conference", "Scheduling Conference" }
                                    option { value: "settlement_conference", "Settlement Conference" }
                                    option { value: "pretrial_conference", "Pretrial Conference" }
                                    option { value: "motion_hearing", "Motion Hearing" }
                                    option { value: "evidentiary_hearing", "Evidentiary Hearing" }
                                    option { value: "jury_selection", "Jury Selection" }
                                    option { value: "jury_trial", "Jury Trial" }
                                    option { value: "bench_trial", "Bench Trial" }
                                    option { value: "show_cause_hearing", "Show Cause Hearing" }
                                    option { value: "contempt_hearing", "Contempt Hearing" }
                                    option { value: "emergency_hearing", "Emergency Hearing" }
                                    option { value: "telephonic", "Telephonic" }
                                    option { value: "video_conference", "Video Conference" }
                                }
                            }
                            Input {
                                label: "Duration (minutes) *",
                                input_type: "number",
                                value: duration_minutes.read().clone(),
                                on_input: move |e: FormEvent| duration_minutes.set(e.value().to_string()),
                            }
                        }
                    }

                    CardHeader {
                        CardTitle { "Scheduling" }
                    }
                    CardContent {
                        div { class: "form-row",
                            Input {
                                label: "Scheduled Date/Time (ISO 8601) *",
                                value: scheduled_date.read().clone(),
                                on_input: move |e: FormEvent| scheduled_date.set(e.value().to_string()),
                                placeholder: "2026-06-15T09:00:00Z",
                            }
                            Input {
                                label: "Courtroom *",
                                value: courtroom.read().clone(),
                                on_input: move |e: FormEvent| courtroom.set(e.value().to_string()),
                                placeholder: "Courtroom 4A",
                            }
                        }
                    }

                    CardHeader {
                        CardTitle { "Description & Participants" }
                    }
                    CardContent {
                        div { class: "form-row",
                            div { class: "form-group-wide",
                                Input {
                                    label: "Description *",
                                    value: description.read().clone(),
                                    on_input: move |e: FormEvent| description.set(e.value().to_string()),
                                    placeholder: "Brief description of the event",
                                }
                            }
                        }
                        div { class: "form-row",
                            div { class: "form-group-wide",
                                Input {
                                    label: "Participants (comma-separated)",
                                    value: participants.read().clone(),
                                    on_input: move |e: FormEvent| participants.set(e.value().to_string()),
                                    placeholder: "Prosecutor, Defense Counsel, Witness",
                                }
                            }
                        }
                        div { class: "form-row",
                            div { class: "form-group",
                                label { "Public Event" }
                                input {
                                    r#type: "checkbox",
                                    checked: *is_public.read(),
                                    onchange: move |evt| {
                                        is_public.set(evt.value() == "true");
                                    },
                                }
                            }
                        }
                    }

                    div { class: "form-actions",
                        button {
                            class: "button",
                            "data-style": "primary",
                            r#type: "submit",
                            disabled: *submitting.read(),
                            if *submitting.read() { "Scheduling..." } else { "Schedule Event" }
                        }
                        Link { to: Route::CalendarList {},
                            Button { variant: ButtonVariant::Secondary, "Cancel" }
                        }
                    }
                }
            }
        }
    }
}
