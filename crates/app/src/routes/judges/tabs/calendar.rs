use dioxus::prelude::*;
use shared_types::CalendarEntryResponse;
use shared_ui::components::{
    Badge, BadgeVariant, Card, CardContent, CardHeader, CardTitle, DataTable, DataTableBody,
    DataTableCell, DataTableColumn, DataTableHeader, DataTableRow, Skeleton,
};
use crate::CourtContext;

#[component]
pub fn JudgeCalendarTab(judge_id: String) -> Element {
    let ctx = use_context::<CourtContext>();

    let data = use_resource(move || {
        let court = ctx.court_id.read().clone();
        let jid = judge_id.clone();
        async move {
            server::api::search_calendar_events(
                court, Some(jid), None, None, None, None, None, Some(0), Some(50),
            )
            .await
            .ok()
            .map(|resp| resp.events)
        }
    });

    rsx! {
        Card {
            CardHeader { CardTitle { "Scheduled Events" } }
            CardContent {
                match &*data.read() {
                    Some(Some(rows)) if !rows.is_empty() => rsx! {
                        DataTable {
                            DataTableHeader {
                                DataTableColumn { "Date" }
                                DataTableColumn { "Event Type" }
                                DataTableColumn { "Case" }
                                DataTableColumn { "Status" }
                            }
                            DataTableBody {
                                for row in rows.iter() {
                                    CalendarEventRow { event: row.clone() }
                                }
                            }
                        }
                    },
                    Some(_) => rsx! { p { class: "text-muted", "No scheduled events." } },
                    None => rsx! { Skeleton { width: "100%", height: "200px" } },
                }
            }
        }
    }
}

#[component]
fn CalendarEventRow(event: CalendarEntryResponse) -> Element {
    let date_display = event.scheduled_date.chars().take(10).collect::<String>();
    let case_display = event.case_id.chars().take(8).collect::<String>();

    rsx! {
        DataTableRow {
            DataTableCell { "{date_display}" }
            DataTableCell {
                Badge { variant: BadgeVariant::Secondary,
                    {event.event_type.as_str()}
                }
            }
            DataTableCell { "{case_display}" }
            DataTableCell {
                Badge { variant: BadgeVariant::Primary,
                    {event.status.as_str()}
                }
            }
        }
    }
}
