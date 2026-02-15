use dioxus::prelude::*;

/// A cyberpunk-styled text input component.
#[component]
pub fn Input(
    #[props(default)] value: String,
    #[props(default)] on_input: EventHandler<FormEvent>,
    #[props(default)] placeholder: String,
    #[props(default)] label: String,
    #[props(default = "text".to_string())] input_type: String,
    #[props(default = false)] disabled: bool,
    #[props(extends = GlobalAttributes)] attributes: Vec<Attribute>,
) -> Element {
    let base = vec![Attribute::new("class", "input", None, false)];
    let merged = dioxus_primitives::merge_attributes(vec![base, attributes]);

    rsx! {
        document::Link { rel: "stylesheet", href: asset!("./style.css") }
        div { class: "input-wrapper",
            if !label.is_empty() {
                label { class: "input-label", "{label}" }
            }
            input {
                r#type: "{input_type}",
                value: value,
                placeholder: placeholder,
                disabled: disabled,
                oninput: move |evt| on_input.call(evt),
                ..merged,
            }
        }
    }
}
