use dioxus::prelude::*;
use dioxus_primitives::calendar as prim;

pub use dioxus_primitives::calendar::{CalendarContext, DateRange, RangeCalendarContext};
pub use time::{Date, UtcDateTime};

#[component]
pub fn Calendar(mut props: prim::CalendarProps) -> Element {
    props
        .attributes
        .push(Attribute::new("class", "cyber-calendar", None, false));

    rsx! {
        document::Link { rel: "stylesheet", href: asset!("./style.css") }
        prim::Calendar { ..props }
    }
}

#[component]
pub fn RangeCalendar(mut props: prim::RangeCalendarProps) -> Element {
    props
        .attributes
        .push(Attribute::new("class", "cyber-calendar", None, false));

    rsx! {
        prim::RangeCalendar { ..props }
    }
}

#[component]
pub fn CalendarHeader(mut props: prim::CalendarHeaderProps) -> Element {
    props.attributes.push(Attribute::new(
        "class",
        "cyber-calendar-header",
        None,
        false,
    ));

    rsx! {
        prim::CalendarHeader { ..props }
    }
}

#[component]
pub fn CalendarNavigation(mut props: prim::CalendarNavigationProps) -> Element {
    props.attributes.push(Attribute::new(
        "class",
        "cyber-calendar-navigation",
        None,
        false,
    ));

    rsx! {
        prim::CalendarNavigation { ..props }
    }
}

#[component]
pub fn CalendarPreviousMonthButton(mut props: prim::CalendarPreviousMonthButtonProps) -> Element {
    props.attributes.push(Attribute::new(
        "class",
        "cyber-calendar-nav-btn",
        None,
        false,
    ));

    rsx! {
        prim::CalendarPreviousMonthButton { ..props }
    }
}

#[component]
pub fn CalendarNextMonthButton(mut props: prim::CalendarNextMonthButtonProps) -> Element {
    props.attributes.push(Attribute::new(
        "class",
        "cyber-calendar-nav-btn",
        None,
        false,
    ));

    rsx! {
        prim::CalendarNextMonthButton { ..props }
    }
}

#[component]
pub fn CalendarMonthTitle(mut props: prim::CalendarMonthTitleProps) -> Element {
    props.attributes.push(Attribute::new(
        "class",
        "cyber-calendar-month-title",
        None,
        false,
    ));

    rsx! {
        prim::CalendarMonthTitle { ..props }
    }
}

#[component]
pub fn CalendarGrid(mut props: prim::CalendarGridProps) -> Element {
    props
        .attributes
        .push(Attribute::new("class", "cyber-calendar-grid", None, false));

    rsx! {
        prim::CalendarGrid { ..props }
    }
}

#[component]
pub fn CalendarSelectMonth(mut props: prim::CalendarSelectMonthProps) -> Element {
    props.attributes.push(Attribute::new(
        "class",
        "cyber-calendar-select-month",
        None,
        false,
    ));

    rsx! {
        prim::CalendarSelectMonth { ..props }
    }
}

#[component]
pub fn CalendarSelectYear(mut props: prim::CalendarSelectYearProps) -> Element {
    props.attributes.push(Attribute::new(
        "class",
        "cyber-calendar-select-year",
        None,
        false,
    ));

    rsx! {
        prim::CalendarSelectYear { ..props }
    }
}

#[component]
pub fn CalendarDay(mut props: prim::CalendarDayProps) -> Element {
    props
        .attributes
        .push(Attribute::new("class", "cyber-calendar-day", None, false));

    rsx! {
        prim::CalendarDay { ..props }
    }
}
