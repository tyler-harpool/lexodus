use dioxus::prelude::*;

/// Which edge of the screen the sheet slides in from.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum SheetSide {
    Top,
    #[default]
    Right,
    Bottom,
    Left,
}

impl SheetSide {
    fn class(&self) -> &'static str {
        match self {
            SheetSide::Top => "top",
            SheetSide::Right => "right",
            SheetSide::Bottom => "bottom",
            SheetSide::Left => "left",
        }
    }
}

/// A cyberpunk-styled sliding panel overlay.
#[component]
pub fn Sheet(
    open: bool,
    on_close: EventHandler<()>,
    #[props(default)] side: SheetSide,
    children: Element,
) -> Element {
    if !open {
        return rsx! {};
    }

    rsx! {
        document::Link { rel: "stylesheet", href: asset!("./style.css") }
        div {
            class: "sheet-overlay",
            "data-open": "true",
            onclick: move |_| on_close.call(()),
            div {
                class: "sheet-panel",
                "data-side": side.class(),
                onclick: move |evt| evt.stop_propagation(),
                {children}
            }
        }
    }
}

/// Content area inside a Sheet.
#[component]
pub fn SheetContent(
    #[props(extends = GlobalAttributes)] attributes: Vec<Attribute>,
    children: Element,
) -> Element {
    let base = vec![Attribute::new("class", "sheet-content", None, false)];
    let merged = dioxus_primitives::merge_attributes(vec![base, attributes]);

    rsx! {
        div {
            ..merged,
            {children}
        }
    }
}

/// Header section of a Sheet.
#[component]
pub fn SheetHeader(
    #[props(extends = GlobalAttributes)] attributes: Vec<Attribute>,
    children: Element,
) -> Element {
    let base = vec![Attribute::new("class", "sheet-header", None, false)];
    let merged = dioxus_primitives::merge_attributes(vec![base, attributes]);

    rsx! {
        div {
            ..merged,
            {children}
        }
    }
}

/// Footer section of a Sheet.
#[component]
pub fn SheetFooter(
    #[props(extends = GlobalAttributes)] attributes: Vec<Attribute>,
    children: Element,
) -> Element {
    let base = vec![Attribute::new("class", "sheet-footer", None, false)];
    let merged = dioxus_primitives::merge_attributes(vec![base, attributes]);

    rsx! {
        div {
            ..merged,
            {children}
        }
    }
}

/// Title element within a SheetHeader.
#[component]
pub fn SheetTitle(
    #[props(extends = GlobalAttributes)] attributes: Vec<Attribute>,
    children: Element,
) -> Element {
    let base = vec![Attribute::new("class", "sheet-title", None, false)];
    let merged = dioxus_primitives::merge_attributes(vec![base, attributes]);

    rsx! {
        h2 {
            ..merged,
            {children}
        }
    }
}

/// Description text within a SheetHeader.
#[component]
pub fn SheetDescription(
    #[props(extends = GlobalAttributes)] attributes: Vec<Attribute>,
    children: Element,
) -> Element {
    let base = vec![Attribute::new("class", "sheet-description", None, false)];
    let merged = dioxus_primitives::merge_attributes(vec![base, attributes]);

    rsx! {
        p {
            ..merged,
            {children}
        }
    }
}

/// Close button for a Sheet.
#[component]
pub fn SheetClose(on_close: EventHandler<()>) -> Element {
    rsx! {
        button {
            class: "sheet-close",
            r#type: "button",
            "aria-label": "Close",
            onclick: move |_| on_close.call(()),
            "\u{2715}" // Unicode multiplication sign (X)
        }
    }
}
