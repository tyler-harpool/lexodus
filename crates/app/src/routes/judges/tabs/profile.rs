use dioxus::prelude::*;
use shared_types::JudgeResponse;
use shared_ui::components::{
    Badge, BadgeVariant, Card, CardContent, CardHeader, CardTitle, DetailGrid, DetailItem,
    DetailList,
};

#[component]
pub fn JudgeProfileTab(judge: JudgeResponse) -> Element {
    rsx! {
        DetailGrid {
            Card {
                CardHeader { CardTitle { "Biographical Information" } }
                CardContent {
                    DetailList {
                        DetailItem { label: "Name", value: judge.name.clone() }
                        DetailItem { label: "Title", value: judge.title.clone() }
                        DetailItem { label: "District", value: judge.district.clone() }
                        DetailItem { label: "Status",
                            Badge {
                                variant: match judge.status.as_str() {
                                    "Active" => BadgeVariant::Primary,
                                    "Senior" => BadgeVariant::Secondary,
                                    "Retired" => BadgeVariant::Outline,
                                    _ => BadgeVariant::Secondary,
                                },
                                {judge.status.as_str()}
                            }
                        }
                        if let Some(ref d) = judge.appointed_date {
                            DetailItem { label: "Appointed", value: d.chars().take(10).collect::<String>() }
                        }
                    }
                }
            }

            Card {
                CardHeader { CardTitle { "Chambers" } }
                CardContent {
                    DetailList {
                        if let Some(ref cr) = judge.courtroom {
                            DetailItem { label: "Courtroom", value: cr.clone() }
                        }
                        DetailItem {
                            label: "Specializations",
                            value: if judge.specializations.is_empty() {
                                "None".to_string()
                            } else {
                                judge.specializations.join(", ")
                            }
                        }
                    }
                }
            }

            Card {
                CardHeader { CardTitle { "Caseload Capacity" } }
                CardContent {
                    DetailList {
                        DetailItem {
                            label: "Current / Max",
                            value: format!("{} / {}", judge.current_caseload, judge.max_caseload)
                        }
                    }
                }
            }
        }
    }
}
