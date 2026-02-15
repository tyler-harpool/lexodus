use dioxus::prelude::*;
use dioxus_primitives::menubar as prim;

#[component]
pub fn MenubarRoot(mut props: prim::MenubarProps) -> Element {
    props
        .attributes
        .push(Attribute::new("class", "cyber-menubar", None, false));

    rsx! {
        document::Link { rel: "stylesheet", href: asset!("./style.css") }
        prim::Menubar { ..props }
    }
}

#[component]
pub fn MenubarMenu(mut props: prim::MenubarMenuProps) -> Element {
    props
        .attributes
        .push(Attribute::new("class", "cyber-menubar-menu", None, false));

    rsx! {
        prim::MenubarMenu { ..props }
    }
}

#[component]
pub fn MenubarTrigger(mut props: prim::MenubarTriggerProps) -> Element {
    props.attributes.push(Attribute::new(
        "class",
        "cyber-menubar-trigger",
        None,
        false,
    ));

    rsx! {
        prim::MenubarTrigger { ..props }
    }
}

#[component]
pub fn MenubarContent(mut props: prim::MenubarContentProps) -> Element {
    props.attributes.push(Attribute::new(
        "class",
        "cyber-menubar-content",
        None,
        false,
    ));

    rsx! {
        prim::MenubarContent { ..props }
    }
}

#[component]
pub fn MenubarItem(mut props: prim::MenubarItemProps) -> Element {
    props
        .attributes
        .push(Attribute::new("class", "cyber-menubar-item", None, false));

    rsx! {
        prim::MenubarItem { ..props }
    }
}

#[derive(Props, Clone, PartialEq)]
pub struct MenubarSeparatorProps {
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,
}

#[component]
pub fn MenubarSeparator(props: MenubarSeparatorProps) -> Element {
    rsx! {
        div {
            class: "cyber-menubar-separator",
            role: "separator",
            ..props.attributes,
        }
    }
}
