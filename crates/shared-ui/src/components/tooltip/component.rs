use dioxus::prelude::*;
use dioxus_primitives::tooltip as prim;

pub use dioxus_primitives::{ContentAlign, ContentSide};

#[component]
pub fn Tooltip(mut props: prim::TooltipProps) -> Element {
    props
        .attributes
        .push(Attribute::new("class", "cyber-tooltip", None, false));

    rsx! {
        document::Link { rel: "stylesheet", href: asset!("./style.css") }
        prim::Tooltip { ..props }
    }
}

#[component]
pub fn TooltipTrigger(mut props: prim::TooltipTriggerProps) -> Element {
    props.attributes.push(Attribute::new(
        "class",
        "cyber-tooltip-trigger",
        None,
        false,
    ));

    rsx! {
        prim::TooltipTrigger { ..props }
    }
}

#[component]
pub fn TooltipContent(mut props: prim::TooltipContentProps) -> Element {
    props.attributes.push(Attribute::new(
        "class",
        "cyber-tooltip-content",
        None,
        false,
    ));

    rsx! {
        prim::TooltipContent { ..props }
    }
}
