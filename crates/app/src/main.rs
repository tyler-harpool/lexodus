use dioxus::prelude::*;
use shared_types::{FeatureFlags, UserTier};

mod auth;
pub mod billing_listener;
pub mod notify;
mod routes;
pub mod tier_gate;
use auth::{use_auth, AuthState};
use routes::Route;

/// Shared profile state accessible across all routes.
/// Backed by `Memo`s that read directly from `AuthState` — always in sync.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ProfileState {
    pub display_name: Memo<String>,
    pub email: Memo<String>,
    pub avatar_url: Memo<Option<String>>,
}

/// Selected court district context shared across all court domain routes.
#[derive(Clone, Copy)]
pub struct CourtContext {
    pub court_id: Signal<String>,
    /// The subscription tier for the currently selected court.
    pub court_tier: Signal<UserTier>,
}


pub const COURT_OPTIONS: &[(&str, &str)] = &[
    ("district9", "District 9 (Test)"),
    ("district12", "District 12 (Test)"),
    ("sdny", "Southern District of New York"),
    ("edny", "Eastern District of New York"),
    ("cdca", "Central District of California"),
    ("ndca", "Northern District of California"),
];

const THEME_BASE: Asset = asset!("/assets/theme-base.css");
const THEME_CYBERPUNK: Asset = asset!("/assets/themes/cyberpunk.css");
const THEME_SOLARIZED: Asset = asset!("/assets/themes/solarized.css");
const THEME_FEDERAL: Asset = asset!("/assets/themes/federal.css");
const THEME_CHAMBERS: Asset = asset!("/assets/themes/chambers.css");
const THEME_PARCHMENT: Asset = asset!("/assets/themes/parchment.css");

fn main() {
    #[cfg(feature = "server")]
    dioxus::serve(|| async move {
        server::config::load_feature_flags();
        let flags = server::config::feature_flags();

        if flags.telemetry {
            server::telemetry::init_telemetry();
        }
        server::health::record_start_time();

        let pool = server::db::create_pool();
        server::db::run_migrations(&pool).await;

        if flags.s3 {
            server::s3::ensure_bucket().await;
            // Also ensure the docket attachments bucket exists
            let att_store = server::storage::S3ObjectStore::from_env();
            att_store.ensure_bucket().await;
        }

        // Background task: clean up expired device authorizations every 15 minutes
        let cleanup_pool = pool.clone();
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(std::time::Duration::from_secs(15 * 60));
            loop {
                interval.tick().await;
                let _ = sqlx::query!(
                    "DELETE FROM device_authorizations WHERE expires_at < NOW() - INTERVAL '1 hour'"
                )
                .execute(&cleanup_pool)
                .await;
            }
        });

        let state = server::db::AppState { pool: pool.clone() };

        let mut router = dioxus::server::router(App).merge(server::openapi::api_router(pool));

        if flags.telemetry {
            router = router.layer(server::telemetry::OtelTraceLayer);
        }

        // Max upload size (default 50 MB) — configurable via MAX_UPLOAD_BYTES env var.
        let max_body: usize = std::env::var("MAX_UPLOAD_BYTES")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(50 * 1024 * 1024);

        let router = router
            .layer(axum::extract::DefaultBodyLimit::max(max_body))
            .layer(axum::middleware::from_fn_with_state(
                state,
                server::auth::middleware::auth_middleware,
            ))
            .layer(tower_http::request_id::PropagateRequestIdLayer::x_request_id())
            .layer(tower_http::request_id::SetRequestIdLayer::x_request_id(
                tower_http::request_id::MakeRequestUuid,
            ));
        Ok(router)
    });

    #[cfg(not(feature = "server"))]
    dioxus::launch(App);
}

/// Detect the client platform from compile-time feature flags.
pub fn client_platform() -> &'static str {
    if cfg!(feature = "web") {
        "web"
    } else if cfg!(feature = "desktop") {
        "desktop"
    } else if cfg!(feature = "mobile") {
        "mobile"
    } else {
        "unknown"
    }
}

#[component]
fn App() -> Element {
    // Set the X-Client-Platform header on all server function calls
    use_hook(|| {
        use dioxus::fullstack::{set_request_headers, HeaderMap, HeaderValue};

        let mut headers = HeaderMap::new();
        headers.insert(
            "x-client-platform",
            HeaderValue::from_static(client_platform()),
        );
        set_request_headers(headers);
    });

    // Fetch feature flags once and provide via context (defaults all-off on error)
    let flags_resource =
        use_server_future(move || async move { server::api::get_feature_flags().await })?;

    let flags = flags_resource
        .read()
        .as_ref()
        .cloned()
        .unwrap_or(Ok(FeatureFlags::default()))
        .unwrap_or_default();

    use_context_provider(|| flags);

    use_context_provider(AuthState::new);

    // Provide court context for court domain pages
    use_context_provider(|| CourtContext {
        court_id: Signal::new("district9".to_string()),
        court_tier: Signal::new(UserTier::Free),
    });

    // Auth state used by the profile memos below
    let auth = use_auth();
    let mut ctx = use_context::<CourtContext>();

    // Restore last selected court from user preferences on login
    {
        let auth_for_restore = auth.clone();
        use_effect(move || {
            if let Some(user) = auth_for_restore.current_user.read().as_ref() {
                if let Some(pref) = &user.preferred_court_id {
                    if !pref.is_empty() {
                        ctx.court_id.set(pref.clone());
                    }
                }
            }
        });
    }

    // Persist court selection to server whenever it changes (if logged in)
    {
        let auth_for_save = auth.clone();
        use_effect(move || {
            let court = ctx.court_id.read().clone();
            let has_user = auth_for_save.current_user.read().is_some();
            if has_user {
                spawn(async move {
                    let _ = server::api::set_preferred_court(court).await;
                });
            }
        });
    }

    // Sync court_tier from auth.current_user.court_tiers whenever user or court changes
    use_effect(move || {
        let court = ctx.court_id.read().clone();
        let tier = auth
            .current_user
            .read()
            .as_ref()
            .and_then(|u| u.court_tiers.get(&court).cloned())
            .map(|t| UserTier::from_str_or_default(&t))
            .unwrap_or(UserTier::Free);
        ctx.court_tier.set(tier);
    });

    // Derive profile state from auth — updates when user logs in/out
    let display_name = use_memo(move || {
        auth.current_user
            .read()
            .as_ref()
            .map(|u| u.display_name.clone())
            .unwrap_or_else(|| "Guest".to_string())
    });
    let email = use_memo(move || {
        auth.current_user
            .read()
            .as_ref()
            .map(|u| u.email.clone())
            .unwrap_or_else(|| "guest@example.com".to_string())
    });
    let avatar_url = use_memo(move || {
        auth.current_user
            .read()
            .as_ref()
            .and_then(|u| u.avatar_url.clone())
    });

    use_context_provider(|| ProfileState {
        display_name,
        email,
        avatar_url,
    });

    rsx! {
        document::Link { rel: "stylesheet", href: THEME_BASE }
        document::Link { rel: "stylesheet", href: THEME_CYBERPUNK }
        document::Link { rel: "stylesheet", href: THEME_SOLARIZED }
        document::Link { rel: "stylesheet", href: THEME_FEDERAL }
        document::Link { rel: "stylesheet", href: THEME_CHAMBERS }
        document::Link { rel: "stylesheet", href: THEME_PARCHMENT }
        shared_ui::theme::ThemeSeed {}
        shared_ui::ToastProvider {
            SuspenseBoundary {
                fallback: |_| rsx! {
                    div { class: "auth-guard-loading",
                        p { "Loading..." }
                    }
                },
                Router::<Route> {}
            }
        }
    }
}
