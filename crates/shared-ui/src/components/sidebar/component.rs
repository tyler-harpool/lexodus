use dioxus::prelude::*;

// ─── Context ───────────────────────────────────────────────────────────

/// Shared state for controlling sidebar open/closed.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SidebarState {
    pub open: bool,
}

/// Provides sidebar state context to children.
#[component]
pub fn SidebarProvider(#[props(default = true)] default_open: bool, children: Element) -> Element {
    let state = use_signal(|| SidebarState { open: default_open });
    use_context_provider(|| state);

    rsx! {
        document::Link { rel: "stylesheet", href: asset!("./style.css") }
        div {
            class: "sidebar-provider",
            "data-sidebar-open": if (state)().open { "true" } else { "false" },
            {children}
        }
    }
}

/// Hook to access sidebar state.
fn use_sidebar() -> Signal<SidebarState> {
    use_context::<Signal<SidebarState>>()
}

// ─── Layout components ─────────────────────────────────────────────────

/// The main sidebar container. Collapses based on context state.
/// On mobile viewports, shows a backdrop overlay when open.
#[component]
pub fn Sidebar(
    #[props(extends = GlobalAttributes)] attributes: Vec<Attribute>,
    children: Element,
) -> Element {
    let mut state = use_sidebar();
    let is_open = (state)().open;

    let base = vec![
        Attribute::new("class", "sidebar", None, false),
        Attribute::new(
            "data-state",
            if is_open { "open" } else { "closed" },
            None,
            false,
        ),
    ];
    let merged = dioxus_primitives::merge_attributes(vec![base, attributes]);

    rsx! {
        // Mobile backdrop overlay - closes sidebar when tapped
        if is_open {
            div {
                class: "sidebar-backdrop",
                onclick: move |_| state.set(SidebarState { open: false }),
            }
        }
        aside {
            ..merged,
            {children}
        }
    }
}

/// Header section inside the Sidebar.
#[component]
pub fn SidebarHeader(
    #[props(extends = GlobalAttributes)] attributes: Vec<Attribute>,
    children: Element,
) -> Element {
    let base = vec![Attribute::new("class", "sidebar-header", None, false)];
    let merged = dioxus_primitives::merge_attributes(vec![base, attributes]);

    rsx! {
        div {
            ..merged,
            {children}
        }
    }
}

/// Scrollable content area of the Sidebar.
#[component]
pub fn SidebarContent(
    #[props(extends = GlobalAttributes)] attributes: Vec<Attribute>,
    children: Element,
) -> Element {
    let base = vec![Attribute::new("class", "sidebar-content", None, false)];
    let merged = dioxus_primitives::merge_attributes(vec![base, attributes]);

    rsx! {
        div {
            ..merged,
            {children}
        }
    }
}

/// Footer section inside the Sidebar.
#[component]
pub fn SidebarFooter(
    #[props(extends = GlobalAttributes)] attributes: Vec<Attribute>,
    children: Element,
) -> Element {
    let base = vec![Attribute::new("class", "sidebar-footer", None, false)];
    let merged = dioxus_primitives::merge_attributes(vec![base, attributes]);

    rsx! {
        div {
            ..merged,
            {children}
        }
    }
}

// ─── Group components ──────────────────────────────────────────────────

/// A group of related sidebar items.
#[component]
pub fn SidebarGroup(
    #[props(extends = GlobalAttributes)] attributes: Vec<Attribute>,
    children: Element,
) -> Element {
    let base = vec![Attribute::new("class", "sidebar-group", None, false)];
    let merged = dioxus_primitives::merge_attributes(vec![base, attributes]);

    rsx! {
        div {
            ..merged,
            {children}
        }
    }
}

/// Label for a SidebarGroup.
#[component]
pub fn SidebarGroupLabel(
    #[props(extends = GlobalAttributes)] attributes: Vec<Attribute>,
    children: Element,
) -> Element {
    let base = vec![Attribute::new("class", "sidebar-group-label", None, false)];
    let merged = dioxus_primitives::merge_attributes(vec![base, attributes]);

    rsx! {
        div {
            ..merged,
            {children}
        }
    }
}

/// Content container within a SidebarGroup.
#[component]
pub fn SidebarGroupContent(
    #[props(extends = GlobalAttributes)] attributes: Vec<Attribute>,
    children: Element,
) -> Element {
    let base = vec![Attribute::new(
        "class",
        "sidebar-group-content",
        None,
        false,
    )];
    let merged = dioxus_primitives::merge_attributes(vec![base, attributes]);

    rsx! {
        div {
            ..merged,
            {children}
        }
    }
}

// ─── Menu components ───────────────────────────────────────────────────

/// Navigation menu list inside the sidebar.
#[component]
pub fn SidebarMenu(
    #[props(extends = GlobalAttributes)] attributes: Vec<Attribute>,
    children: Element,
) -> Element {
    let base = vec![Attribute::new("class", "sidebar-menu", None, false)];
    let merged = dioxus_primitives::merge_attributes(vec![base, attributes]);

    rsx! {
        ul {
            ..merged,
            {children}
        }
    }
}

/// A single item in a SidebarMenu.
#[component]
pub fn SidebarMenuItem(
    #[props(extends = GlobalAttributes)] attributes: Vec<Attribute>,
    children: Element,
) -> Element {
    let base = vec![Attribute::new("class", "sidebar-menu-item", None, false)];
    let merged = dioxus_primitives::merge_attributes(vec![base, attributes]);

    rsx! {
        li {
            ..merged,
            {children}
        }
    }
}

/// Interactive button within a SidebarMenuItem.
/// On mobile viewports (overlay mode), clicking auto-closes the sidebar.
#[component]
pub fn SidebarMenuButton(
    #[props(default = false)] active: bool,
    #[props(extends = GlobalAttributes)] attributes: Vec<Attribute>,
    children: Element,
) -> Element {
    let mut state = use_sidebar();

    let base = vec![
        Attribute::new("class", "sidebar-menu-button", None, false),
        Attribute::new(
            "data-active",
            if active { "true" } else { "false" },
            None,
            false,
        ),
    ];
    let merged = dioxus_primitives::merge_attributes(vec![base, attributes]);

    rsx! {
        button {
            onclick: move |_| {
                state.set(SidebarState { open: false });
            },
            ..merged,
            {children}
        }
    }
}

/// Sub-menu container for nested navigation.
#[component]
pub fn SidebarMenuSub(
    #[props(extends = GlobalAttributes)] attributes: Vec<Attribute>,
    children: Element,
) -> Element {
    let base = vec![Attribute::new("class", "sidebar-menu-sub", None, false)];
    let merged = dioxus_primitives::merge_attributes(vec![base, attributes]);

    rsx! {
        ul {
            ..merged,
            {children}
        }
    }
}

/// Item within a SidebarMenuSub.
#[component]
pub fn SidebarMenuSubItem(
    #[props(extends = GlobalAttributes)] attributes: Vec<Attribute>,
    children: Element,
) -> Element {
    let base = vec![Attribute::new(
        "class",
        "sidebar-menu-sub-item",
        None,
        false,
    )];
    let merged = dioxus_primitives::merge_attributes(vec![base, attributes]);

    rsx! {
        li {
            ..merged,
            {children}
        }
    }
}

/// Interactive button within a SidebarMenuSubItem.
#[component]
pub fn SidebarMenuSubButton(
    #[props(default = false)] active: bool,
    #[props(extends = GlobalAttributes)] attributes: Vec<Attribute>,
    children: Element,
) -> Element {
    let base = vec![
        Attribute::new("class", "sidebar-menu-sub-button", None, false),
        Attribute::new(
            "data-active",
            if active { "true" } else { "false" },
            None,
            false,
        ),
    ];
    let merged = dioxus_primitives::merge_attributes(vec![base, attributes]);

    rsx! {
        button {
            ..merged,
            {children}
        }
    }
}

// ─── Utility components ────────────────────────────────────────────────

/// Toggle button that opens/closes the sidebar.
#[component]
pub fn SidebarTrigger(
    #[props(extends = GlobalAttributes)] attributes: Vec<Attribute>,
    children: Element,
) -> Element {
    let mut state = use_sidebar();

    let base = vec![Attribute::new("class", "sidebar-trigger", None, false)];
    let merged = dioxus_primitives::merge_attributes(vec![base, attributes]);

    rsx! {
        button {
            r#type: "button",
            "aria-label": "Toggle sidebar",
            onclick: move |_| {
                let current = (state)().open;
                state.set(SidebarState { open: !current });
            },
            ..merged,
            {children}
        }
    }
}

/// Visual separator line inside the sidebar.
#[component]
pub fn SidebarSeparator(
    #[props(extends = GlobalAttributes)] attributes: Vec<Attribute>,
) -> Element {
    let base = vec![Attribute::new("class", "sidebar-separator", None, false)];
    let merged = dioxus_primitives::merge_attributes(vec![base, attributes]);

    rsx! {
        hr {
            ..merged,
        }
    }
}

/// The main content area that sits alongside the Sidebar. Adjusts margin
/// based on sidebar open/closed state.
#[component]
pub fn SidebarInset(
    #[props(extends = GlobalAttributes)] attributes: Vec<Attribute>,
    children: Element,
) -> Element {
    let base = vec![Attribute::new("class", "sidebar-inset", None, false)];
    let merged = dioxus_primitives::merge_attributes(vec![base, attributes]);

    rsx! {
        main {
            ..merged,
            {children}
        }
    }
}

/// Thin rail on the sidebar edge for hover/click to toggle.
#[component]
pub fn SidebarRail() -> Element {
    let mut state = use_sidebar();

    rsx! {
        button {
            class: "sidebar-rail",
            r#type: "button",
            "aria-label": "Toggle sidebar",
            tabindex: -1,
            onclick: move |_| {
                let current = (state)().open;
                state.set(SidebarState { open: !current });
            },
        }
    }
}
