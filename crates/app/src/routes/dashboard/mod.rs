pub mod attorney;
pub mod clerk;
pub mod judge;

use dioxus::prelude::*;
use shared_types::UserRole;

use crate::auth::use_user_role;

/// Role-adaptive dashboard — renders the appropriate dashboard for the user's role.
#[component]
pub fn Dashboard() -> Element {
    let role = use_user_role();

    match role {
        UserRole::Admin | UserRole::Clerk => rsx! { clerk::ClerkDashboard {} },
        UserRole::Judge => rsx! { judge::JudgeDashboard {} },
        UserRole::Attorney => rsx! { attorney::AttorneyDashboard {} },
        UserRole::Public => rsx! { PublicDashboard {} },
    }
}

/// Minimal public dashboard with search focus.
#[component]
fn PublicDashboard() -> Element {
    rsx! {
        div { class: "dashboard-public",
            h1 { "Lexodus Public Access" }
            p { "Search public court records, opinions, and attorney information." }
        }
    }
}
