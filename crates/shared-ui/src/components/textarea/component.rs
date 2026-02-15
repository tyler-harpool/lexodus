use dioxus::prelude::*;

/// Visual variant for textareas.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum TextareaVariant {
    #[default]
    Default,
    Fade,
    Outline,
    Ghost,
}

impl TextareaVariant {
    fn class(&self) -> &'static str {
        match self {
            TextareaVariant::Default => "default",
            TextareaVariant::Fade => "fade",
            TextareaVariant::Outline => "outline",
            TextareaVariant::Ghost => "ghost",
        }
    }
}

/// A cyberpunk-styled multi-line text input component.
#[component]
pub fn Textarea(
    #[props(default)] variant: TextareaVariant,
    #[props(default)] value: String,
    #[props(default)] on_input: EventHandler<FormEvent>,
    #[props(default)] placeholder: String,
    #[props(default)] label: String,
    #[props(default = false)] disabled: bool,
    #[props(extends = GlobalAttributes)] attributes: Vec<Attribute>,
) -> Element {
    let base = vec![
        Attribute::new("class", "textarea", None, false),
        Attribute::new("data-style", variant.class(), None, false),
    ];
    let merged = dioxus_primitives::merge_attributes(vec![base, attributes]);

    rsx! {
        document::Link { rel: "stylesheet", href: asset!("./style.css") }
        div { class: "textarea-wrapper",
            if !label.is_empty() {
                label { class: "textarea-label", "{label}" }
            }
            textarea {
                value: value,
                placeholder: placeholder,
                disabled: disabled,
                oninput: move |evt| on_input.call(evt),
                ..merged,
            }
        }
    }
}
