use dioxus::prelude::*;
use shared_types::{CalendarEntryResponse, JudicialOrderResponse, MotionResponse};
use shared_ui::components::{
    Badge, BadgeVariant, Button, ButtonVariant, Card, CardContent, CardDescription, CardHeader,
    CardTitle, DataTable, DataTableBody, DataTableCell, DataTableColumn, DataTableHeader,
    DataTableRow, PageHeader, PageTitle, Skeleton,
};

use crate::auth::use_auth;
use crate::routes::Route;
use crate::CourtContext;

/// Judge dashboard with three actionable work lists:
/// 1. Orders pending the judge's signature
/// 2. Upcoming hearings within the next 7 days
/// 3. Pending motions on the judge's assigned cases
#[component]
pub fn JudgeDashboard() -> Element {
    let ctx = use_context::<CourtContext>();
    let court = ctx.court_id.read().clone();
    let auth = use_auth();
    let judge_id = auth
        .current_user
        .read()
        .as_ref()
        .and_then(|u| u.linked_judge_id.clone());

    // If the user account is not linked to a judge record, show a message.
    let Some(judge_id) = judge_id else {
        return rsx! {
            document::Link { rel: "stylesheet", href: asset!("./judge.css") }
            PageHeader {
                PageTitle { "Judicial Dashboard" }
            }
            Card {
                CardContent {
                    div { class: "judge-empty-state",
                        p { class: "judge-empty-title",
                            "Account Not Linked"
                        }
                        p { class: "judge-empty-description",
                            "Your account is not linked to a judge record. Contact your administrator to link your account."
                        }
                    }
                }
            }
        };
    };

    // ── Section 1: Orders Pending Signature ──
    let court_orders = court.clone();
    let jid_orders = judge_id.clone();
    let pending_orders = use_resource(move || {
        let court = court_orders.clone();
        let jid = jid_orders.clone();
        async move {
            server::api::list_orders_by_judge(court, jid)
                .await
                .ok()
                .map(|orders| {
                    orders
                        .into_iter()
                        .filter(|o| o.status == "pending_signature")
                        .collect::<Vec<JudicialOrderResponse>>()
                })
        }
    });

    // ── Section 2: Upcoming Hearings (7 days) ──
    let court_cal = court.clone();
    let jid_cal = judge_id.clone();
    let upcoming_hearings = use_resource(move || {
        let court = court_cal.clone();
        let jid = jid_cal.clone();
        async move {
            let now = chrono::Utc::now();
            let week_later = now + chrono::Duration::days(7);
            let date_from = now.to_rfc3339();
            let date_to = week_later.to_rfc3339();

            server::api::search_calendar_events(
                court,
                Some(jid),
                None,
                None,
                None,
                Some(date_from),
                Some(date_to),
                Some(0),
                Some(50),
            )
            .await
            .ok()
            .map(|resp| resp.events)
        }
    });

    // ── Section 3: Pending Motions ──
    let court_motions = court.clone();
    let jid_motions = judge_id.clone();
    let pending_motions = use_resource(move || {
        let court = court_motions.clone();
        let jid = jid_motions.clone();
        async move {
            server::api::list_pending_motions_for_judge(court, jid)
                .await
                .ok()
        }
    });

    rsx! {
        document::Link { rel: "stylesheet", href: asset!("./judge.css") }
        PageHeader {
            PageTitle { "Judicial Dashboard" }
        }

        // ── Orders Pending Signature ──
        OrdersPendingSignature { data: pending_orders }

        // ── Upcoming Hearings ──
        UpcomingHearings { data: upcoming_hearings }

        // ── Pending Motions ──
        PendingMotions { data: pending_motions }
    }
}

// ── Section Components ──────────────────────────────────────────────

/// Orders awaiting the judge's signature.
#[component]
fn OrdersPendingSignature(
    data: Resource<Option<Vec<JudicialOrderResponse>>>,
) -> Element {
    rsx! {
        Card { class: "judge-section-card",
            CardHeader {
                CardTitle { "Orders Pending Signature" }
                match &*data.read() {
                    Some(Some(orders)) => rsx! {
                        CardDescription { "{orders.len()} order(s)" }
                    },
                    _ => rsx! {},
                }
            }
            CardContent {
                match &*data.read() {
                    Some(Some(orders)) if orders.is_empty() => rsx! {
                        div { class: "judge-empty-state",
                            p { class: "judge-empty-title", "No orders awaiting your signature" }
                            p { class: "judge-empty-description",
                                "Drafted orders will appear here when they are ready for review."
                            }
                        }
                    },
                    Some(Some(orders)) => rsx! {
                        DataTable {
                            DataTableHeader {
                                DataTableColumn { "Case" }
                                DataTableColumn { "Order Title" }
                                DataTableColumn { "Submitted" }
                                DataTableColumn { "Action" }
                            }
                            DataTableBody {
                                for order in orders.iter() {
                                    OrderRow {
                                        order: order.clone(),
                                        on_signed: move |_| data.restart(),
                                    }
                                }
                            }
                        }
                    },
                    Some(None) => rsx! {
                        p { class: "judge-error-text", "Failed to load orders." }
                    },
                    None => rsx! {
                        for _ in 0..3 {
                            Skeleton { style: "height: 2.5rem; width: 100%; margin-bottom: 0.5rem;" }
                        }
                    },
                }
            }
        }
    }
}

/// A single order row with inline signing action.
#[component]
fn OrderRow(order: JudicialOrderResponse, on_signed: EventHandler<()>) -> Element {
    let ctx = use_context::<CourtContext>();
    let auth = use_auth();

    let case_id = order.case_id.clone();
    let case_display = order
        .case_number
        .clone()
        .unwrap_or_else(|| truncate_id(&order.case_id));

    let submitted = format_date_short(&order.created_at);

    let mut signing = use_signal(|| false);
    let mut error_msg = use_signal(|| Option::<String>::None);

    let order_id = order.id.clone();

    let handle_sign = move |_: MouseEvent| {
        let court = ctx.court_id.read().clone();
        let oid = order_id.clone();
        let user = auth.current_user.read().clone();

        if let Some(user) = user {
            let signer_name = user.display_name.clone();
            spawn(async move {
                signing.set(true);
                error_msg.set(None);
                let result = server::api::sign_order_action(court, oid, signer_name).await;
                signing.set(false);
                match result {
                    Ok(_) => {
                        on_signed.call(());
                    }
                    Err(e) => {
                        tracing::error!("Failed to sign order: {}", e);
                        error_msg.set(Some(e.to_string()));
                    }
                }
            });
        }
    };

    rsx! {
        DataTableRow {
            DataTableCell {
                Link { to: Route::CaseDetail { id: case_id.clone(), tab: Some("docket".to_string()) },
                    span { class: "judge-link", "{case_display}" }
                }
            }
            DataTableCell { "{order.title}" }
            DataTableCell { "{submitted}" }
            DataTableCell {
                div { class: "judge-order-actions",
                    Button {
                        variant: ButtonVariant::Primary,
                        onclick: handle_sign,
                        disabled: *signing.read(),
                        if *signing.read() { "Signing..." } else { "Sign" }
                    }
                    Link { to: Route::CaseDetail { id: case_id, tab: Some("docket".to_string()) },
                        Button { variant: ButtonVariant::Secondary, "Review" }
                    }
                }
                if let Some(err) = error_msg.read().as_ref() {
                    p { class: "judge-error-text", "{err}" }
                }
            }
        }
    }
}

/// Upcoming hearings within the next 7 days.
#[component]
fn UpcomingHearings(
    data: Resource<Option<Vec<CalendarEntryResponse>>>,
) -> Element {
    rsx! {
        Card { class: "judge-section-card",
            CardHeader {
                CardTitle { "Upcoming Hearings" }
                CardDescription { "Next 7 days" }
            }
            CardContent {
                match &*data.read() {
                    Some(Some(events)) if events.is_empty() => rsx! {
                        div { class: "judge-empty-state",
                            p { class: "judge-empty-title", "No hearings scheduled this week" }
                            p { class: "judge-empty-description",
                                "Scheduled hearings and conferences will appear here."
                            }
                        }
                    },
                    Some(Some(events)) => rsx! {
                        DataTable {
                            DataTableHeader {
                                DataTableColumn { "Date / Time" }
                                DataTableColumn { "Case" }
                                DataTableColumn { "Type" }
                                DataTableColumn { "Courtroom" }
                            }
                            DataTableBody {
                                for event in events.iter() {
                                    HearingRow { event: event.clone() }
                                }
                            }
                        }
                    },
                    Some(None) => rsx! {
                        p { class: "judge-error-text", "Failed to load hearings." }
                    },
                    None => rsx! {
                        for _ in 0..3 {
                            Skeleton { style: "height: 2.5rem; width: 100%; margin-bottom: 0.5rem;" }
                        }
                    },
                }
            }
        }
    }
}

/// A single hearing row in the upcoming hearings table.
#[component]
fn HearingRow(event: CalendarEntryResponse) -> Element {
    let case_id = event.case_id.clone();
    let case_display = event
        .case_number
        .clone()
        .unwrap_or_else(|| truncate_id(&event.case_id));

    let datetime = format_date_short(&event.scheduled_date);

    // Convert raw event_type to a friendlier label
    let event_label = humanize_event_type(&event.event_type);

    rsx! {
        DataTableRow {
            DataTableCell { "{datetime}" }
            DataTableCell {
                Link { to: Route::CaseDetail { id: case_id, tab: Some("scheduling".to_string()) },
                    span { class: "judge-link", "{case_display}" }
                }
            }
            DataTableCell {
                Badge { variant: BadgeVariant::Secondary, "{event_label}" }
            }
            DataTableCell { "{event.courtroom}" }
        }
    }
}

/// Pending motions on the judge's assigned cases.
#[component]
fn PendingMotions(
    data: Resource<Option<Vec<MotionResponse>>>,
) -> Element {
    rsx! {
        Card { class: "judge-section-card",
            CardHeader {
                CardTitle { "Pending Motions" }
                match &*data.read() {
                    Some(Some(motions)) => rsx! {
                        CardDescription { "{motions.len()} pending" }
                    },
                    _ => rsx! {},
                }
            }
            CardContent {
                match &*data.read() {
                    Some(Some(motions)) if motions.is_empty() => rsx! {
                        div { class: "judge-empty-state",
                            p { class: "judge-empty-title", "No pending motions" }
                            p { class: "judge-empty-description",
                                "Motions filed on your cases that require a ruling will appear here."
                            }
                        }
                    },
                    Some(Some(motions)) => rsx! {
                        DataTable {
                            DataTableHeader {
                                DataTableColumn { "Case" }
                                DataTableColumn { "Motion Type" }
                                DataTableColumn { "Filed" }
                                DataTableColumn { "Filed By" }
                                DataTableColumn { "Action" }
                            }
                            DataTableBody {
                                for motion in motions.iter() {
                                    MotionRow {
                                        motion: motion.clone(),
                                        on_ruled: move |_| data.restart(),
                                    }
                                }
                            }
                        }
                    },
                    Some(None) => rsx! {
                        p { class: "judge-error-text", "Failed to load motions." }
                    },
                    None => rsx! {
                        for _ in 0..3 {
                            Skeleton { style: "height: 2.5rem; width: 100%; margin-bottom: 0.5rem;" }
                        }
                    },
                }
            }
        }
    }
}

/// A single motion row with inline ruling action.
#[component]
fn MotionRow(motion: MotionResponse, on_ruled: EventHandler<()>) -> Element {
    let ctx = use_context::<CourtContext>();
    let auth = use_auth();

    let case_id = motion.case_id.clone();
    let case_display = motion
        .case_number
        .clone()
        .unwrap_or_else(|| truncate_id(&motion.case_id));

    let filed_date = format_date_short(&motion.filed_date);

    let mut expanded = use_signal(|| false);
    let mut selected_disposition = use_signal(|| Option::<String>::None);
    let mut ruling_text = use_signal(|| String::new());
    let mut submitting = use_signal(|| false);
    let mut error_msg = use_signal(|| Option::<String>::None);

    // Clone values needed across closures
    let motion_id = motion.id.clone();
    let motion_type = motion.motion_type.clone();
    let motion_filed_by = motion.filed_by.clone();
    let motion_description = motion.description.clone();

    let handle_rule = move |_: MouseEvent| {
        let court = ctx.court_id.read().clone();
        let mid = motion_id.clone();
        let disposition = selected_disposition.read().clone();
        let text = ruling_text.read().clone();
        let user = auth.current_user.read().clone();

        if let (Some(disposition), Some(user)) = (disposition, user) {
            let judge_id = user.linked_judge_id.unwrap_or_default();
            let judge_name = user.display_name.clone();

            spawn(async move {
                submitting.set(true);
                error_msg.set(None);
                let result = server::api::rule_on_motion(
                    court,
                    mid,
                    disposition,
                    if text.is_empty() { None } else { Some(text) },
                    judge_id,
                    judge_name,
                )
                .await;
                submitting.set(false);
                match result {
                    Ok(_) => {
                        on_ruled.call(());
                    }
                    Err(e) => {
                        tracing::error!("Failed to rule on motion: {}", e);
                        error_msg.set(Some(e.to_string()));
                    }
                }
            });
        }
    };

    rsx! {
        DataTableRow {
            DataTableCell {
                Link { to: Route::CaseDetail { id: case_id.clone(), tab: None },
                    span { class: "judge-link", "{case_display}" }
                }
            }
            DataTableCell {
                Badge { variant: BadgeVariant::Outline, "{motion_type}" }
            }
            DataTableCell { "{filed_date}" }
            DataTableCell { "{motion_filed_by}" }
            DataTableCell {
                Button {
                    variant: if *expanded.read() { ButtonVariant::Secondary } else { ButtonVariant::Primary },
                    onclick: move |_| expanded.toggle(),
                    if *expanded.read() { "Cancel" } else { "Rule" }
                }
            }
        }
        if *expanded.read() {
            tr {
                td { colspan: "5",
                    div { class: "judge-ruling-panel",
                        p { class: "judge-ruling-description", "{motion_description}" }

                        div { class: "judge-ruling-dispositions",
                            for disp in shared_types::RULING_DISPOSITIONS.iter().filter(|d| **d != "Moot") {
                                {
                                    let d = disp.to_string();
                                    let is_selected = selected_disposition.read().as_deref() == Some(*disp);
                                    rsx! {
                                        Button {
                                            variant: if is_selected { ButtonVariant::Primary } else { ButtonVariant::Secondary },
                                            onclick: move |_| {
                                                selected_disposition.set(Some(d.clone()));
                                            },
                                            "{disp}"
                                        }
                                    }
                                }
                            }
                        }

                        if selected_disposition.read().is_some() {
                            textarea {
                                class: "input judge-ruling-text",
                                placeholder: "Ruling text (optional — a default will be generated)",
                                rows: 4,
                                value: "{ruling_text}",
                                oninput: move |evt: Event<FormData>| ruling_text.set(evt.value()),
                            }
                            div { class: "judge-ruling-submit",
                                Button {
                                    variant: ButtonVariant::Primary,
                                    onclick: handle_rule,
                                    disabled: *submitting.read(),
                                    if *submitting.read() { "Submitting..." } else { "Submit Ruling" }
                                }
                            }
                        }

                        if let Some(err) = error_msg.read().as_ref() {
                            p { class: "judge-error-text", "{err}" }
                        }
                    }
                }
            }
        }
    }
}

// ── Helper functions ────────────────────────────────────────────────

/// Truncate a UUID string to the first 8 characters for compact display.
fn truncate_id(id: &str) -> String {
    if id.len() > 8 {
        format!("{}...", &id[..8])
    } else {
        id.to_string()
    }
}

/// Format an RFC3339 date string into a shorter human-readable form.
/// Falls back to the raw string if parsing fails.
fn format_date_short(rfc3339: &str) -> String {
    chrono::DateTime::parse_from_rfc3339(rfc3339)
        .map(|dt| dt.format("%b %d, %H:%M").to_string())
        .unwrap_or_else(|_| rfc3339.to_string())
}

/// Convert snake_case event types into a friendlier display label.
fn humanize_event_type(event_type: &str) -> String {
    event_type
        .split('_')
        .map(|word| {
            let mut chars = word.chars();
            match chars.next() {
                Some(c) => c.to_uppercase().to_string() + chars.as_str(),
                None => String::new(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}
