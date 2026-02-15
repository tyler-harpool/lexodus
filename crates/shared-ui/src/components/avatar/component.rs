use dioxus::prelude::*;
use dioxus_primitives::avatar as prim;

pub use dioxus_primitives::avatar::AvatarState;

#[component]
pub fn Avatar(mut props: prim::AvatarProps) -> Element {
    props
        .attributes
        .push(Attribute::new("class", "cyber-avatar", None, false));

    rsx! {
        document::Link { rel: "stylesheet", href: asset!("./style.css") }
        prim::Avatar { ..props }
    }
}

#[component]
pub fn AvatarImage(mut props: prim::AvatarImageProps) -> Element {
    props
        .attributes
        .push(Attribute::new("class", "cyber-avatar-image", None, false));

    rsx! {
        prim::AvatarImage { ..props }
    }
}

#[component]
pub fn AvatarFallback(mut props: prim::AvatarFallbackProps) -> Element {
    props.attributes.push(Attribute::new(
        "class",
        "cyber-avatar-fallback",
        None,
        false,
    ));

    rsx! {
        prim::AvatarFallback { ..props }
    }
}
