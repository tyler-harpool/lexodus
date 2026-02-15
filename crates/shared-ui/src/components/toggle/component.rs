use dioxus::prelude::*;
use dioxus_primitives::toggle as prim;

#[component]
pub fn Toggle(mut props: prim::ToggleProps) -> Element {
    props
        .attributes
        .push(Attribute::new("class", "toggle", None, false));

    rsx! {
        document::Link { rel: "stylesheet", href: asset!("./style.css") }
        prim::Toggle { ..props }
    }
}
