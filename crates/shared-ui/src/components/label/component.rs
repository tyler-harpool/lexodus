use dioxus::prelude::*;
use dioxus_primitives::label as prim;

#[component]
pub fn Label(mut props: prim::LabelProps) -> Element {
    props
        .attributes
        .push(Attribute::new("class", "label", None, false));

    rsx! {
        document::Link { rel: "stylesheet", href: asset!("./style.css") }
        prim::Label { ..props }
    }
}
