use dioxus::prelude::*;
use dioxus_primitives::dropdown_menu as prim;

#[component]
pub fn DropdownMenu(mut props: prim::DropdownMenuProps) -> Element {
    props
        .attributes
        .push(Attribute::new("class", "cyber-dropdown-menu", None, false));

    rsx! {
        document::Link { rel: "stylesheet", href: asset!("./style.css") }
        prim::DropdownMenu { ..props }
    }
}

#[component]
pub fn DropdownMenuTrigger(mut props: prim::DropdownMenuTriggerProps) -> Element {
    props.attributes.push(Attribute::new(
        "class",
        "cyber-dropdown-menu-trigger",
        None,
        false,
    ));

    rsx! {
        prim::DropdownMenuTrigger { ..props }
    }
}

#[component]
pub fn DropdownMenuContent(mut props: prim::DropdownMenuContentProps) -> Element {
    props.attributes.push(Attribute::new(
        "class",
        "cyber-dropdown-menu-content",
        None,
        false,
    ));

    rsx! {
        prim::DropdownMenuContent { ..props }
    }
}

#[component]
pub fn DropdownMenuItem<T: Clone + PartialEq + 'static>(
    mut props: prim::DropdownMenuItemProps<T>,
) -> Element {
    props.attributes.push(Attribute::new(
        "class",
        "cyber-dropdown-menu-item",
        None,
        false,
    ));

    rsx! {
        prim::DropdownMenuItem { ..props }
    }
}

#[derive(Props, Clone, PartialEq)]
pub struct DropdownMenuSeparatorProps {
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,
}

#[component]
pub fn DropdownMenuSeparator(props: DropdownMenuSeparatorProps) -> Element {
    rsx! {
        div {
            class: "cyber-dropdown-menu-separator",
            role: "separator",
            ..props.attributes,
        }
    }
}
