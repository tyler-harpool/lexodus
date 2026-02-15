use dioxus::prelude::*;
use dioxus_primitives::hover_card as prim;

#[component]
pub fn HoverCard(mut props: prim::HoverCardProps) -> Element {
    props
        .attributes
        .push(Attribute::new("class", "cyber-hover-card", None, false));

    rsx! {
        document::Link { rel: "stylesheet", href: asset!("./style.css") }
        prim::HoverCard { ..props }
    }
}

#[component]
pub fn HoverCardTrigger(mut props: prim::HoverCardTriggerProps) -> Element {
    props.attributes.push(Attribute::new(
        "class",
        "cyber-hover-card-trigger",
        None,
        false,
    ));

    rsx! {
        prim::HoverCardTrigger { ..props }
    }
}

#[component]
pub fn HoverCardContent(mut props: prim::HoverCardContentProps) -> Element {
    props.attributes.push(Attribute::new(
        "class",
        "cyber-hover-card-content",
        None,
        false,
    ));

    rsx! {
        prim::HoverCardContent { ..props }
    }
}
