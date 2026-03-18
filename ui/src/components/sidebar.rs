use leptos::prelude::*;

use crate::{
    app::View,
    auth::{AuthState, AuthStatus},
    components::{layout::SidebarCollapsed, search_bar::SearchBar},
};

#[component]
pub fn Sidebar(auth_state: AuthState, collapsed: SidebarCollapsed) -> impl IntoView {
    let current_view = use_context::<RwSignal<View>>().expect("View signal must be provided");

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
            // "All Nodes" + per-type sub-links
            <SidebarLink
                icon="segment" label="All Nodes"
                on_click=move || {
                    node_type_filter.set(None);
                    current_view.set(View::NodeList);
                }
                collapsed=collapsed
            />
            // Type-specific sub-links (hidden when sidebar is collapsed)
            {move || {
                if collapsed.get() { return None; }
                Some(view! {
                    <div class="ml-3 border-l border-stone-200 dark:border-stone-700 pl-2 space-y-0.5">
                        <TypeFilterLink
                            icon="description" label="Articles" value="article"
                            node_type_filter=node_type_filter
                            current_view=current_view
                        />
                        <TypeFilterLink
                            icon="rocket_launch" label="Projects" value="project"
                            node_type_filter=node_type_filter
                            current_view=current_view
                        />
                        <TypeFilterLink
                            icon="category" label="Areas" value="area"
                            node_type_filter=node_type_filter
                            current_view=current_view
                        />
                        <TypeFilterLink
                            icon="bookmarks" label="Resources" value="resource"
                            node_type_filter=node_type_filter
                            current_view=current_view
                        />
                        <TypeFilterLink
                            icon="menu_book" label="References" value="reference"
                            node_type_filter=node_type_filter
                            current_view=current_view
                        />
                    </div>
                }.into_any())
            }}
            <div class="border-t border-stone-200 dark:border-stone-700 my-3" />
            <SidebarLink
                icon="label" label="Tags"
                on_click=move || current_view.set(View::TagManager)
                collapsed=collapsed
            />
            <SidebarLink
                icon="sticky_note_2" label="Notes"
                on_click=move || current_view.set(View::Notes)
                collapsed=collapsed
            />
            <div class="border-t border-stone-200 dark:border-stone-700 my-3" />
            <SidebarLink
                icon="wb_sunny" label="My Day"
                on_click=move || current_view.set(View::MyDay)
                collapsed=collapsed
            />
            <SidebarLink
                icon="dashboard" label="Dashboard"
                on_click=move || current_view.set(View::ProjectDashboard)
                collapsed=collapsed
            />
            <div class="border-t border-stone-200 dark:border-stone-700 my-3" />
            <SidebarLink
                icon="share" label="Graph"
                on_click=move || current_view.set(View::Graph)
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
                                icon="admin_panel_settings" label="Admin"
                                on_click=move || current_view.set(View::Admin)
                                collapsed=collapsed
                            />
                            <SidebarLink
                                icon="backup" label="Backup"
                                on_click=move || current_view.set(View::Backup)
                                collapsed=collapsed
                            />
                        </div>
                    }.into_any())
                } else {
                    None
                }
            }}
        </nav>
        // User / logout section
        <div class="px-2 py-3 border-t border-stone-200 dark:border-stone-800">
            {move || {
                let is_collapsed = collapsed.get();
                if let AuthStatus::Authenticated(ref user) = auth_state.get() {
                    let name = user.name.clone().unwrap_or_else(|| user.sub.clone());
                    if is_collapsed {
                        // Collapsed: icon-only logout with tooltip showing username
                        Some(view! {
                            <button
                                on:click=on_logout
                                class="flex items-center justify-center w-full p-2 rounded-lg
                                    text-stone-500 hover:bg-stone-100 dark:hover:bg-stone-800
                                    dark:text-stone-400 cursor-pointer"
                                title=format!("{name} — Logout")
                            >
                                <span class="material-symbols-outlined">"logout"</span>
                            </button>
                        }.into_any())
                    } else {
                        // Expanded: username + logout link
                        Some(view! {
                            <div class="flex items-center justify-between px-1">
                                <span class="text-xs text-stone-500 dark:text-stone-400 truncate">
                                    {name}
                                </span>
                                <button
                                    class="text-xs text-stone-400 hover:text-stone-600 dark:hover:text-stone-300 cursor-pointer"
                                    on:click=on_logout
                                >
                                    "Logout"
                                </button>
                            </div>
                        }.into_any())
                    }
                } else {
                    None
                }
            }}
        </div>
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
