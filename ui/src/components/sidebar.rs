use leptos::prelude::*;

use crate::{
    app::View,
    auth::{AuthState, AuthStatus},
    components::{
        change_password_modal::ChangePasswordModal,
        favorites_section::FavoritesSection,
        layout::SidebarCollapsed,
        search_bar::SearchBar,
    },
};
use common::id::NodeId;

#[component]
pub fn Sidebar(auth_state: AuthState, collapsed: SidebarCollapsed, on_nav: Callback<()>) -> impl IntoView {
    let current_view = use_context::<RwSignal<View>>().expect("View signal must be provided");

    let show_change_pw = RwSignal::new(false);

    let on_logout = move |_| {
        wasm_bindgen_futures::spawn_local(async move {
            if let Ok(url) = crate::api::fetch_logout_url().await
                && let Some(window) = web_sys::window()
            {
                let _ = window.location().set_href(&url);
            }
        });
    };

    let node_type_filter =
        use_context::<RwSignal<Option<String>>>().expect("node_type_filter signal must be provided");

    view! {
        <nav class="flex-1 overflow-y-auto px-2 py-4 space-y-1">
            // Search — top of sidebar, always first
            // Expanded: inline SearchBar + "Browse by tag" shortcut
            // Collapsed: icon navigates to Search view
            {move || {
                if collapsed.get() {
                    view! {
                        <SidebarLink
                            icon="search" label="Search"
                            on_click=move || current_view.set(View::Search)
                            collapsed=collapsed
                        />
                    }.into_any()
                } else {
                    view! {
                        <div class="px-1 mb-1">
                            <SearchBar />
                        </div>
                    }.into_any()
                }
            }}
            <div class="border-t border-stone-200 dark:border-stone-700 my-3" />
            // Favorites — pinned nodes and external URLs
            <FavoritesSection collapsed=collapsed on_nav=on_nav />
            <div class="border-t border-stone-200 dark:border-stone-700 my-3" />
            // Recent nodes — re-reads localStorage on every view change
            <RecentSection collapsed=collapsed on_nav=on_nav />
            <div class="border-t border-stone-200 dark:border-stone-700 my-3" />
            // "All Nodes" + per-type sub-links
            <SidebarLink
                icon="segment" label="All Nodes"
                on_click=move || {
                    node_type_filter.set(None);
                    current_view.set(View::NodeList);
                    on_nav.run(());
                }
                collapsed=collapsed
            />
            // Type-specific sub-links (hidden when sidebar is collapsed)
            {move || {
                if collapsed.get() { return None; }
                Some(view! {
                    <div class="ml-3 border-l border-stone-200 dark:border-stone-700 pl-2 space-y-0.5">
                        <TypeFilterLink icon="description" label="Articles" value="article"
                            node_type_filter=node_type_filter current_view=current_view on_nav=on_nav />
                        <TypeFilterLink icon="rocket_launch" label="Projects" value="project"
                            node_type_filter=node_type_filter current_view=current_view on_nav=on_nav />
                        <TypeFilterLink icon="category" label="Areas" value="area"
                            node_type_filter=node_type_filter current_view=current_view on_nav=on_nav />
                        <TypeFilterLink icon="bookmarks" label="Resources" value="resource"
                            node_type_filter=node_type_filter current_view=current_view on_nav=on_nav />
                        <TypeFilterLink icon="menu_book" label="References" value="reference"
                            node_type_filter=node_type_filter current_view=current_view on_nav=on_nav />
                    </div>
                }.into_any())
            }}
            <div class="border-t border-stone-200 dark:border-stone-700 my-3" />
            <SidebarLink
                icon="label" label="Tags"
                on_click=move || { current_view.set(View::TagManager); on_nav.run(()); }
                collapsed=collapsed
            />
            <SidebarLink
                icon="sticky_note_2" label="Notes"
                on_click=move || { current_view.set(View::Notes); on_nav.run(()); }
                collapsed=collapsed
            />
            <SidebarLink
                icon="content_copy" label="Templates"
                on_click=move || { current_view.set(View::Templates); on_nav.run(()); }
                collapsed=collapsed
            />
            <div class="border-t border-stone-200 dark:border-stone-700 my-3" />
            <SidebarLink
                icon="inbox" label="Inbox"
                on_click=move || { current_view.set(View::Inbox); on_nav.run(()); }
                collapsed=collapsed
            />
            <SidebarLink
                icon="wb_sunny" label="My Day"
                on_click=move || { current_view.set(View::MyDay); on_nav.run(()); }
                collapsed=collapsed
            />
            <SidebarLink
                icon="calendar_month" label="Calendar"
                on_click=move || { current_view.set(View::Calendar); on_nav.run(()); }
                collapsed=collapsed
            />
            <SidebarLink
                icon="dashboard" label="Dashboard"
                on_click=move || { current_view.set(View::ProjectDashboard); on_nav.run(()); }
                collapsed=collapsed
            />
            <div class="border-t border-stone-200 dark:border-stone-700 my-3" />
            <SidebarLink
                icon="share" label="Graph"
                on_click=move || { current_view.set(View::Graph); on_nav.run(()); }
                collapsed=collapsed
            />
            {move || {
                if let AuthStatus::Authenticated(ref u) = auth_state.get()
                    && u.roles.contains(&"admin".to_string())
                {
                    Some(view! {
                        <div>
                            <div class="border-t border-stone-200 dark:border-stone-700 my-3" />
                            <SidebarLink
                                icon="group" label="Users"
                                on_click=move || { current_view.set(View::Admin); on_nav.run(()); }
                                collapsed=collapsed
                            />
                            <SidebarLink
                                icon="manage_accounts" label="Permissions"
                                on_click=move || { current_view.set(View::BulkPermissions); on_nav.run(()); }
                                collapsed=collapsed
                            />
                            <SidebarLink
                                icon="backup" label="Backup"
                                on_click=move || { current_view.set(View::Backup); on_nav.run(()); }
                                collapsed=collapsed
                            />
                        </div>
                    }.into_any())
                } else {
                    None
                }
            }}
            <div class="border-t border-stone-200 dark:border-stone-700 my-3" />
            // Export all nodes as a ZIP of Markdown files
            <SidebarLink
                icon="download"
                label="Export ZIP"
                on_click=move || {
                    if let Some(window) = web_sys::window() {
                        let _ = window.open_with_url("/api/export");
                    }
                }
                collapsed=collapsed
            />
        </nav>
        // User / logout section
        <div class="px-2 py-3 border-t border-stone-200 dark:border-stone-800">
            {move || {
                let is_collapsed = collapsed.get();
                if let AuthStatus::Authenticated(ref user) = auth_state.get() {
                    let name = user.name.clone()
                        .or_else(|| user.email.clone())
                        .unwrap_or_else(|| user.sub.clone());
                    if is_collapsed {
                        // Collapsed: stack of icon-only buttons with tooltips
                        Some(view! {
                            <div class="flex flex-col gap-1">
                                <button
                                    class="flex items-center justify-center w-full p-2 rounded-lg
                                        text-stone-500 hover:bg-stone-100 dark:hover:bg-stone-800
                                        dark:text-stone-400 cursor-pointer"
                                    title=format!("{name} — Change password")
                                    on:click=move |_| show_change_pw.set(true)
                                >
                                    <span class="material-symbols-outlined">"lock_reset"</span>
                                </button>
                                <button
                                    on:click=on_logout
                                    class="flex items-center justify-center w-full p-2 rounded-lg
                                        text-stone-500 hover:bg-stone-100 dark:hover:bg-stone-800
                                        dark:text-stone-400 cursor-pointer"
                                    title=format!("{name} — Logout")
                                >
                                    <span class="material-symbols-outlined">"logout"</span>
                                </button>
                            </div>
                        }.into_any())
                    } else {
                        // Expanded: username + change-password + logout
                        Some(view! {
                            <div class="space-y-1 px-1">
                                <span class="block text-xs text-stone-500 dark:text-stone-400 truncate">
                                    {name}
                                </span>
                                <div class="flex items-center justify-between">
                                    <button
                                        class="text-xs text-stone-400 hover:text-amber-500
                                               dark:hover:text-amber-400 cursor-pointer
                                               flex items-center gap-1 transition-colors"
                                        on:click=move |_| show_change_pw.set(true)
                                    >
                                        <span class="material-symbols-outlined" style="font-size: 13px;">
                                            "lock_reset"
                                        </span>
                                        "Change password"
                                    </button>
                                    <button
                                        class="text-xs text-stone-400 hover:text-stone-600
                                               dark:hover:text-stone-300 cursor-pointer"
                                        on:click=on_logout
                                    >
                                        "Logout"
                                    </button>
                                </div>
                            </div>
                        }.into_any())
                    }
                } else {
                    None
                }
            }}
        </div>

        // Change-password modal (portal rendered above everything)
        {move || show_change_pw.get().then(|| view! {
            <ChangePasswordModal on_close=Callback::new(move |_| show_change_pw.set(false)) />
        })}
    }
}

// ── Recent nodes ─────────────────────────────────────────────────────────────

/// Sidebar section that lists the last 10 visited nodes, read from
/// `localStorage`.  Re-reads on every view-signal change so new visits appear
/// immediately when the user navigates back to the sidebar.
#[component]
fn RecentSection(collapsed: SidebarCollapsed, on_nav: Callback<()>) -> impl IntoView {
    let current_view = use_context::<RwSignal<View>>().expect("View signal must be provided");

    move || {
        // Track current_view so this closure re-runs after each navigation.
        let _view = current_view.get();
        let entries = crate::recent::read_recent();

        if entries.is_empty() {
            return None;
        }

        let is_collapsed = collapsed.get();

        Some(view! {
            <div>
                // Section heading (hidden when collapsed — only icons show)
                {(!is_collapsed).then(|| view! {
                    <p class="px-3 mb-1 text-[10px] font-semibold uppercase tracking-widest
                               text-stone-400 dark:text-stone-500 select-none">
                        "Recent"
                    </p>
                })}
                {entries.into_iter().map(|entry| {
                    let id = entry.id;
                    let title = entry.title.clone();
                    let icon = entry.icon.clone();
                    let label = entry.title.clone();
                    view! {
                        <button
                            class=move || {
                                let base = "flex items-center w-full rounded-lg text-sm \
                                    text-stone-600 dark:text-stone-400 \
                                    hover:bg-stone-100 dark:hover:bg-stone-800 \
                                    hover:text-stone-800 dark:hover:text-stone-200 \
                                    transition-colors cursor-pointer py-1.5";
                                if collapsed.get() {
                                    format!("{base} justify-center px-0")
                                } else {
                                    format!("{base} px-3 gap-2")
                                }
                            }
                            title=title.clone()
                            on:click=move |_| {
                                current_view.set(View::NodeDetail(NodeId(id)));
                                on_nav.run(());
                            }
                        >
                            <span class="material-symbols-outlined shrink-0
                                         text-stone-400 dark:text-stone-500"
                                  style="font-size: 16px;">
                                {icon.clone()}
                            </span>
                            <span class="truncate text-xs"
                                  class:hidden=move || collapsed.get()>
                                {label.clone()}
                            </span>
                        </button>
                    }
                }).collect_view()}
            </div>
        }.into_any())
    }
}

/// Compact sub-link for filtering NodeList by a specific node type.
#[component]
fn TypeFilterLink(
    icon: &'static str,
    label: &'static str,
    value: &'static str,
    node_type_filter: RwSignal<Option<String>>,
    current_view: RwSignal<View>,
    on_nav: Callback<()>,
) -> impl IntoView {
    view! {
        <button
            class=move || {
                let active = node_type_filter.get().as_deref() == Some(value);
                let base = "flex items-center w-full gap-2 px-2 py-1.5 rounded-lg text-xs \
                            font-medium transition-colors cursor-pointer";
                if active {
                    format!("{base} bg-amber-50 dark:bg-amber-900/20 \
                             text-amber-700 dark:text-amber-400")
                } else {
                    format!("{base} text-stone-600 dark:text-stone-400 \
                             hover:bg-stone-100 dark:hover:bg-stone-800 \
                             hover:text-stone-800 dark:hover:text-stone-200")
                }
            }
            on:click=move |_| {
                node_type_filter.set(Some(value.to_string()));
                current_view.set(View::NodeList);
                on_nav.run(());
            }
            title=label
        >
            <span class="material-symbols-outlined" style="font-size: 14px;">{icon}</span>
            {label}
        </button>
    }
}

#[component]
fn SidebarLink(
    icon: &'static str,
    label: &'static str,
    #[prop(into)] on_click: Callback<()>,
    collapsed: SidebarCollapsed,
) -> impl IntoView {
    view! {
        <button
            class=move || {
                let base = "flex items-center w-full rounded-lg text-sm font-medium \
                    text-stone-700 dark:text-stone-300 hover:bg-stone-100 dark:hover:bg-stone-800 \
                    transition-colors cursor-pointer py-2";
                if collapsed.get() {
                    format!("{base} justify-center px-0")
                } else {
                    format!("{base} px-3 gap-3")
                }
            }
            title=label
            on:click=move |_| on_click.run(())
        >
            <span class="material-symbols-outlined text-stone-500 dark:text-stone-400">{icon}</span>
            <span
                class="truncate"
                class:hidden=move || collapsed.get()
            >
                {label}
            </span>
        </button>
    }
}
