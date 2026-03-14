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

    view! {
        <nav class="flex-1 overflow-y-auto px-2 py-4 space-y-1">
            <SidebarLink
                icon="description" label="All Nodes"
                on_click=move || current_view.set(View::NodeList)
                collapsed=collapsed
            />
            <div class="border-t border-gray-200 dark:border-gray-700 my-3" />
            <SidebarLink
                icon="label" label="Tags"
                on_click=move || current_view.set(View::TagManager)
                collapsed=collapsed
            />
            <div class="border-t border-gray-200 dark:border-gray-700 my-3" />
            <SidebarLink
                icon="share" label="Graph"
                on_click=move || current_view.set(View::Graph)
                collapsed=collapsed
            />
            <SidebarLink
                icon="search" label="Search"
                on_click=move || current_view.set(View::Search)
                collapsed=collapsed
            />
            // SearchBar — hidden in icon-only mode
            <div
                class="border-t border-gray-200 dark:border-gray-700 my-3"
                class:hidden=move || collapsed.get()
            />
            <div
                class="px-1"
                class:hidden=move || collapsed.get()
            >
                <SearchBar />
            </div>
        </nav>
        // User / logout section
        <div class="px-2 py-3 border-t border-gray-200 dark:border-gray-800">
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
                                    text-gray-500 hover:bg-gray-100 dark:hover:bg-gray-800
                                    dark:text-gray-400 cursor-pointer"
                                title=format!("{name} — Logout")
                            >
                                <span class="material-symbols-outlined">"logout"</span>
                            </button>
                        }.into_any())
                    } else {
                        // Expanded: username + logout link
                        Some(view! {
                            <div class="flex items-center justify-between px-1">
                                <span class="text-xs text-gray-500 dark:text-gray-400 truncate">
                                    {name}
                                </span>
                                <button
                                    class="text-xs text-gray-400 hover:text-gray-600 dark:hover:text-gray-300 cursor-pointer"
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
                    text-gray-700 dark:text-gray-300 hover:bg-gray-100 dark:hover:bg-gray-800 \
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
            <span class="material-symbols-outlined text-gray-500 dark:text-gray-400">{icon}</span>
            <span
                class="truncate"
                class:hidden=move || collapsed.get()
            >
                {label}
            </span>
        </button>
    }
}
