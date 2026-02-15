use dioxus::prelude::*;
use dioxus_primitives::toggle_group as prim;

#[component]
pub fn ToggleGroup(mut props: prim::ToggleGroupProps) -> Element {
    props
        .attributes
        .push(Attribute::new("class", "toggle-group", None, false));

    rsx! {
        document::Link { rel: "stylesheet", href: asset!("./style.css") }
        prim::ToggleGroup { ..props }
    }
}

#[component]
pub fn ToggleGroupItem(mut props: prim::ToggleItemProps) -> Element {
    props
        .attributes
        .push(Attribute::new("class", "toggle-group-item", None, false));

    rsx! {
        prim::ToggleItem { ..props }
    }
}
