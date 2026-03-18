use leptos::prelude::*;

use crate::{
    app::View,
    auth::{AuthState, AuthStatus},
    components::{
        admin_view::AdminView, dark_mode_toggle::DarkModeToggle, graph_view::GraphView,
        modals::create_node::CreateNodeModal,
        node_editor::NodeEditor,
        node_list::NodeList, node_view::NodeView, search_view::SearchView, sidebar::Sidebar,
        tag_manager::TagManager,
    },
};

/// Whether the sidebar is collapsed (icon-only mode).
pub type SidebarCollapsed = RwSignal<bool>;

#[component]
pub fn Layout(auth_state: AuthState) -> impl IntoView {
    let collapsed: SidebarCollapsed = RwSignal::new(false);
    let show_capture: RwSignal<bool> = RwSignal::new(false);

    view! {
        <AuthGate auth_state=auth_state>
            <div class="flex h-screen overflow-hidden bg-stone-50 dark:bg-stone-950">
                // Left sidebar — `relative` allows the floating toggle to be positioned on the border
                <aside
                    class="relative flex-shrink-0 border-r border-stone-200 dark:border-stone-800 flex flex-col transition-all duration-200"
                    class:w-64=move || !collapsed.get()
                    class:w-16=move || collapsed.get()
                >
                    <SidebarHeader collapsed=collapsed />
                    <Sidebar auth_state=auth_state collapsed=collapsed />

                    // Floating circular toggle — sits exactly on the right border, centred vertically
                    <button
                        on:click=move |_| collapsed.update(|c| *c = !*c)
                        class="absolute right-0 top-1/2 -translate-y-1/2 translate-x-1/2 z-20
                            w-5 h-5 rounded-full
                            bg-white dark:bg-stone-900
                            border border-stone-200 dark:border-stone-700
                            shadow-sm flex items-center justify-center
                            text-stone-400 hover:text-stone-600 dark:hover:text-stone-300
                            hover:border-stone-400 dark:hover:border-stone-500
                            hover:shadow-md transition-all cursor-pointer"
                        title=move || if collapsed.get() { "Expand sidebar" } else { "Collapse sidebar" }
                    >
                        <span
                            class="material-symbols-outlined"
                            style="font-size: 14px; line-height: 1;"
                        >
                            {move || if collapsed.get() { "chevron_right" } else { "chevron_left" }}
                        </span>
                    </button>
                </aside>

                <main class="flex-1 overflow-auto flex flex-col">
                    <ViewSwitch />
                </main>
            </div>

            // Floating Action Button — always on top, bottom-right
            <button
                class="fixed bottom-6 right-6 z-30
                       w-14 h-14 rounded-full shadow-lg
                       bg-gradient-to-br from-amber-500 to-orange-600
                       hover:from-amber-400 hover:to-orange-500
                       text-white flex items-center justify-center
                       hover:shadow-xl hover:scale-105
                       transition-all duration-150 cursor-pointer"
                title="Quick capture (new node)"
                on:click=move |_| show_capture.set(true)
            >
                <span class="material-symbols-outlined" style="font-size: 28px; font-weight: 300;">
                    "add"
                </span>
            </button>

            // Quick-capture modal
            <CreateNodeModal
                show=show_capture.read_only()
                on_close=Callback::new(move |_| show_capture.set(false))
            />
        </AuthGate>
    }
}

/// Sidebar header: banner icon + title + dark-mode toggle.
/// The collapse toggle is now a floating button on the aside border (see Layout).
#[component]
fn SidebarHeader(collapsed: SidebarCollapsed) -> impl IntoView {
    view! {
        <div class="flex items-center border-b border-stone-200 dark:border-stone-800 px-3 py-4 gap-2">
            // Banner icon — inline SVG ember flame
            <div class="flex-shrink-0 w-8 h-8">
                <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 64 64" class="w-full h-full">
                    <defs>
                        <linearGradient id="flame" x1="0" y1="1" x2="0" y2="0">
                            <stop offset="0%" stop-color="#f59e0b"/>
                            <stop offset="50%" stop-color="#ef4444"/>
                            <stop offset="100%" stop-color="#f97316"/>
                        </linearGradient>
                    </defs>
                    <path d="M32 4 C24 16, 12 24, 12 38 C12 50, 20 60, 32 60 C44 60, 52 50, 52 38 C52 24, 40 16, 32 4Z"
                        fill="url(#flame)" opacity="0.9"/>
                    <path d="M32 22 C28 30, 22 34, 22 42 C22 50, 26 54, 32 54 C38 54, 42 50, 42 42 C42 34, 36 30, 32 22Z"
                        fill="#fbbf24" opacity="0.85"/>
                    <circle cx="32" cy="38" r="3" fill="#ffffff" opacity="0.9"/>
                    <circle cx="26" cy="45" r="2" fill="#ffffff" opacity="0.7"/>
                    <circle cx="38" cy="45" r="2" fill="#ffffff" opacity="0.7"/>
                    <line x1="32" y1="38" x2="26" y2="45" stroke="#ffffff" stroke-width="0.8" opacity="0.5"/>
                    <line x1="32" y1="38" x2="38" y2="45" stroke="#ffffff" stroke-width="0.8" opacity="0.5"/>
                    <line x1="26" y1="45" x2="38" y2="45" stroke="#ffffff" stroke-width="0.8" opacity="0.5"/>
                </svg>
            </div>

            // Title + dark-mode toggle (hidden when collapsed)
            <div
                class="flex-1 flex items-center justify-between min-w-0 overflow-hidden"
                class:hidden=move || collapsed.get()
            >
                <span class="font-semibold text-stone-900 dark:text-stone-100 truncate">
                    "Ember Trove"
                </span>
                <DarkModeToggle />
            </div>
        </div>
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
        View::Graph => view! { <GraphView /> }.into_any(),
        View::Search => view! { <SearchView /> }.into_any(),
        View::Admin => view! { <AdminView /> }.into_any(),
    }
}

/// Auth gate: spinner → login redirect → render app.
#[component]
fn AuthGate(auth_state: AuthState, children: Children) -> impl IntoView {
    let app_view = children();

    view! {
        <div style:display=move || {
            if matches!(auth_state.get(), AuthStatus::Authenticated(_)) { "none" } else { "" }
        }>
            {move || match auth_state.get() {
                AuthStatus::Loading => view! {
                    <div class="flex items-center justify-center h-screen bg-stone-50 dark:bg-stone-950">
                        <div class="text-stone-400 dark:text-stone-500 text-sm">"Loading..."</div>
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
                        <div class="flex items-center justify-center h-screen bg-stone-50 dark:bg-stone-950">
                            <div class="text-stone-400 dark:text-stone-500 text-sm">"Redirecting to login..."</div>
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
