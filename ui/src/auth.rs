/// OIDC authentication state and helpers for the UI.
///
/// Phase 1 stub — Phase 2 implements the full OIDC redirect flow,
/// token storage, and refresh logic.
use common::auth::UserInfo;
use leptos::prelude::*;

/// Global auth signal — `None` means unauthenticated.
pub type AuthState = RwSignal<Option<UserInfo>>;

/// Initialise the auth state signal and provide it via context.
pub fn provide_auth_state() -> AuthState {
    let auth_state: AuthState = RwSignal::new(None);
    provide_context(auth_state);
    auth_state
}

/// Read the auth state from context.
///
/// # Panics
///
/// Panics if called outside a component that provides `AuthState`.
#[allow(dead_code)]
#[must_use]
pub fn use_auth_state() -> AuthState {
    use_context::<AuthState>().expect("AuthState context must be provided")
}
