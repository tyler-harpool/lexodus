use dioxus::prelude::*;
#[allow(unused_imports)]
use shared_ui::{
    theme::{ThemeFamily, ALL_FAMILIES},
    AccordionContent, AccordionItem, AccordionTrigger, SelectContent, SelectItem, SelectRoot,
    SelectTrigger, SelectValue, Separator, Switch, SwitchThumb, Toggle,
};

/// Appearance accordion section: theme, dark/light toggle, animations, compact mode.
#[component]
pub fn AppearanceSection(index: usize) -> Element {
    let mut theme_state: shared_ui::theme::ThemeState = use_context();

    let mut animations_enabled = use_signal(|| true);
    let mut compact_mode = use_signal(|| false);

    let current_family = ThemeFamily::from_key(&(theme_state.family)());
    let show_mode_toggle = current_family.has_dark() && current_family.has_light();

    rsx! {
        AccordionItem {
            index: index,

            AccordionTrigger { "Appearance" }
            AccordionContent {
                div {
                    class: "settings-section-lg",

                    // Theme family selector
                    div {
                        class: "settings-theme-group",
                        span {
                            class: "settings-theme-label",
                            "Theme"
                        }
                        SelectRoot::<String> {
                            default_value: Some((theme_state.family)()),
                            on_value_change: move |val: Option<String>| {
                                if let Some(v) = val {
                                    theme_state.family.set(v);
                                    theme_state.apply();
                                }
                            },
                            SelectTrigger {
                                SelectValue {}
                            }
                            SelectContent {
                                for (i, family) in ALL_FAMILIES.iter().enumerate() {
                                    SelectItem::<String> {
                                        value: family.as_str().to_string(),
                                        index: i,
                                        "{family.display_name()}"
                                    }
                                }
                            }
                        }
                    }

                    Separator {}

                    // Dark/light toggle — only shown for dual-mode themes
                    if show_mode_toggle {
                        div {
                            class: "settings-toggle-row",
                            span {
                                class: "settings-toggle-label",
                                "Dark mode"
                            }
                            Switch {
                                checked: Some((theme_state.is_dark)()),
                                on_checked_change: move |val: bool| {
                                    theme_state.is_dark.set(val);
                                    theme_state.apply();
                                },
                                SwitchThumb {}
                            }
                        }

                        Separator {}
                    }

                    // Animations toggle
                    div {
                        class: "settings-toggle-row",
                        span {
                            class: "settings-toggle-label",
                            "Enable animations"
                        }
                        Toggle {
                            pressed: Some(animations_enabled()),
                            on_pressed_change: move |val: bool| {
                                animations_enabled.set(val);
                            },
                            "Animations"
                        }
                    }

                    Separator {}

                    // Compact mode switch
                    div {
                        class: "settings-toggle-row",
                        span {
                            class: "settings-toggle-label",
                            "Compact mode"
                        }
                        Switch {
                            checked: Some(compact_mode()),
                            on_checked_change: move |val: bool| {
                                compact_mode.set(val);
                            },
                            SwitchThumb {}
                        }
                    }
                }
            }
        }
    }
}
