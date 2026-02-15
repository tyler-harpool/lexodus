use dioxus::prelude::*;
use dioxus_primitives::navbar as prim;

#[component]
pub fn Navbar(mut props: prim::NavbarProps) -> Element {
    props
        .attributes
        .push(Attribute::new("class", "cyber-navbar", None, false));

    rsx! {
        document::Link { rel: "stylesheet", href: asset!("./style.css") }
        prim::Navbar { ..props }
    }
}

#[component]
pub fn NavbarNav(mut props: prim::NavbarNavProps) -> Element {
    props
        .attributes
        .push(Attribute::new("class", "cyber-navbar-nav", None, false));

    rsx! {
        prim::NavbarNav { ..props }
    }
}

#[component]
pub fn NavbarTrigger(mut props: prim::NavbarTriggerProps) -> Element {
    props
        .attributes
        .push(Attribute::new("class", "cyber-navbar-trigger", None, false));

    rsx! {
        prim::NavbarTrigger { ..props }
    }
}

#[component]
pub fn NavbarContent(mut props: prim::NavbarContentProps) -> Element {
    props
        .attributes
        .push(Attribute::new("class", "cyber-navbar-content", None, false));

    rsx! {
        prim::NavbarContent { ..props }
    }
}

#[component]
pub fn NavbarItem(mut props: prim::NavbarItemProps) -> Element {
    if props.class.is_none() {
        props.class = Some("cyber-navbar-item".to_string());
    }

    rsx! {
        prim::NavbarItem { ..props }
    }
}
