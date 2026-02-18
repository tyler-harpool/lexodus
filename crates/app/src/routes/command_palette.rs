use dioxus::prelude::*;
use shared_ui::Separator;

use crate::routes::Route;

/// Global command palette overlay, toggled with Cmd+K / Ctrl+K.
#[component]
pub fn CommandPalette(show: Signal<bool>) -> Element {
    let mut query = use_signal(String::new);
    let nav = navigator();

    if !show() {
        return rsx! {};
    }

    let q = query.read().to_lowercase();

    // Static navigation items
    let nav_items: Vec<(&str, &str, Route)> = vec![
        ("Queue", "Go to work queue", Route::Dashboard {}),
        ("Cases", "View all cases", Route::CaseList {}),
        ("Calendar", "Court calendar", Route::CalendarList {}),
        ("Deadlines", "View deadlines", Route::DeadlineList {}),
        ("Attorneys", "View all attorneys", Route::AttorneyList {}),
        ("Judges", "View all judges", Route::JudgeList {}),
        ("Opinions", "View opinions", Route::OpinionList {}),
        (
            "Compliance",
            "Compliance dashboard",
            Route::ComplianceDashboard {},
        ),
        ("Rules", "Court rules", Route::RuleList {}),
        ("Users", "Manage users", Route::Users {}),
        (
            "Settings",
            "User settings",
            Route::Settings {
                billing: None,
                verified: None,
            },
        ),
    ];

    let filtered: Vec<_> = if q.is_empty() {
        nav_items.iter().take(8).collect()
    } else {
        nav_items
            .iter()
            .filter(|(name, desc, _)| {
                name.to_lowercase().contains(&q) || desc.to_lowercase().contains(&q)
            })
            .collect()
    };

    rsx! {
        // Backdrop
        div {
            class: "cmd-palette-backdrop",
            onclick: move |_| show.set(false),
        }
        div {
            class: "cmd-palette",
            div {
                class: "cmd-palette-input-wrap",
                input {
                    class: "input",
                    placeholder: "Type a command or search...",
                    value: "{query}",
                    oninput: move |e: FormEvent| query.set(e.value()),
                    autofocus: true,
                }
            }
            Separator {}
            div {
                class: "cmd-palette-results",
                if filtered.is_empty() {
                    p { class: "cmd-palette-empty", "No results found." }
                }
                for (name, desc, route) in filtered {
                    {
                        let route = route.clone();
                        rsx! {
                            button {
                                class: "cmd-palette-item",
                                onclick: move |_| {
                                    nav.push(route.clone());
                                    show.set(false);
                                    query.set(String::new());
                                },
                                span { class: "cmd-palette-item-name", "{name}" }
                                span { class: "cmd-palette-item-desc", "{desc}" }
                            }
                        }
                    }
                }
            }
        }
    }
}
