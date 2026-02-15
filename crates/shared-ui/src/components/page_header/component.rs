use dioxus::prelude::*;

/// Page header container — wraps a title and optional action buttons.
#[component]
pub fn PageHeader(children: Element) -> Element {
    rsx! {
        document::Link { rel: "stylesheet", href: asset!("./style.css") }
        div { class: "page-header",
            {children}
        }
    }
}

/// Page title element rendered as an h1.
#[component]
pub fn PageTitle(children: Element) -> Element {
    rsx! {
        h1 { class: "page-title", {children} }
    }
}

/// Container for action buttons in the page header.
#[component]
pub fn PageActions(children: Element) -> Element {
    rsx! {
        div { class: "page-actions", {children} }
    }
}
