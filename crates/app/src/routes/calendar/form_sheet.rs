use dioxus::prelude::*;
use shared_types::{CalendarEntryResponse, ScheduleEventRequest};
use shared_ui::components::{
    AlertDialogAction, AlertDialogActions, AlertDialogCancel, AlertDialogContent,
    AlertDialogDescription, AlertDialogRoot, AlertDialogTitle, Form, FormSelect, Input, Separator,
    Sheet, SheetClose, SheetContent, SheetDescription, SheetFooter, SheetHeader, SheetSide,
    SheetTitle, Textarea,
};
use shared_ui::{use_toast, ToastOptions};

use crate::CourtContext;

/// Controls whether the form is in Create or Edit mode.
/// Calendar events currently only support Create (no general update API).
#[derive(Clone, Copy, PartialEq)]
pub enum FormMode {
    Create,
}

/// Calendar event scheduling form rendered inside a Sheet.
/// Loads cases and judges for dropdown selectors.
#[component]
pub fn CalendarFormSheet(
    mode: FormMode,
    initial: Option<CalendarEntryResponse>,
    open: bool,
    on_close: EventHandler<()>,
    on_saved: EventHandler<()>,
    /// Pre-fill the scheduled date (e.g. from calendar date click).
    #[props(default)]
    prefill_date: Option<String>,
) -> Element {
    let ctx = use_context::<CourtContext>();
    let toast = use_toast();

    // --- Form field signals ---
    let mut case_id = use_signal(String::new);
    let mut judge_id = use_signal(String::new);
    let mut event_type = use_signal(|| "motion_hearing".to_string());
    let mut scheduled_date = use_signal(String::new);
    let mut duration = use_signal(|| "60".to_string());
    let mut courtroom = use_signal(String::new);
    let mut description = use_signal(String::new);
    let mut participants = use_signal(String::new);

    // --- Load cases and judges for selectors ---
    let cases_for_select = use_resource(move || {
        let court = ctx.court_id.read().clone();
        async move {
            server::api::search_cases(court, None, None, None, None, None, Some(100))
                .await
                .ok()
                .map(|r| r.cases)
        }
    });

    let judges_for_select = use_resource(move || {
        let court = ctx.court_id.read().clone();
        async move { server::api::list_judges(court).await.ok() }
    });

    // --- Hydration ---
    let mut hydrated = use_signal(|| false);
    let prefill = prefill_date.clone();

    use_effect(move || {
        if !open {
            return;
        }
        if !*hydrated.read() {
            hydrated.set(true);
            // Reset to defaults
            case_id.set(String::new());
            judge_id.set(String::new());
            event_type.set("motion_hearing".to_string());
            duration.set("60".to_string());
            courtroom.set(String::new());
            description.set(String::new());
            participants.set(String::new());
            // Apply prefilled date if provided
            if let Some(ref d) = prefill {
                scheduled_date.set(d.clone());
            } else {
                scheduled_date.set(String::new());
            }
        }
    });

    // Reset hydrated flag when sheet closes so next open re-hydrates
    use_effect(move || {
        if !open {
            hydrated.set(false);
        }
    });

    // --- Dirty state tracking ---
    let mut initial_snapshot = use_signal(String::new);

    use_effect(move || {
        if open {
            initial_snapshot.set(snapshot(
                &case_id,
                &judge_id,
                &event_type,
                &scheduled_date,
                &duration,
                &courtroom,
                &description,
                &participants,
            ));
        }
    });

    let is_dirty = move || {
        let current = snapshot(
            &case_id,
            &judge_id,
            &event_type,
            &scheduled_date,
            &duration,
            &courtroom,
            &description,
            &participants,
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

        let participants_vec: Vec<String> = participants
            .read()
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();

        let case_uuid = uuid::Uuid::parse_str(&case_id.read()).unwrap_or_default();
        let judge_uuid = uuid::Uuid::parse_str(&judge_id.read()).unwrap_or_default();
        let sched_date = chrono::DateTime::parse_from_rfc3339(&scheduled_date.read())
            .map(|d| d.with_timezone(&chrono::Utc))
            .unwrap_or_else(|_| chrono::Utc::now());

        let req = ScheduleEventRequest {
            case_id: case_uuid,
            judge_id: judge_uuid,
            event_type: event_type.read().clone(),
            scheduled_date: sched_date,
            duration_minutes: duration.read().parse::<i32>().unwrap_or(60),
            courtroom: courtroom.read().clone(),
            description: description.read().clone(),
            participants: participants_vec,
            is_public: true,
        };

        spawn(async move {
            in_flight.set(true);
            match server::api::schedule_calendar_event(court, req).await {
                Ok(_) => {
                    on_saved.call(());
                    on_close.call(());
                    toast.success(
                        "Event scheduled successfully".to_string(),
                        ToastOptions::new(),
                    );
                }
                Err(e) => {
                    toast.error(format!("{e}"), ToastOptions::new());
                }
            }
            in_flight.set(false);
        });
    };

    rsx! {
        Sheet {
            open,
            on_close: try_close,
            side: SheetSide::Right,
            SheetContent {
                SheetHeader {
                    SheetTitle { "Schedule Event" }
                    SheetDescription {
                        "Fill in the details to schedule a new calendar event."
                    }
                    SheetClose { on_close: try_close }
                }

                Form {
                    onsubmit: handle_save,

                    div {
                        class: "sheet-form",

                        FormSelect {
                            label: "Event Type *",
                            value: event_type.read().clone(),
                            onchange: move |evt: Event<FormData>| event_type.set(evt.value()),
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

                        Input {
                            label: "Scheduled Date/Time *",
                            value: scheduled_date.read().clone(),
                            on_input: move |e: FormEvent| scheduled_date.set(e.value()),
                            placeholder: "2026-06-15T09:00:00Z",
                        }

                        Separator {}

                        label { class: "input-label", "Case *" }
                        select {
                            class: "input",
                            value: case_id.read().clone(),
                            onchange: move |e: FormEvent| case_id.set(e.value()),
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

                        label { class: "input-label", "Judge *" }
                        select {
                            class: "input",
                            value: judge_id.read().clone(),
                            onchange: move |e: FormEvent| judge_id.set(e.value()),
                            option { value: "", "-- Select a judge --" }
                            {match &*judges_for_select.read() {
                                Some(Some(judges)) => rsx! {
                                    for j in judges.iter() {
                                        option {
                                            value: "{j.id}",
                                            "{j.name}"
                                        }
                                    }
                                },
                                _ => rsx! {
                                    option { value: "", disabled: true, "Loading judges..." }
                                },
                            }}
                        }

                        Separator {}

                        Input {
                            label: "Duration (minutes) *",
                            input_type: "number",
                            value: duration.read().clone(),
                            on_input: move |e: FormEvent| duration.set(e.value()),
                        }

                        Input {
                            label: "Courtroom *",
                            value: courtroom.read().clone(),
                            on_input: move |e: FormEvent| courtroom.set(e.value()),
                            placeholder: "Courtroom 4A",
                        }

                        Textarea {
                            label: "Description *",
                            value: description.read().clone(),
                            on_input: move |e: FormEvent| description.set(e.value()),
                            placeholder: "Brief description of the event",
                        }

                        Input {
                            label: "Participants (comma-separated)",
                            value: participants.read().clone(),
                            on_input: move |e: FormEvent| participants.set(e.value()),
                            placeholder: "Prosecutor, Defense Counsel",
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
                                if *in_flight.read() { "Scheduling..." } else { "Schedule" }
                            }
                        }
                    }
                }
            }
        }

        // Discard changes confirmation
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
    case_id: &Signal<String>,
    judge_id: &Signal<String>,
    event_type: &Signal<String>,
    scheduled_date: &Signal<String>,
    duration: &Signal<String>,
    courtroom: &Signal<String>,
    description: &Signal<String>,
    participants: &Signal<String>,
) -> String {
    serde_json::json!({
        "case_id": case_id.read().clone(),
        "judge_id": judge_id.read().clone(),
        "event_type": event_type.read().clone(),
        "scheduled_date": scheduled_date.read().clone(),
        "duration": duration.read().clone(),
        "courtroom": courtroom.read().clone(),
        "description": description.read().clone(),
        "participants": participants.read().clone(),
    })
    .to_string()
}
