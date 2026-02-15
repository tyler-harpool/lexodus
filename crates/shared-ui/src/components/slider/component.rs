use dioxus::prelude::*;
use dioxus_primitives::slider as prim;

pub use dioxus_primitives::slider::SliderValue;

#[component]
pub fn SliderRoot(mut props: prim::SliderProps) -> Element {
    props
        .attributes
        .push(Attribute::new("class", "cyber-slider", None, false));

    rsx! {
        document::Link { rel: "stylesheet", href: asset!("./style.css") }
        prim::Slider { ..props }
    }
}

#[component]
pub fn SliderTrack(mut props: prim::SliderTrackProps) -> Element {
    props
        .attributes
        .push(Attribute::new("class", "cyber-slider-track", None, false));

    rsx! {
        prim::SliderTrack { ..props }
    }
}

#[component]
pub fn SliderRange(mut props: prim::SliderRangeProps) -> Element {
    props
        .attributes
        .push(Attribute::new("class", "cyber-slider-range", None, false));

    rsx! {
        prim::SliderRange { ..props }
    }
}

#[component]
pub fn SliderThumb(mut props: prim::SliderThumbProps) -> Element {
    props
        .attributes
        .push(Attribute::new("class", "cyber-slider-thumb", None, false));

    rsx! {
        prim::SliderThumb { ..props }
    }
}
