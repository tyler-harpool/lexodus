use dioxus::prelude::*;
use shared_ui::components::{Badge, BadgeVariant, Card, CardContent, CardHeader, Skeleton};

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

    let case_id_cond = case_id.clone();

    let sentencing_data = use_resource(move || {
        let court = ctx.court_id.read().clone();
        let cid = case_id.clone();
        async move {
            server::api::list_sentencing_by_case(court, cid)
                .await
                .ok()
                .and_then(|json| serde_json::from_str::<Vec<serde_json::Value>>(&json).ok())
        }
    });

    // Conditions loading will be expanded in a future iteration
    let _ = case_id_cond;

    rsx! {
        div {
            style: "display: flex; justify-content: space-between; align-items: center; margin-bottom: var(--space-md);",
            h3 { "Sentencing" }
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
                                // Guidelines calculation
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

                            // Departure/variance info if present
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
    }
}
