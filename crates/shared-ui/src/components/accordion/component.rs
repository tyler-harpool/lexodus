use dioxus::prelude::*;
use dioxus_primitives::accordion as prim;

#[component]
pub fn Accordion(mut props: prim::AccordionProps) -> Element {
    props
        .attributes
        .push(Attribute::new("class", "accordion", None, false));

    rsx! {
        document::Link { rel: "stylesheet", href: asset!("./style.css") }
        prim::Accordion { ..props }
    }
}

#[component]
pub fn AccordionItem(mut props: prim::AccordionItemProps) -> Element {
    props
        .attributes
        .push(Attribute::new("class", "accordion-item", None, false));

    rsx! {
        prim::AccordionItem { ..props }
    }
}

#[component]
pub fn AccordionTrigger(mut props: prim::AccordionTriggerProps) -> Element {
    props
        .attributes
        .push(Attribute::new("class", "accordion-trigger", None, false));

    // Inject the chevron SVG as part of children by wrapping
    let original_children = props.children;
    props.children = rsx! {
        {original_children}
        svg {
            class: "accordion-chevron",
            xmlns: "http://www.w3.org/2000/svg",
            width: "16",
            height: "16",
            view_box: "0 0 24 24",
            fill: "none",
            stroke: "currentColor",
            stroke_width: "2",
            stroke_linecap: "round",
            stroke_linejoin: "round",
            path { d: "M6 9l6 6 6-6" }
        }
    };

    rsx! {
        prim::AccordionTrigger { ..props }
    }
}

#[component]
pub fn AccordionContent(mut props: prim::AccordionContentProps) -> Element {
    props
        .attributes
        .push(Attribute::new("class", "accordion-content", None, false));

    rsx! {
        prim::AccordionContent { ..props }
    }
}
