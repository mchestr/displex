use axum_sessions::{async_session::{CookieStore}, SessionLayer};

use crate::config::SessionArgs;



pub fn configure(config: &SessionArgs) -> SessionLayer<CookieStore> {
    let store = CookieStore::new();
    let secret = config.session_secret_key.sensitive_string();
    SessionLayer::new(store, secret.as_bytes()).with_secure(false)
}