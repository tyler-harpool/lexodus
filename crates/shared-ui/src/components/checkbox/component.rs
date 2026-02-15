use dioxus::prelude::*;
use dioxus_primitives::checkbox as prim;

pub use prim::CheckboxState;

#[component]
pub fn Checkbox(mut props: prim::CheckboxProps) -> Element {
    props
        .attributes
        .push(Attribute::new("class", "checkbox", None, false));

    rsx! {
        document::Link { rel: "stylesheet", href: asset!("./style.css") }
        prim::Checkbox { ..props }
    }
}

#[component]
pub fn CheckboxIndicator(
    #[props(extends = GlobalAttributes)] attributes: Vec<Attribute>,
    children: Element,
) -> Element {
    let mut attrs = attributes;
    attrs.push(Attribute::new("class", "checkbox-indicator", None, false));

    let indicator_children = if children.is_ok() {
        children
    } else {
        rsx! {
            svg {
                class: "checkbox-icon",
                xmlns: "http://www.w3.org/2000/svg",
                width: "14",
                height: "14",
                view_box: "0 0 24 24",
                fill: "none",
                stroke: "currentColor",
                stroke_width: "3",
                stroke_linecap: "round",
                stroke_linejoin: "round",
                path { d: "M20 6L9 17l-5-5" }
            }
        }
    };

    rsx! {
        prim::CheckboxIndicator {
            attributes: attrs,
            {indicator_children}
        }
    }
}
