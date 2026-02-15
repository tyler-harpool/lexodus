use dioxus::prelude::*;
use shared_ui::components::{Badge, BadgeVariant, Card, CardContent, CardHeader, Separator};

#[component]
pub fn OverviewTab(
    case_id: String,
    title: String,
    case_number: String,
    status: String,
    crime_type: String,
    district: String,
    priority: String,
    description: String,
) -> Element {
    let status_variant = match status.as_str() {
        "filed" | "arraigned" => BadgeVariant::Primary,
        "dismissed" | "sentenced" => BadgeVariant::Secondary,
        _ => BadgeVariant::Secondary,
    };
    let priority_variant = match priority.as_str() {
        "high" | "critical" => BadgeVariant::Destructive,
        "medium" => BadgeVariant::Primary,
        _ => BadgeVariant::Secondary,
    };
    let display_status = status.replace('_', " ");
    let display_type = crime_type.replace('_', " ");

    rsx! {
        document::Link { rel: "stylesheet", href: asset!("./overview.css") }
        div { class: "case-overview",
            Card {
                CardHeader { "Case Information" }
                CardContent {
                    div { class: "overview-grid",
                        div { class: "overview-item",
                            span { class: "overview-label", "Case Number" }
                            span { class: "overview-value", "{case_number}" }
                        }
                        div { class: "overview-item",
                            span { class: "overview-label", "Status" }
                            Badge { variant: status_variant, "{display_status}" }
                        }
                        div { class: "overview-item",
                            span { class: "overview-label", "Crime Type" }
                            span { class: "overview-value", "{display_type}" }
                        }
                        div { class: "overview-item",
                            span { class: "overview-label", "District" }
                            span { class: "overview-value", "{district}" }
                        }
                        div { class: "overview-item",
                            span { class: "overview-label", "Priority" }
                            Badge { variant: priority_variant, "{priority}" }
                        }
                    }

                    if !description.is_empty() {
                        Separator {}
                        div { class: "overview-description",
                            h4 { "Description" }
                            p { "{description}" }
                        }
                    }
                }
            }
        }
    }
}
