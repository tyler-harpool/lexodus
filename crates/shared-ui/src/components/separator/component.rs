use dioxus::prelude::*;
use dioxus_primitives::separator as prim;

#[component]
pub fn Separator(mut props: prim::SeparatorProps) -> Element {
    props
        .attributes
        .push(Attribute::new("class", "separator", None, false));

    rsx! {
        document::Link { rel: "stylesheet", href: asset!("./style.css") }
        prim::Separator { ..props }
    }
}
