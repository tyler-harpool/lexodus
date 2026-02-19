use dioxus::prelude::*;
use shared_ui::components::{TabContent, TabList, TabTrigger, Tabs};

use super::calendar_tab::CalendarTab;
use super::deadlines::DeadlinesTab;
use super::speedy_trial::SpeedyTrialTab;

#[component]
pub fn SchedulingTab(case_id: String) -> Element {
    rsx! {
        Tabs { default_value: "calendar", horizontal: true,
            TabList {
                TabTrigger { value: "calendar", index: 0usize, "Calendar" }
                TabTrigger { value: "deadlines", index: 1usize, "Deadlines" }
                TabTrigger { value: "speedy-trial", index: 2usize, "Speedy Trial" }
            }
            TabContent { value: "calendar", index: 0usize,
                CalendarTab { case_id: case_id.clone() }
            }
            TabContent { value: "deadlines", index: 1usize,
                DeadlinesTab { case_id: case_id.clone() }
            }
            TabContent { value: "speedy-trial", index: 2usize,
                SpeedyTrialTab { case_id: case_id.clone() }
            }
        }
    }
}
