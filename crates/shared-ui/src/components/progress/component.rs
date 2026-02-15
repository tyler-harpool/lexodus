use dioxus::prelude::*;
use dioxus_primitives::progress as prim;

#[component]
pub fn Progress(mut props: prim::ProgressProps) -> Element {
    props
        .attributes
        .push(Attribute::new("class", "progress", None, false));

    rsx! {
        document::Link { rel: "stylesheet", href: asset!("./style.css") }
        prim::Progress { ..props }
    }
}

#[component]
pub fn ProgressIndicator(mut props: prim::ProgressIndicatorProps) -> Element {
    props
        .attributes
        .push(Attribute::new("class", "progress-indicator", None, false));

    rsx! {
        prim::ProgressIndicator { ..props }
    }
}
