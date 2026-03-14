use leptos::prelude::*;

use crate::{
    app::View,
    auth::{AuthState, AuthStatus},
    components::search_bar::SearchBar,
};

#[component]
pub fn Sidebar(auth_state: AuthState) -> impl IntoView {
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
        <nav class="flex-1 overflow-y-auto px-3 py-4 space-y-1">
            <SidebarLink
                icon="description" label="All Nodes"
                on_click=move || current_view.set(View::NodeList)
            />
            <div class="border-t border-gray-200 dark:border-gray-700 my-3" />
            <SidebarLink
                icon="label" label="Tags"
                on_click=move || current_view.set(View::TagManager)
            />
            <div class="border-t border-gray-200 dark:border-gray-700 my-3" />
            <SidebarLink
                icon="share" label="Graph"
                on_click=move || current_view.set(View::Graph)
            />
            <SidebarLink
                icon="search" label="Search"
                on_click=move || current_view.set(View::Search)
            />
            <div class="border-t border-gray-200 dark:border-gray-700 my-3" />
            <div class="px-1">
                <SearchBar />
            </div>
        </nav>
        <div class="px-3 py-3 border-t border-gray-200 dark:border-gray-800">
            {move || {
                if let AuthStatus::Authenticated(ref user) = auth_state.get() {
                    let name = user.name.clone().unwrap_or_else(|| user.sub.clone());
                    Some(view! {
                        <div class="flex items-center justify-between">
                            <span class="text-xs text-gray-500 dark:text-gray-400 truncate">
                                {name}
                            </span>
                            <button
                                class="text-xs text-gray-400 hover:text-gray-600 dark:hover:text-gray-300"
                                on:click=on_logout
                            >
                                "Logout"
                            </button>
                        </div>
                    })
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
) -> impl IntoView {
    view! {
        <button
            class="flex items-center gap-3 w-full px-3 py-2 text-sm font-medium rounded-lg
                text-gray-700 dark:text-gray-300 hover:bg-gray-100 dark:hover:bg-gray-800
                transition-colors"
            on:click=move |_| on_click.run(())
        >
            <span class="material-symbols-outlined text-gray-500 dark:text-gray-400">{icon}</span>
            {label}
        </button>
    }
}
