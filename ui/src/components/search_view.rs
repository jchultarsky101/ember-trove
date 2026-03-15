use common::search::SearchResponse;
use leptos::prelude::*;

use crate::app::View;

/// Full-page search results view.
///
/// The query comes from the shared `search_query` context signal written by the
/// sidebar `SearchBar` — there is no duplicate input here. Results auto-update
/// whenever the sidebar query changes. The Fuzzy and Published checkboxes let
/// the user refine results without re-typing the query.
#[component]
pub fn SearchView() -> impl IntoView {
    let current_view = use_context::<RwSignal<View>>().expect("View signal must be provided");
    // Shared with SearchBar — sidebar is the single query input.
    let search_query =
        use_context::<RwSignal<String>>().expect("search_query signal must be provided");

    let fuzzy = RwSignal::new(false);
    let published_only = RwSignal::new(false);
    let page = RwSignal::new(1u32);
    let error_msg = RwSignal::new(Option::<String>::None);
    let results: RwSignal<Option<SearchResponse>> = RwSignal::new(None);
    let loading = RwSignal::new(false);

    let do_search = move || {
        let q = search_query.get_untracked().trim().to_string();
        if q.is_empty() {
            results.set(None);
            return;
        }
        loading.set(true);
        error_msg.set(None);
        let is_fuzzy = fuzzy.get_untracked();
        let status = if published_only.get_untracked() {
            Some("published")
        } else {
            None
        };
        let current_page = page.get_untracked();

        wasm_bindgen_futures::spawn_local(async move {
            match crate::api::search_nodes(&q, is_fuzzy, status, current_page, 20).await {
                Ok(resp) => {
                    results.set(Some(resp));
                }
                Err(e) => {
                    error_msg.set(Some(format!("{e}")));
                }
            }
            loading.set(false);
        });
    };

    // Auto-search when query changes or when options change.
    Effect::new(move |_| {
        let _q = search_query.get(); // subscribe
        let _f = fuzzy.get();
        let _p = published_only.get();
        page.set(1);
        do_search();
    });

    let on_page_change = move |new_page: u32| {
        page.set(new_page);
        do_search();
    };

    view! {
        <div class="flex flex-col h-full">
            // Compact filter bar — no duplicate search input
            <div class="flex items-center gap-4 px-6 py-3 border-b border-gray-200 dark:border-gray-800
                bg-white dark:bg-gray-900">
                <h1 class="text-lg font-semibold text-gray-900 dark:text-gray-100 shrink-0">"Search"</h1>
                <div class="flex items-center gap-3 ml-auto">
                    {move || loading.get().then_some(view! {
                        <span class="text-xs text-gray-400 dark:text-gray-500 animate-pulse">"Searching\u{2026}"</span>
                    })}
                    <label class="flex items-center gap-1.5 text-sm text-gray-600 dark:text-gray-400 cursor-pointer select-none">
                        <input
                            type="checkbox"
                            class="rounded border-gray-300 dark:border-gray-600 text-blue-500
                                focus:ring-blue-500 dark:bg-gray-700"
                            prop:checked=move || fuzzy.get()
                            on:change=move |_| fuzzy.update(|f| *f = !*f)
                        />
                        "Fuzzy"
                    </label>
                    <label class="flex items-center gap-1.5 text-sm text-gray-600 dark:text-gray-400 cursor-pointer select-none">
                        <input
                            type="checkbox"
                            class="rounded border-gray-300 dark:border-gray-600 text-green-500
                                focus:ring-green-500 dark:bg-gray-700"
                            prop:checked=move || published_only.get()
                            on:change=move |_| published_only.update(|v| *v = !*v)
                        />
                        "Published"
                    </label>
                </div>
            </div>

            // Results area
            <div class="flex-1 overflow-y-auto px-6 py-4">
                {move || error_msg.get().map(|msg| view! {
                    <div class="mb-4 px-4 py-3 rounded-lg bg-red-50 dark:bg-red-900/20 text-red-600 dark:text-red-400 text-sm">
                        {msg}
                    </div>
                })}

                {move || {
                    results.get().map(|resp| {
                        let total = resp.total;
                        let current_page = resp.page;
                        let per_page = resp.per_page;
                        let total_pages = ((total as f64) / (per_page as f64)).ceil() as u32;

                        view! {
                            <div class="mb-3 text-sm text-gray-500 dark:text-gray-400">
                                {format!("{total} result{}", if total == 1 { "" } else { "s" })}
                                {if total_pages > 1 {
                                    format!(" \u{00b7} page {current_page} of {total_pages}")
                                } else {
                                    String::new()
                                }}
                            </div>

                            {if resp.results.is_empty() && total == 0 {
                                view! {
                                    <div class="text-center py-12 text-gray-400 dark:text-gray-600">
                                        <span class="material-symbols-outlined text-4xl mb-2 block">"search_off"</span>
                                        <p>"No results found. Try different keywords or enable fuzzy search."</p>
                                    </div>
                                }.into_any()
                            } else {
                                view! {
                                    <div class="space-y-3">
                                        {resp.results.into_iter().map(|result| {
                                            let node_id = result.node_id;
                                            let rank_pct = (result.rank * 100.0).min(100.0);

                                            view! {
                                                <button
                                                    class="w-full text-left block p-4 rounded-lg border border-gray-200
                                                        dark:border-gray-700 hover:border-blue-300 dark:hover:border-blue-700
                                                        bg-white dark:bg-gray-900 transition-colors"
                                                    on:click=move |_| {
                                                        current_view.set(View::NodeDetail(node_id));
                                                    }
                                                >
                                                    <div class="flex items-center justify-between mb-1">
                                                        <h3 class="text-sm font-medium text-gray-900 dark:text-gray-100">
                                                            {result.title}
                                                        </h3>
                                                        <span class="text-xs text-gray-400 dark:text-gray-500 ml-2 shrink-0">
                                                            {format!("{rank_pct:.0}%")}
                                                        </span>
                                                    </div>
                                                    <p class="text-xs text-gray-500 dark:text-gray-400 mb-1 font-mono">
                                                        {result.slug}
                                                    </p>
                                                    {result.snippet.map(|s| view! {
                                                        <p
                                                            class="text-xs text-gray-600 dark:text-gray-400 line-clamp-2"
                                                            inner_html=s
                                                        />
                                                    })}
                                                </button>
                                            }
                                        }).collect::<Vec<_>>()}
                                    </div>

                                    {if total_pages > 1 {
                                        Some(view! {
                                            <Pagination
                                                current_page=current_page
                                                total_pages=total_pages
                                                on_page=on_page_change
                                            />
                                        })
                                    } else {
                                        None
                                    }}
                                }.into_any()
                            }}
                        }
                    })
                }}

                // Empty state — before any search or when query is empty
                {move || {
                    if results.get().is_none() && !loading.get() {
                        let q = search_query.get();
                        Some(if q.trim().is_empty() {
                            view! {
                                <div class="text-center py-16 text-gray-400 dark:text-gray-600">
                                    <span class="material-symbols-outlined text-5xl mb-3 block">"manage_search"</span>
                                    <p class="text-lg mb-1">"Search your knowledge base"</p>
                                    <p class="text-sm">"Type in the search bar on the left to find nodes."</p>
                                </div>
                            }.into_any()
                        } else {
                            view! { <div /> }.into_any()
                        })
                    } else {
                        None
                    }
                }}
            </div>
        </div>
    }
}

#[component]
fn Pagination(
    current_page: u32,
    total_pages: u32,
    #[prop(into)] on_page: Callback<u32>,
) -> impl IntoView {
    let pages: Vec<u32> = {
        let mut p = Vec::new();
        let start = current_page.saturating_sub(2).max(1);
        let end = (start + 4).min(total_pages);
        let start = end.saturating_sub(4).max(1);
        for i in start..=end {
            p.push(i);
        }
        p
    };

    let prev_disabled = current_page <= 1;
    let next_disabled = current_page >= total_pages;

    let on_prev = move |_: web_sys::MouseEvent| {
        on_page.run(current_page - 1);
    };

    let on_next = move |_: web_sys::MouseEvent| {
        on_page.run(current_page + 1);
    };

    view! {
        <div class="flex items-center justify-center gap-1 mt-6">
            <button
                class="px-3 py-1.5 text-xs rounded-md border border-gray-300 dark:border-gray-600
                    text-gray-600 dark:text-gray-400 hover:bg-gray-100 dark:hover:bg-gray-800
                    disabled:opacity-30 disabled:cursor-not-allowed transition-colors"
                prop:disabled=prev_disabled
                on:click=on_prev
            >
                "\u{2190} Prev"
            </button>

            {pages.into_iter().map(|p| {
                let is_current = p == current_page;
                view! {
                    <button
                        class=move || format!(
                            "px-3 py-1.5 text-xs rounded-md transition-colors {}",
                            if is_current {
                                "bg-blue-600 text-white font-medium"
                            } else {
                                "border border-gray-300 dark:border-gray-600 text-gray-600 dark:text-gray-400 hover:bg-gray-100 dark:hover:bg-gray-800"
                            }
                        )
                        on:click=move |_| {
                            if !is_current {
                                on_page.run(p);
                            }
                        }
                    >
                        {p.to_string()}
                    </button>
                }
            }).collect::<Vec<_>>()}

            <button
                class="px-3 py-1.5 text-xs rounded-md border border-gray-300 dark:border-gray-600
                    text-gray-600 dark:text-gray-400 hover:bg-gray-100 dark:hover:bg-gray-800
                    disabled:opacity-30 disabled:cursor-not-allowed transition-colors"
                prop:disabled=next_disabled
                on:click=on_next
            >
                "Next \u{2192}"
            </button>
        </div>
    }
}
