use dioxus::prelude::*;

/// A cyberpunk-styled loading placeholder with animated pulse.
#[component]
pub fn Skeleton(#[props(extends = GlobalAttributes)] attributes: Vec<Attribute>) -> Element {
    let base = vec![Attribute::new("class", "skeleton", None, false)];
    let merged = dioxus_primitives::merge_attributes(vec![base, attributes]);

    rsx! {
        document::Link { rel: "stylesheet", href: asset!("./style.css") }
        div {
            ..merged,
        }
    }
}
