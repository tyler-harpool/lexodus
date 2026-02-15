use dioxus::prelude::*;
use shared_ui::{
    use_toast, AlertDialogAction, AlertDialogActions, AlertDialogCancel, AlertDialogContent,
    AlertDialogDescription, AlertDialogRoot, AlertDialogTitle, Badge, BadgeVariant, Button,
    ButtonVariant, Calendar, CalendarGrid, CalendarHeader, CalendarMonthTitle, CalendarNavigation,
    CalendarNextMonthButton, CalendarPreviousMonthButton, CalendarSelectMonth, CalendarSelectYear,
    Collapsible, CollapsibleContent, CollapsibleTrigger, Date, Form, Input, Label, Separator,
    Sheet, SheetClose, SheetContent, SheetDescription, SheetFooter, SheetHeader, SheetSide,
    SheetTitle, Textarea, ToastOptions, UtcDateTime,
};

/// Advanced settings section: calendar, event sheet, and danger zone.
#[component]
pub fn AdvancedSection() -> Element {
    let toast = use_toast();

    let mut selected_date = use_signal(|| None::<Date>);
    let mut view_date = use_signal(|| UtcDateTime::now().date());
    let mut event_sheet_open = use_signal(|| false);
    let mut event_title = use_signal(String::new);
    let mut event_notes = use_signal(String::new);
    let mut delete_dialog_open = use_signal(|| false);

    rsx! {
        Collapsible {
            CollapsibleTrigger {
                Button {
                    variant: ButtonVariant::Outline,
                    "Show Advanced Settings"
                }
            }

            CollapsibleContent {
                div {
                    class: "settings-section-lg",

                    // Calendar widget
                    div {
                        class: "calendar-container",
                        Calendar {
                            selected_date: selected_date,
                            on_date_change: move |date: Option<Date>| {
                                selected_date.set(date);
                                if let Some(d) = date {
                                    toast.info(
                                        format!("Selected: {} {}-{:02}-{:02}", d.weekday(), d.year(), d.month() as u8, d.day()),
                                        ToastOptions::new(),
                                    );
                                    event_title.set(String::new());
                                    event_notes.set(String::new());
                                    event_sheet_open.set(true);
                                }
                            },
                            view_date: view_date,
                            on_view_change: move |new_view: Date| {
                                view_date.set(new_view);
                            },
                            CalendarHeader {
                                CalendarNavigation {
                                    CalendarPreviousMonthButton { "\u{2039}" }
                                    CalendarMonthTitle {}
                                    CalendarNextMonthButton { "\u{203a}" }
                                }
                            }
                            CalendarGrid {}
                            CalendarSelectMonth {}
                            CalendarSelectYear {}
                        }

                        if let Some(date) = selected_date() {
                            div {
                                class: "selected-date-display",
                                span { "Selected date:" }
                                Badge {
                                    variant: BadgeVariant::Primary,
                                    "{date.year()}-{date.month() as u8:02}-{date.day():02}"
                                }
                            }
                        }
                    }

                    Separator {}

                    // Danger zone
                    div {
                        class: "danger-zone-stack",
                        p {
                            class: "danger-zone-text",
                            "Irreversible actions that affect your account permanently."
                        }
                        Button {
                            variant: ButtonVariant::Destructive,
                            onclick: move |_| {
                                delete_dialog_open.set(true);
                            },
                            "Delete Account"
                        }
                    }
                }
            }
        }

        // Event Sheet: slides in when a date is selected
        Sheet {
            open: event_sheet_open(),
            on_close: move |_| event_sheet_open.set(false),
            side: SheetSide::Right,

            SheetHeader {
                SheetTitle {
                    if selected_date().is_some() {
                        "Schedule Event"
                    }
                }
                SheetDescription {
                    if let Some(date) = selected_date() {
                        span {
                            "{date.weekday()}, {date.month()} {date.day()}, {date.year()}"
                        }
                    }
                }
            }

            SheetContent {
                Form {
                    onsubmit: move |_| {},
                    div {
                        class: "settings-form",
                        div {
                            class: "settings-field",
                            Label { html_for: "event-title", "Event Title" }
                            Input {
                                value: event_title(),
                                placeholder: "Meeting, Deadline, Reminder...",
                                label: "",
                                on_input: move |evt: FormEvent| {
                                    event_title.set(evt.value());
                                },
                            }
                        }
                        div {
                            class: "settings-field",
                            Label { html_for: "event-notes", "Notes" }
                            Textarea {
                                value: event_notes(),
                                placeholder: "Add details about this event...",
                                on_input: move |evt: FormEvent| {
                                    event_notes.set(evt.value());
                                },
                            }
                        }
                    }
                }
            }

            SheetFooter {
                SheetClose {
                    on_close: move |_| event_sheet_open.set(false),
                }
                Button {
                    variant: ButtonVariant::Primary,
                    onclick: move |_| {
                        if let Some(d) = selected_date() {
                            let title = if event_title().is_empty() {
                                "Untitled Event".to_string()
                            } else {
                                event_title()
                            };
                            toast.success(
                                format!("\"{}\" scheduled for {}-{:02}-{:02}", title, d.year(), d.month() as u8, d.day()),
                                ToastOptions::new(),
                            );
                            event_sheet_open.set(false);
                        }
                    },
                    "Save Event"
                }
            }
        }

        // Delete Account confirmation dialog
        AlertDialogRoot {
            open: delete_dialog_open(),
            on_open_change: move |val: bool| delete_dialog_open.set(val),

            AlertDialogContent {
                AlertDialogTitle { "Delete Account" }
                AlertDialogDescription {
                    "This action cannot be undone. This will permanently delete your account and remove all associated data."
                }
                AlertDialogActions {
                    AlertDialogCancel { "Cancel" }
                    AlertDialogAction {
                        on_click: move |_| {
                            toast.error(
                                "Account deletion is not available in this demo.".to_string(),
                                ToastOptions::new(),
                            );
                            delete_dialog_open.set(false);
                        },
                        "Yes, Delete"
                    }
                }
            }
        }
    }
}
