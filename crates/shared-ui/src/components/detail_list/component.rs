use dioxus::prelude::*;

/// A container for label/value pairs in a detail view.
///
/// Renders a vertical list where each child `DetailItem` displays a key-value
/// row with proper spacing, hover highlight, and theme-aware styling.
#[component]
pub fn DetailList(children: Element) -> Element {
    rsx! {
        document::Link { rel: "stylesheet", href: asset!("./style.css") }
        div { class: "detail-list", {children} }
    }
}

/// A single label/value row inside a `DetailList`.
///
/// Displays the label on the left (uppercase, muted) and value on the right
/// (monospace). Includes a subtle hover highlight.
///
/// For plain text values, pass the `value` prop. For rich content (badges,
/// links, hover cards), use children instead.
#[component]
pub fn DetailItem(
    /// The field label (e.g. "Case Number").
    label: &'static str,
    /// The field value as a string. Ignored when children are provided.
    #[props(default)]
    value: String,
    /// Optional children for rich content (badges, links, etc).
    children: Element,
) -> Element {
    let has_children = children != Ok(VNode::placeholder());

    rsx! {
        div { class: "detail-item",
            span { class: "detail-item-label", "{label}" }
            span { class: "detail-item-value",
                if has_children {
                    {children}
                } else {
                    span { "{value}" }
                }
            }
        }
    }
}

/// Grid layout for multiple cards in a detail view.
///
/// Arranges child Card components in a responsive grid
/// (auto-fill columns with 350px minimum).
#[component]
pub fn DetailGrid(children: Element) -> Element {
    rsx! {
        document::Link { rel: "stylesheet", href: asset!("./style.css") }
        div { class: "detail-grid", {children} }
    }
}

/// Footer row for metadata like IDs and timestamps.
#[component]
pub fn DetailFooter(children: Element) -> Element {
    rsx! {
        document::Link { rel: "stylesheet", href: asset!("./style.css") }
        div { class: "detail-footer", {children} }
    }
}
