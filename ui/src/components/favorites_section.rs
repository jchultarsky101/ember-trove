use common::{favorite::Favorite, id::FavoriteId};
use leptos::prelude::*;
use wasm_bindgen_futures::spawn_local;

use crate::{
    api::{delete_favorite, fetch_favorites, reorder_favorites},
    app::View,
    components::{
        layout::SidebarCollapsed,
        modals::add_favorite::AddFavoriteModal,
        toast::{ToastLevel, push_toast},
    },
};

/// Sidebar section that shows the user's pinned favorites (nodes and external URLs).
/// Position: below search, above "All Nodes".
#[component]
pub fn FavoritesSection(collapsed: SidebarCollapsed, on_nav: Callback<()>) -> impl IntoView {
    let favorites: RwSignal<Vec<Favorite>> = RwSignal::new(vec![]);
    let show_modal = RwSignal::new(false);
    let current_view = use_context::<RwSignal<View>>().expect("View signal");

    // Load favorites once on mount.
    Effect::new(move |_| {
        spawn_local(async move {
            if let Ok(favs) = fetch_favorites().await {
                favorites.set(favs);
            }
        });
    });

    let on_added = Callback::new(move |fav: Favorite| {
        favorites.update(|list| list.push(fav));
    });

    let on_delete = Callback::new(move |id: FavoriteId| {
        favorites.update(|list| list.retain(|f| f.id != id));
        spawn_local(async move {
            if let Err(e) = delete_favorite(id).await {
                push_toast(ToastLevel::Error, format!("Failed to remove favorite: {e}"));
            }
        });
    });

    let move_up = Callback::new(move |id: FavoriteId| {
        favorites.update(|list| {
            if let Some(idx) = list.iter().position(|f| f.id == id) {
                if idx > 0 {
                    list.swap(idx, idx - 1);
                }
            }
        });
        let ids: Vec<uuid::Uuid> = favorites.get_untracked().iter().map(|f| f.id.0).collect();
        spawn_local(async move {
            let req = common::favorite::ReorderFavoritesRequest { ids };
            if let Err(e) = reorder_favorites(&req).await {
                push_toast(ToastLevel::Error, format!("Reorder failed: {e}"));
            }
        });
    });

    let move_down = Callback::new(move |id: FavoriteId| {
        favorites.update(|list| {
            if let Some(idx) = list.iter().position(|f| f.id == id) {
                if idx + 1 < list.len() {
                    list.swap(idx, idx + 1);
                }
            }
        });
        let ids: Vec<uuid::Uuid> = favorites.get_untracked().iter().map(|f| f.id.0).collect();
        spawn_local(async move {
            let req = common::favorite::ReorderFavoritesRequest { ids };
            if let Err(e) = reorder_favorites(&req).await {
                push_toast(ToastLevel::Error, format!("Reorder failed: {e}"));
            }
        });
    });

    view! {
        // Collapsed: single star icon that opens the add modal
        {move || {
            if collapsed.get() {
                return view! {
                    <button
                        class="flex items-center justify-center w-full py-2 rounded-lg
                               text-stone-700 dark:text-stone-300 hover:bg-stone-100
                               dark:hover:bg-stone-800 transition-colors cursor-pointer"
                        title="Favorites"
                        on:click=move |_| show_modal.set(true)
                    >
                        <span class="material-symbols-outlined text-stone-500 dark:text-stone-400">"star"</span>
                    </button>
                }.into_any();
            }

            // Expanded: full favorites list
            view! {
                <div>
                    // Section header
                    <div class="flex items-center justify-between px-3 mb-1">
                        <span class="text-xs font-semibold uppercase tracking-wider
                                     text-stone-500 dark:text-stone-400">
                            "Favorites"
                        </span>
                        <button
                            class="text-stone-400 hover:text-amber-500 dark:hover:text-amber-400
                                   transition-colors cursor-pointer"
                            title="Add favorite"
                            on:click=move |_| show_modal.set(true)
                        >
                            <span class="material-symbols-outlined" style="font-size: 18px">"add"</span>
                        </button>
                    </div>

                    // Favorite items
                    {move || {
                        let list = favorites.get();
                        let len = list.len();
                        if list.is_empty() {
                            return view! {
                                <p class="px-3 py-1 text-xs text-stone-400 dark:text-stone-500 italic">
                                    "No favorites yet"
                                </p>
                            }.into_any();
                        }
                        list.into_iter().enumerate().map(|(idx, fav)| {
                            let fav_id = fav.id;
                            let label = fav.label.clone();
                            let url = fav.url.clone();
                            let node_id = fav.node_id;
                            let is_first = idx == 0;
                            let is_last = idx + 1 == len;

                            view! {
                                <FavoriteRow
                                    label=label
                                    url=url
                                    node_id=node_id
                                    fav_id=fav_id
                                    is_first=is_first
                                    is_last=is_last
                                    on_delete=on_delete
                                    on_move_up=move_up
                                    on_move_down=move_down
                                    current_view=current_view
                                    on_nav=on_nav
                                />
                            }
                        }).collect_view().into_any()
                    }}
                </div>
            }.into_any()
        }}

        // Modal
        <AddFavoriteModal
            show=Signal::derive(move || show_modal.get())
            on_close=Callback::new(move |()| show_modal.set(false))
            on_added=on_added
        />
    }
}

#[component]
#[allow(clippy::too_many_arguments)]
fn FavoriteRow(
    label: String,
    url: Option<String>,
    node_id: Option<common::id::NodeId>,
    fav_id: FavoriteId,
    is_first: bool,
    is_last: bool,
    on_delete: Callback<FavoriteId>,
    on_move_up: Callback<FavoriteId>,
    on_move_down: Callback<FavoriteId>,
    current_view: RwSignal<View>,
    on_nav: Callback<()>,
) -> impl IntoView {
    let label2 = label.clone();
    let url2 = url.clone();

    let handle_click = move || {
        if let Some(nid) = node_id {
            current_view.set(View::NodeDetail(nid));
            on_nav.run(());
        } else if let Some(ref u) = url2 {
            if let Some(win) = web_sys::window() {
                let _ = win.open_with_url_and_target(u, "_blank");
            }
        }
    };

    view! {
        <div class="group flex items-center gap-1 px-2 py-0.5 rounded-lg
                    hover:bg-stone-100 dark:hover:bg-stone-800 transition-colors">
            // Star / link icon
            <span class="material-symbols-outlined text-amber-400 dark:text-amber-500 shrink-0"
                  style="font-size: 14px">
                {if node_id.is_some() { "star" } else { "link" }}
            </span>

            // Label — clickable
            <button
                class="flex-1 text-left text-sm text-stone-700 dark:text-stone-300 truncate cursor-pointer py-1"
                title=label.clone()
                on:click=move |_| handle_click()
            >
                {label2}
            </button>

            // Controls (visible on hover)
            <div class="hidden group-hover:flex items-center gap-0.5 shrink-0">
                <button
                    class="p-0.5 text-stone-400 hover:text-stone-600 dark:hover:text-stone-300 cursor-pointer
                           disabled:opacity-30 disabled:cursor-default"
                    title="Move up"
                    disabled=is_first
                    on:click=move |_| on_move_up.run(fav_id)
                >
                    <span class="material-symbols-outlined" style="font-size: 14px">"keyboard_arrow_up"</span>
                </button>
                <button
                    class="p-0.5 text-stone-400 hover:text-stone-600 dark:hover:text-stone-300 cursor-pointer
                           disabled:opacity-30 disabled:cursor-default"
                    title="Move down"
                    disabled=is_last
                    on:click=move |_| on_move_down.run(fav_id)
                >
                    <span class="material-symbols-outlined" style="font-size: 14px">"keyboard_arrow_down"</span>
                </button>
                <button
                    class="p-0.5 text-stone-400 hover:text-red-500 cursor-pointer"
                    title="Remove from Favorites"
                    on:click=move |_| on_delete.run(fav_id)
                >
                    <span class="material-symbols-outlined" style="font-size: 14px">"delete"</span>
                </button>
            </div>
        </div>
    }
}
