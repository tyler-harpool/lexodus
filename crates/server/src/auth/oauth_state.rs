use oauth2::PkceCodeVerifier;
use std::collections::HashMap;
use std::sync::LazyLock;
use std::time::Instant;
use tokio::sync::Mutex;

/// CSRF state entry with PKCE verifier, creation timestamp, and optional redirect.
struct StateEntry {
    verifier: PkceCodeVerifier,
    created_at: Instant,
    /// Where to redirect after successful OAuth login (e.g. "/activate").
    redirect_after: Option<String>,
}

/// TTL for CSRF state entries.
const STATE_TTL_SECS: u64 = 600;

/// In-memory CSRF state store for OAuth flows.
static STATE_STORE: LazyLock<Mutex<HashMap<String, StateEntry>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

/// Store a CSRF state token with its PKCE verifier and optional redirect path.
pub async fn store_state(
    state: String,
    verifier: PkceCodeVerifier,
    redirect_after: Option<String>,
) {
    let mut store = STATE_STORE.lock().await;

    // Prune expired entries while we hold the lock
    let cutoff = Instant::now() - std::time::Duration::from_secs(STATE_TTL_SECS);
    store.retain(|_, entry| entry.created_at > cutoff);

    store.insert(
        state,
        StateEntry {
            verifier,
            created_at: Instant::now(),
            redirect_after,
        },
    );
}

/// Retrieve and remove a PKCE verifier and redirect path for a given CSRF state token.
/// Returns None if the state is unknown or expired.
pub async fn take_state(state: &str) -> Option<(PkceCodeVerifier, Option<String>)> {
    let mut store = STATE_STORE.lock().await;
    let entry = store.remove(state)?;

    let elapsed = entry.created_at.elapsed().as_secs();
    if elapsed > STATE_TTL_SECS {
        return None;
    }

    Some((entry.verifier, entry.redirect_after))
}
