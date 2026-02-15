use dioxus::prelude::*;
use dioxus_primitives::toolbar as prim;

#[component]
pub fn Toolbar(mut props: prim::ToolbarProps) -> Element {
    props
        .attributes
        .push(Attribute::new("class", "toolbar", None, false));

    rsx! {
        document::Link { rel: "stylesheet", href: asset!("./style.css") }
        prim::Toolbar { ..props }
    }
}

#[component]
pub fn ToolbarButton(mut props: prim::ToolbarButtonProps) -> Element {
    props
        .attributes
        .push(Attribute::new("class", "toolbar-button", None, false));

    rsx! {
        prim::ToolbarButton { ..props }
    }
}

#[component]
pub fn ToolbarSeparator(mut props: prim::ToolbarSeparatorProps) -> Element {
    props
        .attributes
        .push(Attribute::new("class", "toolbar-separator", None, false));

    rsx! {
        prim::ToolbarSeparator { ..props }
    }
}
