use common::auth::UserInfo;
use leptos::prelude::*;

use crate::{
    auth::AuthState,
    components::{
        dark_mode_toggle::DarkModeToggle,
        node_list::NodeList,
        sidebar::Sidebar,
    },
};

#[component]
pub fn Layout(auth_state: AuthState) -> impl IntoView {
    view! {
        <div class="flex h-screen overflow-hidden bg-gray-50 dark:bg-gray-950">
            // Left sidebar
            <aside class="w-64 flex-shrink-0 border-r border-gray-200 dark:border-gray-800 flex flex-col">
                <div class="flex items-center justify-between px-4 py-3 border-b border-gray-200 dark:border-gray-800">
                    <span class="font-semibold text-gray-900 dark:text-gray-100">
                        "Ember Trove"
                    </span>
                    <DarkModeToggle />
                </div>
                <Sidebar auth_state=auth_state />
            </aside>

            // Main panel
            <main class="flex-1 overflow-auto flex flex-col">
                <NodeList />
            </main>
        </div>
    }
}

/// Gate that redirects unauthenticated users.
/// Phase 1: pass-through. Phase 2 adds real OIDC redirect.
#[allow(dead_code)]
#[component]
pub fn AuthGate(children: Children, _auth_state: AuthState) -> impl IntoView {
    children()
}

// Suppress unused-import warning for UserInfo until Phase 2 uses it.
const _: Option<UserInfo> = None;
