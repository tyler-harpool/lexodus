use dioxus::prelude::*;
use dioxus_primitives::select as prim;

#[component]
pub fn SelectRoot<T: Clone + PartialEq + 'static>(mut props: prim::SelectProps<T>) -> Element {
    props
        .attributes
        .push(Attribute::new("class", "cyber-select", None, false));

    rsx! {
        document::Link { rel: "stylesheet", href: asset!("./style.css") }
        prim::Select { ..props }
    }
}

#[component]
pub fn SelectTrigger(mut props: prim::SelectTriggerProps) -> Element {
    props
        .attributes
        .push(Attribute::new("class", "cyber-select-trigger", None, false));

    rsx! {
        prim::SelectTrigger { ..props }
    }
}

#[component]
pub fn SelectValue(mut props: prim::SelectValueProps) -> Element {
    props
        .attributes
        .push(Attribute::new("class", "cyber-select-value", None, false));

    rsx! {
        prim::SelectValue { ..props }
    }
}

#[component]
pub fn SelectContent(mut props: prim::SelectListProps) -> Element {
    props
        .attributes
        .push(Attribute::new("class", "cyber-select-content", None, false));

    rsx! {
        prim::SelectList { ..props }
    }
}

#[component]
pub fn SelectItem<T: Clone + PartialEq + 'static>(
    mut props: prim::SelectOptionProps<T>,
) -> Element {
    props
        .attributes
        .push(Attribute::new("class", "cyber-select-item", None, false));

    rsx! {
        prim::SelectOption { ..props }
    }
}

#[component]
pub fn SelectGroup(mut props: prim::SelectGroupProps) -> Element {
    props
        .attributes
        .push(Attribute::new("class", "cyber-select-group", None, false));

    rsx! {
        prim::SelectGroup { ..props }
    }
}

#[component]
pub fn SelectGroupLabel(mut props: prim::SelectGroupLabelProps) -> Element {
    props.attributes.push(Attribute::new(
        "class",
        "cyber-select-group-label",
        None,
        false,
    ));

    rsx! {
        prim::SelectGroupLabel { ..props }
    }
}

#[component]
pub fn SelectItemIndicator(props: prim::SelectItemIndicatorProps) -> Element {
    rsx! {
        prim::SelectItemIndicator { ..props }
    }
}

#[derive(Props, Clone, PartialEq)]
pub struct SelectSeparatorProps {
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,
}

#[component]
pub fn SelectSeparator(props: SelectSeparatorProps) -> Element {
    rsx! {
        div {
            class: "cyber-select-separator",
            role: "separator",
            ..props.attributes,
        }
    }
}
