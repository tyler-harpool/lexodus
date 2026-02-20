use dioxus::prelude::*;
use shared_types::UserRole;
use shared_ui::components::{Button, ButtonVariant};

use crate::auth::{use_auth, use_user_role};

/// Data scope for list pages: either "My Items" (filtered to current user)
/// or "All Court" (no user filter).
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ListScope {
    /// Show only items related to the current user (attorney's cases, judge's docket, etc.)
    MyItems,
    /// Show all items in the court (no user filter)
    AllCourt,
}

/// The resolved filter IDs extracted from the current user's auth state.
/// Used by list pages to decide which filter parameters to pass to the server.
#[derive(Clone, Debug, Default)]
pub struct ScopeFilter {
    /// If Some, filter by this attorney_id (for Attorney role in MyItems mode)
    pub attorney_id: Option<String>,
    /// If Some, filter by this judge_id (for Judge role in MyItems mode)
    pub judge_id: Option<String>,
}

/// A toggle component that lets Judge and Attorney users switch between
/// viewing their own items ("My Items") and all court items ("All Court").
///
/// Only renders for Judge and Attorney roles. For other roles, renders nothing.
///
/// Props:
/// - `scope`: a writable signal holding the current ListScope
///
/// Example usage in a list page:
/// ```rust,ignore
/// let mut scope = use_signal(|| ListScope::MyItems);
/// rsx! {
///     ScopeToggle { scope: scope }
/// }
/// ```
#[component]
pub fn ScopeToggle(scope: Signal<ListScope>) -> Element {
    let role = use_user_role();

    // Only show for Judge and Attorney roles
    if !matches!(role, UserRole::Judge | UserRole::Attorney) {
        return rsx! {};
    }

    let is_my_items = *scope.read() == ListScope::MyItems;

    rsx! {
        div { class: "scope-toggle",
            Button {
                variant: if is_my_items { ButtonVariant::Primary } else { ButtonVariant::Secondary },
                onclick: move |_| scope.set(ListScope::MyItems),
                "My Items"
            }
            Button {
                variant: if !is_my_items { ButtonVariant::Primary } else { ButtonVariant::Secondary },
                onclick: move |_| scope.set(ListScope::AllCourt),
                "All Court"
            }
        }
    }
}

/// Resolve the current scope into concrete filter IDs for server queries.
///
/// When scope is MyItems:
/// - For Attorneys: returns the linked_attorney_id
/// - For Judges: returns the linked_judge_id
///
/// When scope is AllCourt, returns empty (no filter).
pub fn use_scope_filter(scope: ListScope) -> ScopeFilter {
    let role = use_user_role();
    let auth = use_auth();

    if scope == ListScope::AllCourt {
        return ScopeFilter::default();
    }

    let user = auth.current_user.read();
    let user_ref = user.as_ref();

    match role {
        UserRole::Attorney => ScopeFilter {
            attorney_id: user_ref.and_then(|u| u.linked_attorney_id.clone()),
            judge_id: None,
        },
        UserRole::Judge => ScopeFilter {
            attorney_id: None,
            judge_id: user_ref.and_then(|u| u.linked_judge_id.clone()),
        },
        _ => ScopeFilter::default(),
    }
}
