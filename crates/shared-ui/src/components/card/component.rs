use dioxus::prelude::*;

/// A cyberpunk-styled card container.
#[component]
pub fn Card(
    #[props(extends = GlobalAttributes)] attributes: Vec<Attribute>,
    children: Element,
) -> Element {
    let base = vec![Attribute::new("class", "card", None, false)];
    let merged = dioxus_primitives::merge_attributes(vec![base, attributes]);

    rsx! {
        document::Link { rel: "stylesheet", href: asset!("./style.css") }
        div {
            ..merged,
            {children}
        }
    }
}

/// Header section of a Card.
#[component]
pub fn CardHeader(
    #[props(extends = GlobalAttributes)] attributes: Vec<Attribute>,
    children: Element,
) -> Element {
    let base = vec![Attribute::new("class", "card-header", None, false)];
    let merged = dioxus_primitives::merge_attributes(vec![base, attributes]);

    rsx! {
        div {
            ..merged,
            {children}
        }
    }
}

/// Title element within a CardHeader.
#[component]
pub fn CardTitle(
    #[props(extends = GlobalAttributes)] attributes: Vec<Attribute>,
    children: Element,
) -> Element {
    let base = vec![Attribute::new("class", "card-title", None, false)];
    let merged = dioxus_primitives::merge_attributes(vec![base, attributes]);

    rsx! {
        h3 {
            ..merged,
            {children}
        }
    }
}

/// Description text within a CardHeader.
#[component]
pub fn CardDescription(
    #[props(extends = GlobalAttributes)] attributes: Vec<Attribute>,
    children: Element,
) -> Element {
    let base = vec![Attribute::new("class", "card-description", None, false)];
    let merged = dioxus_primitives::merge_attributes(vec![base, attributes]);

    rsx! {
        p {
            ..merged,
            {children}
        }
    }
}

/// Action area within a CardHeader, typically for buttons or icons.
#[component]
pub fn CardAction(
    #[props(extends = GlobalAttributes)] attributes: Vec<Attribute>,
    children: Element,
) -> Element {
    let base = vec![Attribute::new("class", "card-action", None, false)];
    let merged = dioxus_primitives::merge_attributes(vec![base, attributes]);

    rsx! {
        div {
            ..merged,
            {children}
        }
    }
}

/// Main content section of a Card.
#[component]
pub fn CardContent(
    #[props(extends = GlobalAttributes)] attributes: Vec<Attribute>,
    children: Element,
) -> Element {
    let base = vec![Attribute::new("class", "card-content", None, false)];
    let merged = dioxus_primitives::merge_attributes(vec![base, attributes]);

    rsx! {
        div {
            ..merged,
            {children}
        }
    }
}

/// Footer section of a Card.
#[component]
pub fn CardFooter(
    #[props(extends = GlobalAttributes)] attributes: Vec<Attribute>,
    children: Element,
) -> Element {
    let base = vec![Attribute::new("class", "card-footer", None, false)];
    let merged = dioxus_primitives::merge_attributes(vec![base, attributes]);

    rsx! {
        div {
            ..merged,
            {children}
        }
    }
}
