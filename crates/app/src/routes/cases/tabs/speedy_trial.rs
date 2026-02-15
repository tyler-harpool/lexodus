use dioxus::prelude::*;
use shared_ui::components::{
    Badge, BadgeVariant, Button, ButtonVariant, Card, CardContent,
    DataTable, DataTableBody, DataTableCell, DataTableColumn, DataTableHeader, DataTableRow,
    Form, Input, Separator,
    Sheet, SheetClose, SheetContent, SheetFooter, SheetHeader, SheetSide, SheetTitle,
    Skeleton,
};
use shared_ui::{use_toast, ToastOptions};

use crate::CourtContext;

const SPEEDY_TRIAL_LIMIT_DAYS: i64 = 70;

#[component]
pub fn SpeedyTrialTab(case_id: String) -> Element {
    let ctx = use_context::<CourtContext>();
    let toast = use_toast();

    let mut show_sheet = use_signal(|| false);
    let mut form_reason = use_signal(String::new);
    let mut form_start_date = use_signal(String::new);
    let mut form_end_date = use_signal(String::new);
    let mut form_statutory_ref = use_signal(String::new);
    let mut form_days = use_signal(String::new);

    let case_id_delays = case_id.clone();
    let case_id_save = case_id.clone();

    let clock_data = use_resource(move || {
        let court = ctx.court_id.read().clone();
        let cid = case_id.clone();
        async move {
            server::api::get_speedy_trial(court, cid)
                .await
                .ok()
                .and_then(|json| serde_json::from_str::<serde_json::Value>(&json).ok())
        }
    });

    let mut delays_data = use_resource(move || {
        let court = ctx.court_id.read().clone();
        let cid = case_id_delays.clone();
        async move {
            server::api::list_speedy_trial_delays(court, cid)
                .await
                .ok()
                .and_then(|json| serde_json::from_str::<Vec<serde_json::Value>>(&json).ok())
        }
    });

    let handle_save = move |_: FormEvent| {
        let court = ctx.court_id.read().clone();
        let cid = case_id_save.clone();
        let reason = form_reason.read().clone();
        let start = form_start_date.read().clone();
        let end = form_end_date.read().clone();
        let stat_ref = form_statutory_ref.read().clone();
        let days: i64 = form_days.read().parse().unwrap_or(0);

        spawn(async move {
            if reason.trim().is_empty() || start.is_empty() {
                toast.error("Reason and start date required.".to_string(), ToastOptions::new());
                return;
            }
            let body = serde_json::json!({
                "start_date": format!("{start}T00:00:00Z"),
                "end_date": if end.is_empty() { None } else { Some(format!("{end}T00:00:00Z")) },
                "reason": reason.trim(),
                "statutory_reference": stat_ref.trim(),
                "days_excluded": days,
            });
            match server::api::create_speedy_trial_delay(court, cid, body.to_string()).await {
                Ok(_) => {
                    toast.success("Exclusion added.".to_string(), ToastOptions::new());
                    show_sheet.set(false);
                    form_reason.set(String::new());
                    form_start_date.set(String::new());
                    form_end_date.set(String::new());
                    form_days.set(String::new());
                    delays_data.restart();
                }
                Err(e) => toast.error(format!("Error: {e}"), ToastOptions::new()),
            }
        });
    };

    rsx! {
        div {
            style: "display: flex; justify-content: space-between; align-items: center; margin-bottom: var(--space-md);",
            h3 { "Speedy Trial Clock" }
            Button {
                variant: ButtonVariant::Primary,
                onclick: move |_| show_sheet.set(true),
                "Add Exclusion"
            }
        }

        // Clock status card
        match &*clock_data.read() {
            Some(Some(clock)) => {
                let days_elapsed = clock["days_elapsed"].as_i64().unwrap_or(0);
                let days_remaining = clock["days_remaining"].as_i64().unwrap_or(SPEEDY_TRIAL_LIMIT_DAYS);
                let is_tolled = clock["is_tolled"].as_bool().unwrap_or(false);
                let waived = clock["waived"].as_bool().unwrap_or(false);
                let progress_pct = ((days_elapsed as f64 / SPEEDY_TRIAL_LIMIT_DAYS as f64) * 100.0).min(100.0);

                let status_text = if waived {
                    "Waived"
                } else if is_tolled {
                    "Tolled"
                } else if days_remaining <= 0 {
                    "Expired"
                } else {
                    "Running"
                };
                let status_variant = match status_text {
                    "Expired" => BadgeVariant::Destructive,
                    "Tolled" | "Waived" => BadgeVariant::Secondary,
                    _ => BadgeVariant::Primary,
                };

                rsx! {
                    Card {
                        CardContent {
                            div { style: "display: flex; justify-content: space-between; align-items: center; margin-bottom: var(--space-md);",
                                div { style: "display: flex; gap: var(--space-md); align-items: center;",
                                    Badge { variant: status_variant, {status_text} }
                                    span { style: "font-size: var(--font-size-lg); font-weight: 600;",
                                        "{days_elapsed} / {SPEEDY_TRIAL_LIMIT_DAYS} days"
                                    }
                                }
                                span { style: "color: var(--color-on-surface-muted);",
                                    "{days_remaining} days remaining"
                                }
                            }

                            // Progress bar
                            div { style: "width: 100%; height: 8px; background: var(--color-surface); border-radius: 4px; overflow: hidden;",
                                div {
                                    style: format!(
                                        "width: {progress_pct}%; height: 100%; border-radius: 4px; transition: width 0.3s; background: {};",
                                        if progress_pct > 85.0 { "var(--color-destructive)" }
                                        else if progress_pct > 60.0 { "var(--color-warning, orange)" }
                                        else { "var(--color-primary)" }
                                    ),
                                }
                            }

                            // Key dates
                            div { style: "display: grid; grid-template-columns: repeat(auto-fit, minmax(180px, 1fr)); gap: var(--space-md); margin-top: var(--space-md);",
                                if let Some(date) = clock["arrest_date"].as_str() {
                                    div {
                                        span { style: "font-size: var(--font-size-sm); color: var(--color-on-surface-muted);", "Arrest Date" }
                                        p { {if date.len() >= 10 { &date[..10] } else { date }} }
                                    }
                                }
                                if let Some(date) = clock["indictment_date"].as_str() {
                                    div {
                                        span { style: "font-size: var(--font-size-sm); color: var(--color-on-surface-muted);", "Indictment Date" }
                                        p { {if date.len() >= 10 { &date[..10] } else { date }} }
                                    }
                                }
                                if let Some(date) = clock["trial_start_deadline"].as_str() {
                                    div {
                                        span { style: "font-size: var(--font-size-sm); color: var(--color-on-surface-muted);", "Trial Deadline" }
                                        p { {if date.len() >= 10 { &date[..10] } else { date }} }
                                    }
                                }
                            }
                        }
                    }
                }
            },
            Some(None) => rsx! {
                Card {
                    CardContent {
                        p { class: "empty-state", "No speedy trial clock initialized for this case." }
                    }
                }
            },
            None => rsx! {
                Skeleton { style: "width: 100%; height: 120px" }
            },
        }

        // Delay periods table
        match &*delays_data.read() {
            Some(Some(delays)) if !delays.is_empty() => rsx! {
                h4 { style: "margin-top: var(--space-lg); margin-bottom: var(--space-sm);", "Excludable Delay Periods" }
                DataTable {
                    DataTableHeader {
                        DataTableColumn { "Reason" }
                        DataTableColumn { "Start" }
                        DataTableColumn { "End" }
                        DataTableColumn { "Days" }
                        DataTableColumn { "Statutory Ref" }
                    }
                    DataTableBody {
                        for delay in delays.iter() {
                            DataTableRow {
                                DataTableCell { {delay["reason"].as_str().unwrap_or("—").replace('_', " ")} }
                                DataTableCell {
                                    {delay["start_date"].as_str().map(|d| if d.len() >= 10 { &d[..10] } else { d }).unwrap_or("—")}
                                }
                                DataTableCell {
                                    {delay["end_date"].as_str().map(|d| if d.len() >= 10 { &d[..10] } else { d }).unwrap_or("Ongoing")}
                                }
                                DataTableCell {
                                    {delay["days_excluded"].as_i64().map(|d| d.to_string()).unwrap_or_else(|| "—".to_string())}
                                }
                                DataTableCell {
                                    {delay["statutory_reference"].as_str().unwrap_or("—")}
                                }
                            }
                        }
                    }
                }
            },
            Some(Some(_)) => rsx! {
                p { style: "margin-top: var(--space-md); color: var(--color-on-surface-muted);",
                    "No excludable delay periods recorded."
                }
            },
            _ => rsx! {},
        }

        // Add Exclusion Sheet
        Sheet {
            open: show_sheet(),
            on_close: move |_| show_sheet.set(false),
            side: SheetSide::Right,
            SheetContent {
                SheetHeader {
                    SheetTitle { "Add Exclusion Period" }
                    SheetClose { on_close: move |_| show_sheet.set(false) }
                }
                Form {
                    onsubmit: handle_save,
                    div { class: "sheet-form",
                        Input {
                            label: "Reason",
                            value: form_reason(),
                            on_input: move |e: FormEvent| form_reason.set(e.value()),
                            placeholder: "e.g., Continuance granted",
                        }
                        Input {
                            label: "Start Date",
                            input_type: "date",
                            value: form_start_date(),
                            on_input: move |e: FormEvent| form_start_date.set(e.value()),
                        }
                        Input {
                            label: "End Date (optional)",
                            input_type: "date",
                            value: form_end_date(),
                            on_input: move |e: FormEvent| form_end_date.set(e.value()),
                        }
                        Input {
                            label: "Days Excluded",
                            input_type: "number",
                            value: form_days(),
                            on_input: move |e: FormEvent| form_days.set(e.value()),
                            placeholder: "0",
                        }
                        Input {
                            label: "Statutory Reference",
                            value: form_statutory_ref(),
                            on_input: move |e: FormEvent| form_statutory_ref.set(e.value()),
                            placeholder: "e.g., 18 U.S.C. § 3161(h)(7)(A)",
                        }
                    }
                    Separator {}
                    SheetFooter {
                        div { class: "sheet-footer-actions",
                            SheetClose { on_close: move |_| show_sheet.set(false) }
                            Button { variant: ButtonVariant::Primary, "Add Exclusion" }
                        }
                    }
                }
            }
        }
    }
}
