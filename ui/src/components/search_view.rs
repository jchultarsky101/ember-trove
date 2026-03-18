use common::{search::SearchResponse, tag::Tag};
use leptos::prelude::*;

use crate::app::View;
use crate::components::node_meta::{status_color, status_icon, status_label, type_icon, type_label};

/// Full-page search results view.
///
/// Tag filtering is managed locally: `tag_filters` holds zero or more `Tag`
/// values; `tag_op_and` toggles between OR (default) and AND semantics.
/// The AND/OR toggle only appears once two or more tags are selected.
///
/// The query comes from the shared `search_query` context written by the
/// sidebar `SearchBar`.
#[component]
pub fn SearchView() -> impl IntoView {
    let current_view = use_context::<RwSignal<View>>().expect("View signal must be provided");
    let search_query =
        use_context::<RwSignal<String>>().expect("search_query signal must be provided");
    // Initialise from global single-tag context (set by NodeList chip clicks),
    // then manage locally — no reactive subscription to the global signal.
    let global_tag_filter =
        use_context::<RwSignal<Option<Tag>>>().expect("tag_filter signal must be provided");
    let init_tags: Vec<Tag> = global_tag_filter.get_untracked().into_iter().collect();

    let tag_filters: RwSignal<Vec<Tag>> = RwSignal::new(init_tags);
    // false = OR (default), true = AND
    let tag_op_and = RwSignal::new(false);

    let fuzzy = RwSignal::new(false);
    let published_only = RwSignal::new(false);
    let page = RwSignal::new(1u32);
    let error_msg = RwSignal::new(Option::<String>::None);
    let results: RwSignal<Option<SearchResponse>> = RwSignal::new(None);
    let loading = RwSignal::new(false);
    let search_version = RwSignal::new(0u32);

    // Fetch all tags once for the picker.
    let all_tags: LocalResource<Vec<Tag>> = LocalResource::new(|| async {
        crate::api::fetch_tags().await.unwrap_or_default()
    });

    let do_search = move || {
        let q = search_query.get_untracked().trim().to_string();
        loading.set(true);
        error_msg.set(None);
        let is_fuzzy = fuzzy.get_untracked();
        let status = if published_only.get_untracked() {
            Some("published")
        } else {
            None
        };
        let tag_ids: Vec<uuid::Uuid> = tag_filters
            .get_untracked()
            .iter()
            .map(|t| t.id.0)
            .collect();
        let tag_op = if tag_op_and.get_untracked() { "and" } else { "or" };
        let current_page = page.get_untracked();
        search_version.update(|v| *v += 1);
        let version = search_version.get_untracked();

        wasm_bindgen_futures::spawn_local(async move {
            gloo_timers::future::TimeoutFuture::new(300).await;
            if search_version.get_untracked() != version {
                return;
            }
            match crate::api::search_nodes(&q, is_fuzzy, status, &tag_ids, tag_op, current_page, 20).await {
                Ok(resp) => {
                    if search_version.get_untracked() == version {
                        results.set(Some(resp));
                    }
                }
                Err(e) => {
                    if search_version.get_untracked() == version {
                        error_msg.set(Some(format!("{e}")));
                    }
                }
            }
            if search_version.get_untracked() == version {
                loading.set(false);
            }
        });
    };

    // Auto-search when query, options, or tag filters change.
    Effect::new(move |_| {
        let _q = search_query.get();
        let _f = fuzzy.get();
        let _p = published_only.get();
        let _t = tag_filters.get();
        let _o = tag_op_and.get();
        page.set(1);
        do_search();
    });

    let on_page_change = move |new_page: u32| {
        page.set(new_page);
        do_search();
    };

    view! {
        <div class="flex flex-col h-full">
            // Filter bar
            <div class="flex items-center gap-2 px-6 py-3 border-b border-stone-200 dark:border-stone-800
                bg-white dark:bg-stone-900 flex-wrap">
                <h1 class="text-lg font-semibold text-stone-900 dark:text-stone-100 shrink-0">"Search"</h1>

                // AND/OR toggle — only when 2+ tags selected
                {move || {
                    if tag_filters.get().len() >= 2 {
                        let label = if tag_op_and.get() { "AND" } else { "OR" };
                        Some(view! {
                            <button
                                class="px-2 py-0.5 text-xs font-semibold rounded border
                                    border-amber-400 text-amber-500 dark:text-amber-400
                                    hover:bg-amber-50 dark:hover:bg-amber-900/20 transition-colors shrink-0"
                                title="Toggle AND / OR between tags"
                                on:click=move |_| tag_op_and.update(|v| *v = !*v)
                            >
                                {label}
                            </button>
                        })
                    } else {
                        None
                    }
                }}

                // Active tag chips
                <Suspense fallback=|| ()>
                    {move || all_tags.get().map(|_tags| {
                        let chips = tag_filters.get();
                        chips.into_iter().map(|tag| {
                            let name = tag.name.clone();
                            let color = tag.color.clone();
                            let tag_id = tag.id;
                            view! {
                                <button
                                    class="flex items-center gap-1 px-2 py-0.5 text-xs rounded-full
                                        text-white hover:opacity-80 transition-opacity shrink-0"
                                    style=format!("background-color: {color}")
                                    on:click=move |_| {
                                        tag_filters.update(|v| v.retain(|t| t.id != tag_id));
                                    }
                                    title="Remove tag filter"
                                >
                                    {name}
                                    " \u{00d7}"
                                </button>
                            }
                        }).collect::<Vec<_>>()
                    })}
                </Suspense>

                // Custom tag picker — replaces native <select> which is unreliable in CSR/dark mode
                <TagPicker all_tags=all_tags tag_filters=tag_filters />

                <div class="flex items-center gap-3 ml-auto">
                    {move || loading.get().then_some(view! {
                        <span class="text-xs text-stone-400 dark:text-stone-500 animate-pulse">"Searching\u{2026}"</span>
                    })}
                    <label class="flex items-center gap-1.5 text-sm text-stone-600 dark:text-stone-400 cursor-pointer select-none">
                        <input
                            type="checkbox"
                            class="rounded border-stone-300 dark:border-stone-600 text-amber-500
                                focus:ring-amber-500 dark:bg-stone-700"
                            prop:checked=move || fuzzy.get()
                            on:change=move |_| fuzzy.update(|f| *f = !*f)
                        />
                        "Fuzzy"
                    </label>
                    <label class="flex items-center gap-1.5 text-sm text-stone-600 dark:text-stone-400 cursor-pointer select-none">
                        <input
                            type="checkbox"
                            class="rounded border-stone-300 dark:border-stone-600 text-green-500
                                focus:ring-green-500 dark:bg-stone-700"
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
                            <div class="mb-3 text-sm text-stone-500 dark:text-stone-400">
                                {format!("{total} result{}", if total == 1 { "" } else { "s" })}
                                {if total_pages > 1 {
                                    format!(" \u{00b7} page {current_page} of {total_pages}")
                                } else {
                                    String::new()
                                }}
                            </div>

                            {if resp.results.is_empty() && total == 0 {
                                view! {
                                    <div class="text-center py-12 text-stone-400 dark:text-stone-600">
                                        <span class="material-symbols-outlined text-4xl mb-2 block">"search_off"</span>
                                        <p>"No results found. Try different keywords, tags, or enable fuzzy search."</p>
                                    </div>
                                }.into_any()
                            } else {
                                view! {
                                    <div class="space-y-3">
                                        {resp.results.into_iter().map(|result| {
                                            let node_id = result.node_id;
                                            let rank_pct = (result.rank * 100.0).min(100.0);
                                            let nt = result.node_type.clone();
                                            let st = result.status.clone();
                                            let t_icon  = type_icon(&nt);
                                            let t_label = type_label(&nt);
                                            let s_icon  = status_icon(&st);
                                            let s_label = status_label(&st);
                                            let s_color = status_color(&st);

                                            view! {
                                                <button
                                                    class="w-full text-left block p-4 rounded-lg border border-stone-200
                                                        dark:border-stone-700 hover:border-amber-300 dark:hover:border-amber-700
                                                        bg-white dark:bg-stone-900 transition-colors"
                                                    on:click=move |_| {
                                                        current_view.set(View::NodeDetail(node_id));
                                                    }
                                                >
                                                    // Row 1: type icon · title · status icon · rank
                                                    <div class="flex items-center gap-2 mb-1">
                                                        <span
                                                            class="material-symbols-outlined text-stone-400
                                                                   dark:text-stone-500 flex-shrink-0"
                                                            style="font-size: 15px;"
                                                            title=t_label
                                                        >
                                                            {t_icon}
                                                        </span>
                                                        <h3 class="text-sm font-medium text-stone-900 dark:text-stone-100 truncate flex-1">
                                                            {result.title}
                                                        </h3>
                                                        <span
                                                            class="material-symbols-outlined flex-shrink-0"
                                                            style=format!("font-size: 14px; {s_color}")
                                                            title=s_label
                                                        >
                                                            {s_icon}
                                                        </span>
                                                        <span class="text-xs text-stone-400 dark:text-stone-500 shrink-0">
                                                            {format!("{rank_pct:.0}%")}
                                                        </span>
                                                    </div>
                                                    // Row 2: slug
                                                    <p class="text-xs text-stone-500 dark:text-stone-400 mb-1 font-mono">
                                                        {result.slug}
                                                    </p>
                                                    // Row 3: snippet (optional)
                                                    {result.snippet.map(|s| view! {
                                                        <p
                                                            class="text-xs text-stone-600 dark:text-stone-400 line-clamp-2"
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

                // Loading state on first render
                {move || {
                    if results.get().is_none() && loading.get() {
                        Some(view! {
                            <div class="text-center py-16 text-stone-400 dark:text-stone-600">
                                <span class="text-sm animate-pulse">"Loading\u{2026}"</span>
                            </div>
                        }.into_any())
                    } else {
                        None
                    }
                }}
            </div>
        </div>
    }
}

// ── Custom tag picker ─────────────────────────────────────────────────────────

/// Replaces the native `<select>` for adding tag filters.
///
/// Renders a button that opens a custom dropdown listing unselected tags.
/// A click-away backdrop closes the menu without selecting anything.
#[component]
fn TagPicker(
    all_tags: LocalResource<Vec<Tag>>,
    tag_filters: RwSignal<Vec<Tag>>,
) -> impl IntoView {
    let open = RwSignal::new(false);

    view! {
        <div class="relative">
            <button
                class="flex items-center gap-1 px-2 py-0.5 text-xs rounded-md
                       border border-stone-300 dark:border-stone-600
                       bg-white dark:bg-stone-800
                       text-stone-500 dark:text-stone-400
                       hover:border-amber-400 dark:hover:border-amber-500
                       hover:text-stone-700 dark:hover:text-stone-200
                       transition-colors"
                on:click=move |_| open.update(|v| *v = !*v)
            >
                <span class="material-symbols-outlined" style="font-size: 13px;">"label"</span>
                "+ Add tag filter"
                <span class="material-symbols-outlined" style="font-size: 13px;">
                    {move || if open.get() { "expand_less" } else { "expand_more" }}
                </span>
            </button>

            {move || open.get().then(|| {
                let tags_now = all_tags.get().unwrap_or_default();
                let active_ids: Vec<_> = tag_filters.get().iter().map(|t| t.id).collect();
                let available: Vec<Tag> = tags_now
                    .into_iter()
                    .filter(|t| !active_ids.contains(&t.id))
                    .collect();

                view! {
                    // Click-away backdrop
                    <div
                        class="fixed inset-0 z-10"
                        on:click=move |_| open.set(false)
                    />
                    <div class="absolute left-0 top-full mt-1 z-20 min-w-36
                                bg-white dark:bg-stone-900 rounded-xl shadow-xl
                                border border-stone-200 dark:border-stone-700 overflow-hidden">
                        {if available.is_empty() {
                            view! {
                                <p class="px-3 py-2 text-xs text-stone-400 dark:text-stone-500 italic">
                                    "All tags selected"
                                </p>
                            }.into_any()
                        } else {
                            view! {
                                <div>
                                    {available.into_iter().map(|tag| {
                                        let color = tag.color.clone();
                                        let name = tag.name.clone();
                                        view! {
                                            <button
                                                class="w-full text-left flex items-center gap-2 px-3 py-2 text-xs
                                                       text-stone-700 dark:text-stone-300
                                                       hover:bg-stone-50 dark:hover:bg-stone-800
                                                       transition-colors"
                                                on:click=move |_| {
                                                    let t = tag.clone();
                                                    tag_filters.update(|v| {
                                                        if !v.iter().any(|x| x.id == t.id) {
                                                            v.push(t);
                                                        }
                                                    });
                                                    open.set(false);
                                                }
                                            >
                                                <span
                                                    class="w-2.5 h-2.5 rounded-full flex-shrink-0"
                                                    style=format!("background-color: {color}")
                                                />
                                                {name}
                                            </button>
                                        }
                                    }).collect::<Vec<_>>()}
                                </div>
                            }.into_any()
                        }}
                    </div>
                }
            })}
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
                class="px-3 py-1.5 text-xs rounded-md border border-stone-300 dark:border-stone-600
                    text-stone-600 dark:text-stone-400 hover:bg-stone-100 dark:hover:bg-stone-800
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
                                "bg-amber-600 text-white font-medium"
                            } else {
                                "border border-stone-300 dark:border-stone-600 text-stone-600 dark:text-stone-400 hover:bg-stone-100 dark:hover:bg-stone-800"
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
                class="px-3 py-1.5 text-xs rounded-md border border-stone-300 dark:border-stone-600
                    text-stone-600 dark:text-stone-400 hover:bg-stone-100 dark:hover:bg-stone-800
                    disabled:opacity-30 disabled:cursor-not-allowed transition-colors"
                prop:disabled=next_disabled
                on:click=on_next
            >
                "Next \u{2192}"
            </button>
        </div>
    }
}
