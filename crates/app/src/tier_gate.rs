use crate::auth::use_auth;
use dioxus::prelude::*;
use shared_types::UserTier;

/// Check if the current user meets a tier requirement.
pub fn use_tier_check(required: &UserTier) -> bool {
    let auth = use_auth();
    let guard = auth.current_user.read();
    match &*guard {
        Some(user) => user.tier.has_access(required),
        None => false,
    }
}

/// Conditionally render children based on user tier.
/// Shows `fallback` if the user's tier is insufficient.
#[component]
pub fn TierGate(required: UserTier, fallback: Element, children: Element) -> Element {
    let has_access = use_tier_check(&required);

    if has_access {
        rsx! { {children} }
    } else {
        rsx! { {fallback} }
    }
}
