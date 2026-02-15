use dioxus::prelude::*;

/// Visual variant for badges.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum BadgeVariant {
    #[default]
    Primary,
    Secondary,
    Destructive,
    Outline,
}

impl BadgeVariant {
    fn class(&self) -> &'static str {
        match self {
            BadgeVariant::Primary => "primary",
            BadgeVariant::Secondary => "secondary",
            BadgeVariant::Destructive => "destructive",
            BadgeVariant::Outline => "outline",
        }
    }
}

/// A cyberpunk-styled badge component for inline labels and statuses.
#[component]
pub fn Badge(
    #[props(default)] variant: BadgeVariant,
    #[props(extends = GlobalAttributes)] attributes: Vec<Attribute>,
    children: Element,
) -> Element {
    let base = vec![
        Attribute::new("class", "badge", None, false),
        Attribute::new("data-style", variant.class(), None, false),
    ];
    let merged = dioxus_primitives::merge_attributes(vec![base, attributes]);

    rsx! {
        document::Link { rel: "stylesheet", href: asset!("./style.css") }
        span {
            ..merged,
            {children}
        }
    }
}
