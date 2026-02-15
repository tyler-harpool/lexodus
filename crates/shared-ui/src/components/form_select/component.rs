use dioxus::prelude::*;

/// A themed native select element for forms and filters.
///
/// Simpler than the compound `SelectRoot` — wraps a native `<select>` with
/// `appearance: none` and co-located dark-theme styling. Use this for filter
/// dropdowns and form fields where a full primitive-backed select is overkill.
///
/// Children should be `option { value: "...", "Label" }` elements.
#[component]
pub fn FormSelect(
    /// Current selected value.
    #[props(default)]
    value: String,
    /// Called when the selection changes.
    #[props(default)]
    onchange: Option<EventHandler<Event<FormData>>>,
    /// Optional label displayed above the select.
    #[props(default)]
    label: String,
    /// Whether the select is disabled.
    #[props(default = false)]
    disabled: bool,
    /// Option elements to render inside the select.
    children: Element,
) -> Element {
    rsx! {
        document::Link { rel: "stylesheet", href: asset!("./style.css") }
        div { class: "form-select-wrapper",
            if !label.is_empty() {
                label { class: "form-select-label", "{label}" }
            }
            select {
                class: "form-select",
                value: value,
                disabled: disabled,
                onchange: move |evt| {
                    if let Some(handler) = &onchange {
                        handler.call(evt);
                    }
                },
                {children}
            }
        }
    }
}
