use dioxus::prelude::*;
use dioxus_primitives::alert_dialog as prim;

#[component]
pub fn AlertDialogRoot(mut props: prim::AlertDialogRootProps) -> Element {
    props.attributes.push(Attribute::new(
        "class",
        "cyber-alert-dialog-overlay",
        None,
        false,
    ));

    rsx! {
        document::Link { rel: "stylesheet", href: asset!("./style.css") }
        prim::AlertDialogRoot { ..props }
    }
}

#[component]
pub fn AlertDialogContent(mut props: prim::AlertDialogContentProps) -> Element {
    if props.class.is_none() {
        props.class = Some("cyber-alert-dialog-content".to_string());
    }

    rsx! {
        prim::AlertDialogContent { ..props }
    }
}

#[component]
pub fn AlertDialogTitle(mut props: prim::AlertDialogTitleProps) -> Element {
    props.attributes.push(Attribute::new(
        "class",
        "cyber-alert-dialog-title",
        None,
        false,
    ));

    rsx! {
        prim::AlertDialogTitle { ..props }
    }
}

#[component]
pub fn AlertDialogDescription(mut props: prim::AlertDialogDescriptionProps) -> Element {
    props.attributes.push(Attribute::new(
        "class",
        "cyber-alert-dialog-description",
        None,
        false,
    ));

    rsx! {
        prim::AlertDialogDescription { ..props }
    }
}

#[component]
pub fn AlertDialogActions(mut props: prim::AlertDialogActionsProps) -> Element {
    props.attributes.push(Attribute::new(
        "class",
        "cyber-alert-dialog-actions",
        None,
        false,
    ));

    rsx! {
        prim::AlertDialogActions { ..props }
    }
}

#[component]
pub fn AlertDialogAction(mut props: prim::AlertDialogActionProps) -> Element {
    props.attributes.push(Attribute::new(
        "class",
        "cyber-alert-dialog-action",
        None,
        false,
    ));

    rsx! {
        prim::AlertDialogAction { ..props }
    }
}

#[component]
pub fn AlertDialogCancel(mut props: prim::AlertDialogCancelProps) -> Element {
    props.attributes.push(Attribute::new(
        "class",
        "cyber-alert-dialog-cancel",
        None,
        false,
    ));

    rsx! {
        prim::AlertDialogCancel { ..props }
    }
}
