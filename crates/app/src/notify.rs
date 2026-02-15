use crate::auth::AuthState;
use dioxus::prelude::*;
use shared_types::AuthUser;

#[cfg(feature = "desktop")]
const APP_NAME: &str = "Lexodus";

/// Check whether push notifications are enabled for the given user.
fn is_push_enabled(user: Option<&AuthUser>) -> bool {
    user.map(|u| u.push_notifications_enabled).unwrap_or(false)
}

/// Send a desktop notification (no-op on non-desktop platforms).
#[allow(unused_variables)]
pub fn send(title: &str, body: &str) {
    #[cfg(feature = "desktop")]
    {
        if let Err(e) = dioxus_sdk_notification::Notification::new()
            .app_name(APP_NAME.to_string())
            .summary(title.to_string())
            .body(body.to_string())
            .show()
        {
            eprintln!("[notify] Failed to show desktop notification: {e}");
        }
    }
}

/// Send a desktop notification only when the user has push notifications enabled.
#[allow(unused_variables)]
pub fn send_if_enabled(auth: &AuthState, title: &str, body: &str) {
    let guard = auth.current_user.read();
    if is_push_enabled(guard.as_ref()) {
        send(title, body);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use shared_types::{AuthUser, UserTier};

    /// Build a minimal AuthUser with the given push preference.
    fn make_user(push_enabled: bool) -> AuthUser {
        AuthUser {
            id: 1,
            username: "testuser".into(),
            display_name: "Test User".into(),
            email: "test@example.com".into(),
            role: "user".into(),
            tier: UserTier::default(),
            avatar_url: None,
            email_verified: false,
            phone_number: None,
            phone_verified: false,
            email_notifications_enabled: true,
            push_notifications_enabled: push_enabled,
            weekly_digest_enabled: true,
            has_password: true,
            court_roles: std::collections::HashMap::new(),
        }
    }

    #[test]
    fn push_enabled_returns_true() {
        let user = make_user(true);
        assert!(is_push_enabled(Some(&user)));
    }

    #[test]
    fn push_disabled_returns_false() {
        let user = make_user(false);
        assert!(!is_push_enabled(Some(&user)));
    }

    #[test]
    fn no_user_returns_false() {
        assert!(!is_push_enabled(None));
    }

    #[test]
    fn send_noop_does_not_panic() {
        // Without the desktop feature, send() is a no-op and must not panic.
        send("Test Title", "Test body text");
    }
}
