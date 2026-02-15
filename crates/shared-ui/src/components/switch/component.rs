use dioxus::prelude::*;
use dioxus_primitives::switch as prim;

#[component]
pub fn Switch(mut props: prim::SwitchProps) -> Element {
    props
        .attributes
        .push(Attribute::new("class", "switch", None, false));

    rsx! {
        document::Link { rel: "stylesheet", href: asset!("./style.css") }
        prim::Switch { ..props }
    }
}

#[component]
pub fn SwitchThumb(mut props: prim::SwitchThumbProps) -> Element {
    props
        .attributes
        .push(Attribute::new("class", "switch-thumb", None, false));

    rsx! {
        prim::SwitchThumb { ..props }
    }
}
