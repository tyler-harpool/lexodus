use dioxus::prelude::*;
use shared_types::UserTier;

/// Check if the currently selected court meets a tier requirement.
pub fn use_tier_check(required: &UserTier) -> bool {
    let ctx = use_context::<crate::CourtContext>();
    let court_tier = ctx.court_tier.read().clone();
    court_tier.has_access(required)
}

/// Conditionally render children based on the selected court's tier.
/// Shows `fallback` if the court's tier is insufficient.
#[component]
pub fn TierGate(required: UserTier, fallback: Element, children: Element) -> Element {
    let has_access = use_tier_check(&required);

    if has_access {
        rsx! { {children} }
    } else {
        rsx! { {fallback} }
    }
}
