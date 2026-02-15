use dioxus::prelude::*;
use shared_ui::components::{Card, CardContent, CardHeader, Skeleton};

use crate::CourtContext;

#[component]
pub fn PartiesTab(case_id: String) -> Element {
    let ctx = use_context::<CourtContext>();

    let parties = use_resource(move || {
        let court = ctx.court_id.read().clone();
        let cid = case_id.clone();
        async move { server::api::list_case_parties(court, cid).await.ok() }
    });

    rsx! {
        match &*parties.read() {
            Some(Some(json)) => rsx! {
                Card {
                    CardHeader { "Case Parties" }
                    CardContent {
                        p { "Party data loaded. Full table coming soon." }
                        pre { class: "debug-json", "{json}" }
                    }
                }
            },
            Some(None) => rsx! {
                Card {
                    CardContent { p { "No parties found for this case." } }
                }
            },
            None => rsx! {
                Skeleton { width: "100%", height: "200px" }
            },
        }
    }
}
