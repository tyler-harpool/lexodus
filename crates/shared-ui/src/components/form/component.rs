use dioxus::prelude::*;

/// A cyberpunk-styled form wrapper that prevents default submission.
#[component]
pub fn Form(
    #[props(default)] onsubmit: EventHandler<FormEvent>,
    #[props(extends = GlobalAttributes)] attributes: Vec<Attribute>,
    children: Element,
) -> Element {
    let base = vec![Attribute::new("class", "form", None, false)];
    let merged = dioxus_primitives::merge_attributes(vec![base, attributes]);

    rsx! {
        document::Link { rel: "stylesheet", href: asset!("./style.css") }
        form {
            onsubmit: move |evt| {
                evt.prevent_default();
                onsubmit.call(evt);
            },
            ..merged,
            {children}
        }
    }
}
