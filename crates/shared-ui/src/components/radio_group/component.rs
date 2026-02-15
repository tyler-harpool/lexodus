use dioxus::prelude::*;
use dioxus_primitives::radio_group as prim;

#[component]
pub fn RadioGroup(mut props: prim::RadioGroupProps) -> Element {
    props
        .attributes
        .push(Attribute::new("class", "radio-group", None, false));

    rsx! {
        document::Link { rel: "stylesheet", href: asset!("./style.css") }
        prim::RadioGroup { ..props }
    }
}

#[component]
pub fn RadioGroupItem(mut props: prim::RadioItemProps) -> Element {
    if props.class.is_none() {
        props.class = Some("radio-item".to_string());
    }

    rsx! {
        prim::RadioItem { ..props }
    }
}
