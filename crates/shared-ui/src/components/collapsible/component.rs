use dioxus::prelude::*;
use dioxus_primitives::collapsible as prim;

#[component]
pub fn Collapsible(mut props: prim::CollapsibleProps) -> Element {
    props
        .attributes
        .push(Attribute::new("class", "collapsible", None, false));

    rsx! {
        document::Link { rel: "stylesheet", href: asset!("./style.css") }
        prim::Collapsible { ..props }
    }
}

#[component]
pub fn CollapsibleTrigger(mut props: prim::CollapsibleTriggerProps) -> Element {
    props
        .attributes
        .push(Attribute::new("class", "collapsible-trigger", None, false));

    rsx! {
        prim::CollapsibleTrigger { ..props }
    }
}

#[component]
pub fn CollapsibleContent(mut props: prim::CollapsibleContentProps) -> Element {
    props
        .attributes
        .push(Attribute::new("class", "collapsible-content", None, false));

    rsx! {
        prim::CollapsibleContent { ..props }
    }
}
