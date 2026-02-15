use dioxus::prelude::*;
use dioxus_primitives::tabs as prim;

#[component]
pub fn Tabs(mut props: prim::TabsProps) -> Element {
    props
        .attributes
        .push(Attribute::new("class", "tabs", None, false));

    rsx! {
        document::Link { rel: "stylesheet", href: asset!("./style.css") }
        prim::Tabs { ..props }
    }
}

#[component]
pub fn TabList(mut props: prim::TabListProps) -> Element {
    props
        .attributes
        .push(Attribute::new("class", "tab-list", None, false));

    rsx! {
        prim::TabList { ..props }
    }
}

#[component]
pub fn TabTrigger(mut props: prim::TabTriggerProps) -> Element {
    if props.class.is_none() {
        props.class = Some("tab-trigger".to_string());
    }

    rsx! {
        prim::TabTrigger { ..props }
    }
}

#[component]
pub fn TabContent(mut props: prim::TabContentProps) -> Element {
    if props.class.is_none() {
        props.class = Some("tab-content".to_string());
    }

    rsx! {
        prim::TabContent { ..props }
    }
}
