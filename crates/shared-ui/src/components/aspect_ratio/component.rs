use dioxus::prelude::*;
use dioxus_primitives::aspect_ratio as prim;

#[component]
pub fn AspectRatio(mut props: prim::AspectRatioProps) -> Element {
    props
        .attributes
        .push(Attribute::new("class", "aspect-ratio", None, false));

    rsx! {
        document::Link { rel: "stylesheet", href: asset!("./style.css") }
        prim::AspectRatio { ..props }
    }
}
