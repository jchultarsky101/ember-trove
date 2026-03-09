use leptos::prelude::*;

use crate::{
    app::View,
    auth::{AuthState, AuthStatus},
    components::{
        dark_mode_toggle::DarkModeToggle, node_editor::NodeEditor, node_list::NodeList,
        node_view::NodeView, sidebar::Sidebar, tag_manager::TagManager,
    },
};

#[component]
pub fn Layout(auth_state: AuthState) -> impl IntoView {
    view! {
        <AuthGate auth_state=auth_state>
            <div class="flex h-screen overflow-hidden bg-gray-50 dark:bg-gray-950">
                <aside class="w-64 flex-shrink-0 border-r border-gray-200 dark:border-gray-800 flex flex-col">
                    <div class="flex items-center justify-between px-4 py-3 border-b border-gray-200 dark:border-gray-800">
                        <span class="font-semibold text-gray-900 dark:text-gray-100">
                            "Ember Trove"
                        </span>
                        <DarkModeToggle />
                    </div>
                    <Sidebar auth_state=auth_state />
                </aside>

                <main class="flex-1 overflow-auto flex flex-col">
                    <ViewSwitch />
                </main>
            </div>
        </AuthGate>
    }
}

#[component]
fn ViewSwitch() -> impl IntoView {
    let current_view = use_context::<RwSignal<View>>().expect("View signal must be provided");

    move || match current_view.get() {
        View::NodeList => view! { <NodeList /> }.into_any(),
        View::NodeDetail(id) => view! { <NodeView id=id /> }.into_any(),
        View::NodeCreate => view! { <NodeEditor node=None /> }.into_any(),
        View::NodeEdit(id) => view! { <NodeEditor node=Some(id) /> }.into_any(),
        View::TagManager => view! { <TagManager /> }.into_any(),
        View::Graph => {
            view! { <div class="p-6 text-gray-400">"Graph view — coming soon."</div> }.into_any()
        }
        View::Search => {
            view! { <div class="p-6 text-gray-400">"Search — Phase 5."</div> }.into_any()
        }
    }
}

/// Auth gate: spinner → login redirect → render app.
///
/// Children are always mounted but hidden until auth succeeds. The gate
/// overlay covers the viewport in Loading / Unauthenticated states.
#[component]
fn AuthGate(auth_state: AuthState, children: Children) -> impl IntoView {
    let app_view = children();

    view! {
        <div style:display=move || {
            if matches!(auth_state.get(), AuthStatus::Authenticated(_)) { "none" } else { "" }
        }>
            {move || match auth_state.get() {
                AuthStatus::Loading => view! {
                    <div class="flex items-center justify-center h-screen bg-gray-50 dark:bg-gray-950">
                        <div class="text-gray-400 dark:text-gray-500 text-sm">"Loading..."</div>
                    </div>
                }.into_any(),
                AuthStatus::Unauthenticated => {
                    wasm_bindgen_futures::spawn_local(async {
                        if let Ok(url) = crate::api::fetch_login_url().await
                            && let Some(window) = web_sys::window()
                        {
                            let _ = window.location().set_href(&url);
                        }
                    });
                    view! {
                        <div class="flex items-center justify-center h-screen bg-gray-50 dark:bg-gray-950">
                            <div class="text-gray-400 dark:text-gray-500 text-sm">"Redirecting to login..."</div>
                        </div>
                    }.into_any()
                }
                AuthStatus::Authenticated(_) => ().into_any(),
            }}
        </div>
        <div style:display=move || {
            if matches!(auth_state.get(), AuthStatus::Authenticated(_)) { "" } else { "none" }
        }>
            {app_view}
        </div>
    }
}
