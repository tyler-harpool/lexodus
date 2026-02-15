use dioxus::prelude::*;
use dioxus_primitives::dialog as prim;

#[component]
pub fn DialogRoot(mut props: prim::DialogRootProps) -> Element {
    props
        .attributes
        .push(Attribute::new("class", "cyber-dialog-overlay", None, false));

    rsx! {
        document::Link { rel: "stylesheet", href: asset!("./style.css") }
        prim::DialogRoot { ..props }
    }
}

#[component]
pub fn DialogContent(mut props: prim::DialogContentProps) -> Element {
    if props.class.is_none() {
        props.class = Some("cyber-dialog-content".to_string());
    }

    rsx! {
        prim::DialogContent { ..props }
    }
}

#[component]
pub fn DialogTitle(mut props: prim::DialogTitleProps) -> Element {
    props
        .attributes
        .push(Attribute::new("class", "cyber-dialog-title", None, false));

    rsx! {
        prim::DialogTitle { ..props }
    }
}

#[component]
pub fn DialogDescription(mut props: prim::DialogDescriptionProps) -> Element {
    props.attributes.push(Attribute::new(
        "class",
        "cyber-dialog-description",
        None,
        false,
    ));

    rsx! {
        prim::DialogDescription { ..props }
    }
}
