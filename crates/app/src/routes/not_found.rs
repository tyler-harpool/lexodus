use dioxus::prelude::*;

use crate::routes::Route;

/// 404 Not Found page.
#[component]
pub fn NotFound(route: Vec<String>) -> Element {
    let path = format!("/{}", route.join("/"));

    rsx! {
        document::Link { rel: "stylesheet", href: asset!("./not_found.css") }

        div { class: "not-found-page",
            div { class: "not-found-card",
                div { class: "not-found-code", "404" }
                h1 { class: "not-found-title", "Page Not Found" }
                p { class: "not-found-message",
                    "The page "
                    code { "{path}" }
                    " could not be found."
                }
                Link { to: Route::Dashboard {},
                    class: "not-found-link",
                    "Back to Dashboard"
                }
            }
        }
    }
}
