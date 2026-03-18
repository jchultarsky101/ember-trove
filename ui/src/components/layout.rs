use leptos::prelude::*;

use crate::{
    app::View,
    auth::{AuthState, AuthStatus},
    components::{
        admin_view::AdminView, backup_view::BackupView, dark_mode_toggle::DarkModeToggle,
        graph_view::GraphView, modals::create_node::CreateNodeModal, my_day_view::MyDayView,
        node_editor::NodeEditor, node_list::NodeList, node_view::NodeView,
        notes_view::NotesView, project_dashboard::ProjectDashboard, search_view::SearchView,
        sidebar::Sidebar, tag_manager::TagManager, toast::ToastOverlay,
    },
};

/// Whether the sidebar is collapsed (icon-only mode, desktop only).
pub type SidebarCollapsed = RwSignal<bool>;

#[component]
pub fn Layout(auth_state: AuthState) -> impl IntoView {
    let collapsed: SidebarCollapsed = RwSignal::new(false);
    let mobile_open: RwSignal<bool> = RwSignal::new(false);
    let show_capture: RwSignal<bool> = RwSignal::new(false);

    // Close mobile sidebar on any view change.
    let close_mobile = move || mobile_open.set(false);

    view! {
        <AuthGate auth_state=auth_state>
            // Outer shell — column on mobile (top-bar + content), row on desktop (sidebar + content).
            <div class="flex flex-col md:flex-row h-screen overflow-hidden bg-stone-50 dark:bg-stone-950">

                // ── Mobile top bar (hidden on md+) ──────────────────────────────────────
                <header class="md:hidden flex-shrink-0 flex items-center justify-between
                               px-4 py-3 border-b border-stone-200 dark:border-stone-800
                               bg-stone-50 dark:bg-stone-950 z-10">
                    <div class="flex items-center gap-2">
                        // Flame icon
                        <div class="w-7 h-7 flex-shrink-0">
                            <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 64 64" class="w-full h-full">
                                <defs>
                                    <linearGradient id="flame-m" x1="0" y1="1" x2="0" y2="0">
                                        <stop offset="0%" stop-color="#f59e0b"/>
                                        <stop offset="50%" stop-color="#ef4444"/>
                                        <stop offset="100%" stop-color="#f97316"/>
                                    </linearGradient>
                                </defs>
                                <path d="M32 4 C24 16, 12 24, 12 38 C12 50, 20 60, 32 60 C44 60, 52 50, 52 38 C52 24, 40 16, 32 4Z"
                                    fill="url(#flame-m)" opacity="0.9"/>
                                <path d="M32 22 C28 30, 22 34, 22 42 C22 50, 26 54, 32 54 C38 54, 42 50, 42 42 C42 34, 36 30, 32 22Z"
                                    fill="#fbbf24" opacity="0.85"/>
                                <circle cx="32" cy="38" r="3" fill="#ffffff" opacity="0.9"/>
                            </svg>
                        </div>
                        <span class="font-semibold text-stone-900 dark:text-stone-100 text-sm">
                            "Ember Trove"
                        </span>
                    </div>
                    // Hamburger button
                    <button
                        on:click=move |_| mobile_open.update(|o| *o = !*o)
                        class="p-2 rounded-lg text-stone-500 hover:bg-stone-100
                               dark:hover:bg-stone-800 dark:text-stone-400 cursor-pointer"
                        title="Toggle menu"
                    >
                        <span class="material-symbols-outlined" style="font-size: 22px;">
                            {move || if mobile_open.get() { "close" } else { "menu" }}
                        </span>
                    </button>
                </header>

                // ── Mobile backdrop ──────────────────────────────────────────────────────
                {move || mobile_open.get().then(|| view! {
                    <div
                        class="fixed inset-0 z-30 bg-black/40 md:hidden"
                        on:click=move |_| mobile_open.set(false)
                    />
                })}

                // ── Sidebar ─────────────────────────────────────────────────────────────
                // Desktop: normal flex child (w-64 or w-16).
                // Mobile: fixed overlay sliding in from the left.
                <aside
                    class=move || {
                        let base = "flex flex-col border-r border-stone-200 dark:border-stone-800 \
                                    bg-stone-50 dark:bg-stone-950 transition-all duration-200";
                        // Mobile: fixed overlay
                        let mobile = if mobile_open.get() {
                            "fixed inset-y-0 left-0 z-40 w-72 translate-x-0"
                        } else {
                            "fixed inset-y-0 left-0 z-40 w-72 -translate-x-full md:translate-x-0"
                        };
                        // Desktop width (overrides the fixed/translate on md+)
                        let desktop = if collapsed.get() {
                            "md:relative md:inset-auto md:w-16 md:flex-shrink-0"
                        } else {
                            "md:relative md:inset-auto md:w-64 md:flex-shrink-0"
                        };
                        format!("{base} {mobile} {desktop}")
                    }
                >
                    <SidebarHeader collapsed=collapsed />
                    <Sidebar auth_state=auth_state collapsed=collapsed on_nav=Callback::new(move |_| close_mobile()) />

                    // Desktop-only floating collapse toggle on the sidebar border
                    <button
                        on:click=move |_| collapsed.update(|c| *c = !*c)
                        class="hidden md:flex absolute right-0 top-1/2 -translate-y-1/2 translate-x-1/2 z-20
                            w-5 h-5 rounded-full
                            bg-white dark:bg-stone-900
                            border border-stone-200 dark:border-stone-700
                            shadow-sm items-center justify-center
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

                <main class="flex-1 overflow-auto flex flex-col min-w-0">
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

            // Toast notification overlay
            <ToastOverlay />
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
        View::ProjectDashboard => view! { <ProjectDashboard /> }.into_any(),
        View::MyDay => view! { <MyDayView /> }.into_any(),
        View::Notes => view! { <NotesView /> }.into_any(),
        View::Backup => view! { <BackupView /> }.into_any(),
    }
}

/// Auth gate: spinner → login redirect → render app.
///
/// Children are instantiated lazily — only once `AuthStatus::Authenticated`
/// is confirmed. This prevents unauthenticated API calls (which would trigger
/// parse_json's 401 → refresh → reload loop) from firing before login.
#[component]
fn AuthGate(auth_state: AuthState, children: ChildrenFn) -> impl IntoView {
    move || match auth_state.get() {
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
        AuthStatus::Authenticated(_) => children().into_any(),
    }
}
