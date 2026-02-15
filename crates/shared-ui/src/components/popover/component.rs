use dioxus::prelude::*;
use dioxus_primitives::popover as prim;

#[component]
pub fn PopoverRoot(mut props: prim::PopoverRootProps) -> Element {
    props
        .attributes
        .push(Attribute::new("class", "cyber-popover", None, false));

    rsx! {
        document::Link { rel: "stylesheet", href: asset!("./style.css") }
        prim::PopoverRoot { ..props }
    }
}

#[component]
pub fn PopoverTrigger(mut props: prim::PopoverTriggerProps) -> Element {
    props.attributes.push(Attribute::new(
        "class",
        "cyber-popover-trigger",
        None,
        false,
    ));

    rsx! {
        prim::PopoverTrigger { ..props }
    }
}

#[component]
pub fn PopoverContent(mut props: prim::PopoverContentProps) -> Element {
    if props.class.is_none() {
        props.class = Some("cyber-popover-content".to_string());
    }

    rsx! {
        prim::PopoverContent { ..props }
    }
}
