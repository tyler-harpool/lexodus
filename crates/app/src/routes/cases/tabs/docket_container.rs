use dioxus::prelude::*;
use shared_ui::components::{TabContent, TabList, TabTrigger, Tabs};

use super::docket::DocketTab;
use super::evidence::EvidenceTab;
use super::orders::OrdersTab;

#[component]
pub fn DocketContainerTab(case_id: String) -> Element {
    rsx! {
        Tabs { default_value: "entries", horizontal: true,
            TabList {
                TabTrigger { value: "entries", index: 0usize, "Full Docket" }
                TabTrigger { value: "orders", index: 1usize, "Orders" }
                TabTrigger { value: "evidence", index: 2usize, "Evidence" }
            }
            TabContent { value: "entries", index: 0usize,
                DocketTab { case_id: case_id.clone() }
            }
            TabContent { value: "orders", index: 1usize,
                OrdersTab { case_id: case_id.clone() }
            }
            TabContent { value: "evidence", index: 2usize,
                EvidenceTab { case_id: case_id.clone() }
            }
        }
    }
}
