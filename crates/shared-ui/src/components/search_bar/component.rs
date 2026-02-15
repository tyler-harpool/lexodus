use dioxus::prelude::*;

/// Search/filter bar — wraps inputs, selects, and action buttons in a flex row.
#[component]
pub fn SearchBar(children: Element) -> Element {
    rsx! {
        document::Link { rel: "stylesheet", href: asset!("./style.css") }
        div { class: "search-bar",
            {children}
        }
    }
}
