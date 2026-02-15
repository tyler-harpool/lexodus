use dioxus::prelude::*;
use dioxus_primitives::scroll_area as prim;

pub use prim::ScrollDirection;
pub use prim::ScrollType;

#[component]
pub fn ScrollArea(mut props: prim::ScrollAreaProps) -> Element {
    props
        .attributes
        .push(Attribute::new("class", "scroll-area", None, false));

    rsx! {
        document::Link { rel: "stylesheet", href: asset!("./style.css") }
        prim::ScrollArea { ..props }
    }
}
