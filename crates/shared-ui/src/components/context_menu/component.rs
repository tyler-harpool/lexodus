use dioxus::prelude::*;
use dioxus_primitives::context_menu as prim;

#[component]
pub fn ContextMenu(mut props: prim::ContextMenuProps) -> Element {
    props
        .attributes
        .push(Attribute::new("class", "cyber-context-menu", None, false));

    rsx! {
        document::Link { rel: "stylesheet", href: asset!("./style.css") }
        prim::ContextMenu { ..props }
    }
}

#[component]
pub fn ContextMenuTrigger(mut props: prim::ContextMenuTriggerProps) -> Element {
    props.attributes.push(Attribute::new(
        "class",
        "cyber-context-menu-trigger",
        None,
        false,
    ));

    rsx! {
        prim::ContextMenuTrigger { ..props }
    }
}

#[component]
pub fn ContextMenuContent(mut props: prim::ContextMenuContentProps) -> Element {
    props.attributes.push(Attribute::new(
        "class",
        "cyber-context-menu-content",
        None,
        false,
    ));

    rsx! {
        prim::ContextMenuContent { ..props }
    }
}

#[component]
pub fn ContextMenuItem(mut props: prim::ContextMenuItemProps) -> Element {
    props.attributes.push(Attribute::new(
        "class",
        "cyber-context-menu-item",
        None,
        false,
    ));

    rsx! {
        prim::ContextMenuItem { ..props }
    }
}

#[derive(Props, Clone, PartialEq)]
pub struct ContextMenuSeparatorProps {
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,
}

#[component]
pub fn ContextMenuSeparator(props: ContextMenuSeparatorProps) -> Element {
    rsx! {
        div {
            class: "cyber-context-menu-separator",
            role: "separator",
            ..props.attributes,
        }
    }
}
