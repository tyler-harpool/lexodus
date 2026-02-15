use dioxus::prelude::*;
use dioxus_primitives::toast as prim;

pub use dioxus_primitives::toast::{consume_toast, use_toast, ToastOptions, ToastType, Toasts};

#[component]
pub fn ToastProvider(props: prim::ToastProviderProps) -> Element {
    rsx! {
        document::Link { rel: "stylesheet", href: asset!("./style.css") }
        prim::ToastProvider { ..props }
    }
}

#[component]
pub fn Toast(mut props: prim::ToastProps) -> Element {
    props
        .attributes
        .push(Attribute::new("class", "cyber-toast", None, false));

    rsx! {
        prim::Toast { ..props }
    }
}
