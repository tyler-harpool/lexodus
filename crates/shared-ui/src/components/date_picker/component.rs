use dioxus::prelude::*;
use dioxus_primitives::calendar::CalendarProps;
use dioxus_primitives::date_picker as prim;

pub use dioxus_primitives::date_picker::{DateRangePickerContext, DefaultCalendarProps};

#[component]
pub fn DatePicker(mut props: prim::DatePickerProps) -> Element {
    props
        .attributes
        .push(Attribute::new("class", "cyber-date-picker", None, false));

    rsx! {
        document::Link { rel: "stylesheet", href: asset!("./style.css") }
        prim::DatePicker { ..props }
    }
}

#[component]
pub fn DateRangePicker(mut props: prim::DateRangePickerProps) -> Element {
    props
        .attributes
        .push(Attribute::new("class", "cyber-date-picker", None, false));

    rsx! {
        prim::DateRangePicker { ..props }
    }
}

#[component]
pub fn DatePickerPopover(mut props: prim::DatePickerPopoverProps) -> Element {
    props.attributes.push(Attribute::new(
        "class",
        "cyber-date-picker-popover",
        None,
        false,
    ));

    rsx! {
        prim::DatePickerPopover { ..props }
    }
}

#[component]
pub fn DatePickerCalendar(mut props: prim::DatePickerCalendarProps<CalendarProps>) -> Element {
    props.attributes.push(Attribute::new(
        "class",
        "cyber-date-picker-calendar",
        None,
        false,
    ));

    rsx! {
        prim::DatePickerCalendar { ..props }
    }
}

#[component]
pub fn DatePickerInput(mut props: prim::DatePickerInputProps) -> Element {
    props.attributes.push(Attribute::new(
        "class",
        "cyber-date-picker-input",
        None,
        false,
    ));

    rsx! {
        prim::DatePickerInput { ..props }
    }
}

#[component]
pub fn DateRangePickerInput(mut props: prim::DatePickerInputProps) -> Element {
    props.attributes.push(Attribute::new(
        "class",
        "cyber-date-picker-input",
        None,
        false,
    ));

    rsx! {
        prim::DateRangePickerInput { ..props }
    }
}
