use dioxus::prelude::*;
use shared_ui::components::{
    Badge, BadgeVariant, Button, ButtonVariant, Card, CardContent, CardHeader,
    Collapsible, CollapsibleContent, CollapsibleTrigger,
    Form, FormSelect, Input, Separator,
    Sheet, SheetClose, SheetContent, SheetFooter, SheetHeader, SheetSide, SheetTitle,
    Skeleton,
};
use shared_ui::{use_toast, ToastOptions};

use crate::CourtContext;

/// Format an optional i32 value as a string, or "—" if None.
fn fmt_opt(v: &serde_json::Value) -> String {
    match v {
        serde_json::Value::Number(n) => n.to_string(),
        serde_json::Value::String(s) => s.clone(),
        _ => "—".to_string(),
    }
}

#[component]
pub fn SentencingTab(case_id: String) -> Element {
    let ctx = use_context::<CourtContext>();
    let toast = use_toast();

    let mut show_sheet = use_signal(|| false);

    // Core sentencing form signals
    let mut form_sentencing_date = use_signal(String::new);
    let mut form_total_offense_level = use_signal(String::new);
    let mut form_criminal_history = use_signal(String::new);
    let mut form_custody_months = use_signal(String::new);

    // Optional fields
    let mut form_guidelines_low = use_signal(String::new);
    let mut form_guidelines_high = use_signal(String::new);
    let mut form_probation_months = use_signal(String::new);
    let mut form_fine_amount = use_signal(String::new);
    let mut form_restitution = use_signal(String::new);
    let mut form_supervised_release = use_signal(String::new);
    let mut form_departure_type = use_signal(|| "None".to_string());
    let mut form_departure_reason = use_signal(String::new);

    let case_id_save = case_id.clone();

    // Fetch defendants for this case (needed for sentencing creation)
    let case_id_defendants = case_id.clone();
    let defendants_data = use_resource(move || {
        let court = ctx.court_id.read().clone();
        let cid = case_id_defendants.clone();
        async move {
            server::api::list_defendants(court, cid)
                .await
                .ok()
                .and_then(|json| serde_json::from_str::<Vec<serde_json::Value>>(&json).ok())
        }
    });

    // Fetch assigned judge
    let case_id_judge = case_id.clone();
    let judge_data = use_resource(move || {
        let court = ctx.court_id.read().clone();
        let cid = case_id_judge.clone();
        async move {
            server::api::list_case_assignments(court, cid)
                .await
                .ok()
                .and_then(|json| serde_json::from_str::<Vec<serde_json::Value>>(&json).ok())
                .and_then(|v| v.into_iter().next())
        }
    });

    let mut sentencing_data = use_resource(move || {
        let court = ctx.court_id.read().clone();
        let cid = case_id.clone();
        async move {
            server::api::list_sentencing_by_case(court, cid)
                .await
                .ok()
                .and_then(|json| serde_json::from_str::<Vec<serde_json::Value>>(&json).ok())
        }
    });

    let handle_save = move |_: FormEvent| {
        let court = ctx.court_id.read().clone();
        let cid = case_id_save.clone();
        let sentencing_date = form_sentencing_date.read().clone();
        let total_offense = form_total_offense_level.read().clone();
        let criminal_history = form_criminal_history.read().clone();
        let custody = form_custody_months.read().clone();
        let guidelines_low = form_guidelines_low.read().clone();
        let guidelines_high = form_guidelines_high.read().clone();
        let probation = form_probation_months.read().clone();
        let fine = form_fine_amount.read().clone();
        let restitution = form_restitution.read().clone();
        let supervised = form_supervised_release.read().clone();
        let departure_type = form_departure_type.read().clone();
        let departure_reason = form_departure_reason.read().clone();

        // Get defendant_id and judge_id from loaded data
        let defendant_id = defendants_data
            .read()
            .as_ref()
            .and_then(|d| d.as_ref())
            .and_then(|defs| defs.first())
            .and_then(|d| d["id"].as_str())
            .map(|s| s.to_string());
        let judge_id = judge_data
            .read()
            .as_ref()
            .and_then(|d| d.as_ref())
            .and_then(|a| a["judge_id"].as_str())
            .map(|s| s.to_string());

        spawn(async move {
            let Some(defendant_id) = defendant_id else {
                toast.error("No defendant found for this case.".to_string(), ToastOptions::new());
                return;
            };
            let Some(judge_id) = judge_id else {
                toast.error("No judge assigned to this case.".to_string(), ToastOptions::new());
                return;
            };

            let mut body = serde_json::json!({
                "case_id": cid,
                "defendant_id": defendant_id,
                "judge_id": judge_id,
            });

            if !sentencing_date.is_empty() {
                body["sentencing_date"] = serde_json::Value::String(format!("{sentencing_date}T00:00:00Z"));
            }
            if let Ok(v) = total_offense.parse::<i32>() {
                body["total_offense_level"] = serde_json::json!(v);
            }
            if !criminal_history.is_empty() {
                body["criminal_history_category"] = serde_json::Value::String(criminal_history);
            }
            if let Ok(v) = custody.parse::<i32>() {
                body["custody_months"] = serde_json::json!(v);
            }
            if let Ok(v) = guidelines_low.parse::<i32>() {
                body["guidelines_range_low_months"] = serde_json::json!(v);
            }
            if let Ok(v) = guidelines_high.parse::<i32>() {
                body["guidelines_range_high_months"] = serde_json::json!(v);
            }
            if let Ok(v) = probation.parse::<i32>() {
                body["probation_months"] = serde_json::json!(v);
            }
            if let Ok(v) = fine.parse::<f64>() {
                body["fine_amount"] = serde_json::json!(v);
            }
            if let Ok(v) = restitution.parse::<f64>() {
                body["restitution_amount"] = serde_json::json!(v);
            }
            if let Ok(v) = supervised.parse::<i32>() {
                body["supervised_release_months"] = serde_json::json!(v);
            }
            if departure_type != "None" {
                body["departure_type"] = serde_json::Value::String(departure_type);
                if !departure_reason.trim().is_empty() {
                    body["departure_reason"] = serde_json::Value::String(departure_reason.trim().to_string());
                }
            }

            match server::api::create_sentencing(court, body.to_string()).await {
                Ok(_) => {
                    toast.success("Sentencing record created.".to_string(), ToastOptions::new());
                    show_sheet.set(false);
                    form_sentencing_date.set(String::new());
                    form_total_offense_level.set(String::new());
                    form_criminal_history.set(String::new());
                    form_custody_months.set(String::new());
                    form_guidelines_low.set(String::new());
                    form_guidelines_high.set(String::new());
                    form_probation_months.set(String::new());
                    form_fine_amount.set(String::new());
                    form_restitution.set(String::new());
                    form_supervised_release.set(String::new());
                    form_departure_type.set("None".to_string());
                    form_departure_reason.set(String::new());
                    sentencing_data.restart();
                }
                Err(e) => toast.error(format!("Error: {e}"), ToastOptions::new()),
            }
        });
    };

    rsx! {
        div {
            style: "display: flex; justify-content: space-between; align-items: center; margin-bottom: var(--space-md);",
            h3 { "Sentencing" }
            Button {
                variant: ButtonVariant::Primary,
                onclick: move |_| show_sheet.set(true),
                "Record Sentence"
            }
        }

        match &*sentencing_data.read() {
            Some(Some(records)) if !records.is_empty() => rsx! {
                for record in records.iter() {
                    Card {
                        CardHeader {
                            div { style: "display: flex; justify-content: space-between; align-items: center; width: 100%;",
                                span { "Sentencing Record" }
                                Badge { variant: BadgeVariant::Primary,
                                    {record["sentencing_date"].as_str().map(|d| if d.len() >= 10 { &d[..10] } else { d }).unwrap_or("Pending")}
                                }
                            }
                        }
                        CardContent {
                            div { style: "display: grid; grid-template-columns: repeat(auto-fit, minmax(180px, 1fr)); gap: var(--space-md);",
                                div {
                                    span { style: "font-size: var(--font-size-sm); color: var(--color-on-surface-muted); font-weight: 500;", "Offense Level" }
                                    p { {fmt_opt(&record["total_offense_level"])} }
                                }
                                div {
                                    span { style: "font-size: var(--font-size-sm); color: var(--color-on-surface-muted); font-weight: 500;", "Criminal History" }
                                    p { {record["criminal_history_category"].as_str().unwrap_or("—")} }
                                }
                                div {
                                    span { style: "font-size: var(--font-size-sm); color: var(--color-on-surface-muted); font-weight: 500;", "Guidelines Range" }
                                    p {
                                        {format!(
                                            "{} — {} months",
                                            fmt_opt(&record["guidelines_range_low_months"]),
                                            fmt_opt(&record["guidelines_range_high_months"]),
                                        )}
                                    }
                                }
                                div {
                                    span { style: "font-size: var(--font-size-sm); color: var(--color-on-surface-muted); font-weight: 500;", "Custody" }
                                    p { {format!("{} months", fmt_opt(&record["custody_months"]))} }
                                }
                                div {
                                    span { style: "font-size: var(--font-size-sm); color: var(--color-on-surface-muted); font-weight: 500;", "Probation" }
                                    p { {format!("{} months", fmt_opt(&record["probation_months"]))} }
                                }
                                div {
                                    span { style: "font-size: var(--font-size-sm); color: var(--color-on-surface-muted); font-weight: 500;", "Fine" }
                                    p { {fmt_opt(&record["fine_amount"])} }
                                }
                            }

                            if record["departure_type"].as_str().is_some() {
                                div { style: "margin-top: var(--space-md); padding-top: var(--space-md); border-top: 1px solid var(--color-border);",
                                    div { style: "display: flex; gap: var(--space-md); align-items: center;",
                                        Badge { variant: BadgeVariant::Destructive,
                                            {format!("Departure: {}", record["departure_type"].as_str().unwrap_or(""))}
                                        }
                                        if let Some(reason) = record["departure_reason"].as_str() {
                                            span { style: "color: var(--color-on-surface-muted);", "{reason}" }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            },
            Some(Some(_)) => rsx! {
                Card {
                    CardContent {
                        p { class: "empty-state", "No sentencing records for this case." }
                    }
                }
            },
            Some(None) => rsx! {
                p { class: "error-state", "Failed to load sentencing data." }
            },
            None => rsx! {
                Skeleton { style: "width: 100%; height: 200px" }
            },
        }

        // Record Sentence Sheet
        Sheet {
            open: show_sheet(),
            on_close: move |_| show_sheet.set(false),
            side: SheetSide::Right,
            SheetContent {
                SheetHeader {
                    SheetTitle { "Record Sentence" }
                    SheetClose { on_close: move |_| show_sheet.set(false) }
                }
                Form {
                    onsubmit: handle_save,
                    div { class: "sheet-form",
                        // Show assigned judge (read-only context)
                        {
                            match &*judge_data.read() {
                                Some(Some(assignment)) => {
                                    let judge_display = assignment["judge_name"].as_str()
                                        .unwrap_or(assignment["judge_id"].as_str().unwrap_or("—"));
                                    rsx! {
                                        div { class: "form-field",
                                            label { class: "form-label", "Sentencing Judge" }
                                            p { class: "form-static-value", "{judge_display}" }
                                        }
                                    }
                                },
                                _ => rsx! {},
                            }
                        }

                        // Show defendant (read-only context)
                        {
                            match &*defendants_data.read() {
                                Some(Some(defs)) if !defs.is_empty() => {
                                    let name = defs[0]["name"].as_str()
                                        .or_else(|| defs[0]["full_name"].as_str())
                                        .unwrap_or("—");
                                    rsx! {
                                        div { class: "form-field",
                                            label { class: "form-label", "Defendant" }
                                            p { class: "form-static-value", "{name}" }
                                        }
                                    }
                                },
                                _ => rsx! {},
                            }
                        }

                        Input {
                            label: "Sentencing Date",
                            input_type: "date",
                            value: form_sentencing_date(),
                            on_input: move |e: FormEvent| form_sentencing_date.set(e.value()),
                        }
                        Input {
                            label: "Total Offense Level",
                            input_type: "number",
                            value: form_total_offense_level(),
                            on_input: move |e: FormEvent| form_total_offense_level.set(e.value()),
                            placeholder: "e.g. 24",
                        }
                        FormSelect {
                            label: "Criminal History Category",
                            value: "{form_criminal_history}",
                            onchange: move |e: Event<FormData>| form_criminal_history.set(e.value()),
                            option { value: "", "Select category" }
                            option { value: "I", "I" }
                            option { value: "II", "II" }
                            option { value: "III", "III" }
                            option { value: "IV", "IV" }
                            option { value: "V", "V" }
                            option { value: "VI", "VI" }
                        }
                        Input {
                            label: "Custody (months)",
                            input_type: "number",
                            value: form_custody_months(),
                            on_input: move |e: FormEvent| form_custody_months.set(e.value()),
                            placeholder: "e.g. 60",
                        }

                        // Additional fields in a collapsible section
                        Collapsible {
                            CollapsibleTrigger {
                                div { style: "display: flex; align-items: center; gap: var(--space-sm); cursor: pointer; padding: var(--space-sm) 0; color: var(--color-primary); font-weight: 500;",
                                    "Additional Sentencing Details"
                                }
                            }
                            CollapsibleContent {
                                div { style: "display: flex; flex-direction: column; gap: var(--space-md); padding-top: var(--space-sm);",
                                    div { style: "display: grid; grid-template-columns: 1fr 1fr; gap: var(--space-sm);",
                                        Input {
                                            label: "Guidelines Low (months)",
                                            input_type: "number",
                                            value: form_guidelines_low(),
                                            on_input: move |e: FormEvent| form_guidelines_low.set(e.value()),
                                        }
                                        Input {
                                            label: "Guidelines High (months)",
                                            input_type: "number",
                                            value: form_guidelines_high(),
                                            on_input: move |e: FormEvent| form_guidelines_high.set(e.value()),
                                        }
                                    }
                                    Input {
                                        label: "Probation (months)",
                                        input_type: "number",
                                        value: form_probation_months(),
                                        on_input: move |e: FormEvent| form_probation_months.set(e.value()),
                                    }
                                    div { style: "display: grid; grid-template-columns: 1fr 1fr; gap: var(--space-sm);",
                                        Input {
                                            label: "Fine Amount ($)",
                                            input_type: "number",
                                            value: form_fine_amount(),
                                            on_input: move |e: FormEvent| form_fine_amount.set(e.value()),
                                            placeholder: "0.00",
                                        }
                                        Input {
                                            label: "Restitution ($)",
                                            input_type: "number",
                                            value: form_restitution(),
                                            on_input: move |e: FormEvent| form_restitution.set(e.value()),
                                            placeholder: "0.00",
                                        }
                                    }
                                    Input {
                                        label: "Supervised Release (months)",
                                        input_type: "number",
                                        value: form_supervised_release(),
                                        on_input: move |e: FormEvent| form_supervised_release.set(e.value()),
                                    }
                                    FormSelect {
                                        label: "Departure Type",
                                        value: "{form_departure_type}",
                                        onchange: move |e: Event<FormData>| form_departure_type.set(e.value()),
                                        option { value: "None", "None" }
                                        option { value: "Upward", "Upward" }
                                        option { value: "Downward", "Downward" }
                                    }
                                    if &*form_departure_type.read() != "None" {
                                        Input {
                                            label: "Departure Reason",
                                            value: form_departure_reason(),
                                            on_input: move |e: FormEvent| form_departure_reason.set(e.value()),
                                            placeholder: "Reason for departure from guidelines",
                                        }
                                    }
                                }
                            }
                        }
                    }
                    Separator {}
                    SheetFooter {
                        div { class: "sheet-footer-actions",
                            SheetClose { on_close: move |_| show_sheet.set(false) }
                            Button { variant: ButtonVariant::Primary, "Record Sentence" }
                        }
                    }
                }
            }
        }
    }
}
