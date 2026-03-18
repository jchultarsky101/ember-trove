use common::{node::Node, tag::Tag};
use leptos::prelude::*;

use crate::app::View;

// ── Sorting ────────────────────────────────────────────────────────────────────

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum SortKey {
    NameAsc,
    NameDesc,
    ModifiedDesc,
    ModifiedAsc,
}

impl SortKey {
    fn label(self) -> &'static str {
        match self {
            SortKey::NameAsc      => "Name A→Z",
            SortKey::NameDesc     => "Name Z→A",
            SortKey::ModifiedDesc => "Newest first",
            SortKey::ModifiedAsc  => "Oldest first",
        }
    }

    fn icon(self) -> &'static str {
        match self {
            SortKey::NameAsc      => "sort_by_alpha",
            SortKey::NameDesc     => "sort_by_alpha",
            SortKey::ModifiedDesc => "schedule",
            SortKey::ModifiedAsc  => "schedule",
        }
    }

    fn sort_nodes(self, mut nodes: Vec<Node>) -> Vec<Node> {
        match self {
            SortKey::NameAsc      => nodes.sort_by(|a, b| a.title.to_lowercase().cmp(&b.title.to_lowercase())),
            SortKey::NameDesc     => nodes.sort_by(|a, b| b.title.to_lowercase().cmp(&a.title.to_lowercase())),
            SortKey::ModifiedDesc => nodes.sort_by(|a, b| b.updated_at.cmp(&a.updated_at)),
            SortKey::ModifiedAsc  => nodes.sort_by(|a, b| a.updated_at.cmp(&b.updated_at)),
        }
        nodes
    }
}

const SORT_OPTIONS: &[SortKey] = &[
    SortKey::ModifiedDesc,
    SortKey::ModifiedAsc,
    SortKey::NameAsc,
    SortKey::NameDesc,
];

// ── Preview helper ─────────────────────────────────────────────────────────────

/// Strip basic Markdown and return up to 120 chars as a plain-text preview.
fn body_preview(body: &str) -> Option<String> {
    let text: String = body
        .lines()
        .map(str::trim)
        .filter(|l| {
            !l.is_empty()
                && !l.starts_with('#')
                && !l.starts_with("```")
                && !l.starts_with("---")
                && !l.starts_with("===")
        })
        .collect::<Vec<_>>()
        .join(" ");
    if text.is_empty() {
        return None;
    }
    let clean = text.replace("**", "").replace("__", "").replace('`', "");
    let chars: Vec<char> = clean.chars().collect();
    if chars.is_empty() {
        return None;
    }
    let preview: String = chars.iter().take(120).collect();
    Some(if chars.len() > 120 {
        format!("{preview}\u{2026}")
    } else {
        preview
    })
}

// ── NodeList ───────────────────────────────────────────────────────────────────

#[component]
pub fn NodeList() -> impl IntoView {
    let current_view = use_context::<RwSignal<View>>().expect("View signal must be provided");
    let refresh = use_context::<RwSignal<u32>>().expect("refresh signal must be provided");
    let tag_filter =
        use_context::<RwSignal<Option<Tag>>>().unwrap_or_else(|| RwSignal::new(None));

    // None = "All", Some("draft") | Some("published") | Some("archived")
    let status_filter: RwSignal<Option<String>> = RwSignal::new(None);
    let sort_key = RwSignal::new(SortKey::ModifiedDesc);
    let show_sort_menu = RwSignal::new(false);

    let nodes = LocalResource::new(move || {
        let _ = refresh.get();
        let status = status_filter.get();
        let tag = tag_filter.get();
        async move {
            crate::api::fetch_nodes_filtered(status.as_deref(), tag.map(|t| t.id.0)).await
        }
    });

    view! {
        <div class="flex flex-col h-full">
            <div class="flex items-center justify-between px-6 py-4 border-b border-stone-200 dark:border-stone-800">
                <h1 class="text-lg font-semibold text-stone-900 dark:text-stone-100">"Nodes"</h1>
                <div class="flex items-center gap-2">
                    // Active tag-filter badge
                    {move || tag_filter.get().map(|tag| {
                        let name = tag.name.clone();
                        let color = tag.color.clone();
                        view! {
                            <button
                                class="flex items-center gap-1 px-2 py-0.5 text-xs rounded-full text-white
                                    transition-colors hover:opacity-80"
                                style=format!("background-color: {color}")
                                on:click=move |_| tag_filter.set(None)
                                title="Clear tag filter"
                            >
                                <span class="material-symbols-outlined" style="font-size:11px;">"label"</span>
                                {name}
                                " \u{00d7}"
                            </button>
                        }
                    })}

                    // Sort picker
                    <div class="relative">
                        <button
                            class="flex items-center gap-1 p-1.5 rounded-lg text-stone-400
                                hover:text-stone-600 dark:hover:text-stone-300
                                hover:bg-stone-100 dark:hover:bg-stone-800 transition-colors"
                            title=move || format!("Sort: {}", sort_key.get().label())
                            on:click=move |_| show_sort_menu.update(|v| *v = !*v)
                        >
                            <span class="material-symbols-outlined" style="font-size: 18px;">
                                {move || sort_key.get().icon()}
                            </span>
                        </button>
                        {move || show_sort_menu.get().then(|| view! {
                            // Click-away backdrop
                            <div
                                class="fixed inset-0 z-10"
                                on:click=move |_| show_sort_menu.set(false)
                            />
                            <div class="absolute right-0 top-full mt-1 z-20 w-40
                                bg-white dark:bg-stone-900 rounded-xl shadow-xl
                                border border-stone-200 dark:border-stone-700 overflow-hidden">
                                {SORT_OPTIONS.iter().map(|&opt| {
                                    view! {
                                        <button
                                            class=move || {
                                                let active = sort_key.get() == opt;
                                                let base = "w-full text-left px-3 py-2 text-xs flex items-center gap-2 transition-colors";
                                                if active {
                                                    format!("{base} bg-amber-50 dark:bg-amber-900/20 text-amber-700 dark:text-amber-400 font-medium")
                                                } else {
                                                    format!("{base} text-stone-700 dark:text-stone-300 hover:bg-stone-50 dark:hover:bg-stone-800")
                                                }
                                            }
                                            on:click=move |_| {
                                                sort_key.set(opt);
                                                show_sort_menu.set(false);
                                            }
                                        >
                                            <span class="material-symbols-outlined" style="font-size: 14px;">
                                                {opt.icon()}
                                            </span>
                                            {opt.label()}
                                        </button>
                                    }
                                }).collect::<Vec<_>>()}
                            </div>
                        })}
                    </div>

                    <button
                        class="p-1.5 rounded-lg text-stone-400 hover:text-stone-600 dark:hover:text-stone-300
                            hover:bg-stone-100 dark:hover:bg-stone-800 transition-colors"
                        on:click=move |_| current_view.set(View::NodeCreate)
                        title="New node"
                    >
                        <span class="material-symbols-outlined">"add"</span>
                    </button>
                </div>
            </div>

            // Status filter pills
            <div class="flex gap-1 px-6 py-2 border-b border-stone-100 dark:border-stone-800">
                {[("All", None), ("Draft", Some("draft")), ("Published", Some("published")), ("Archived", Some("archived"))].iter().map(|&(label, value)| {
                    let value_owned: Option<String> = value.map(|s| s.to_string());
                    let value_cmp = value_owned.clone();
                    view! {
                        <button
                            class=move || {
                                let active = status_filter.get() == value_cmp;
                                let base = "px-2.5 py-0.5 text-xs rounded-full font-medium transition-colors";
                                if active {
                                    format!("{base} bg-amber-600 text-white")
                                } else {
                                    format!("{base} bg-stone-100 dark:bg-stone-800 text-stone-600 dark:text-stone-400 hover:bg-stone-200 dark:hover:bg-stone-700")
                                }
                            }
                            on:click={
                                let value_set = value_owned.clone();
                                move |_| status_filter.set(value_set.clone())
                            }
                        >
                            {label}
                        </button>
                    }
                }).collect::<Vec<_>>()}
            </div>

            <div class="flex-1 overflow-auto">
                <Suspense fallback=move || view! {
                    // Skeleton cards
                    <ul class="divide-y divide-stone-200 dark:divide-stone-800">
                        {(0..6).map(|_| view! {
                            <li class="px-6 py-4">
                                <div class="flex items-start justify-between gap-3">
                                    <div class="min-w-0 flex-1 space-y-2">
                                        <div class="flex items-center gap-2">
                                            <div class="h-3.5 rounded bg-stone-200 dark:bg-stone-700 animate-pulse w-40" />
                                            <div class="h-4 rounded-full bg-stone-200 dark:bg-stone-700 animate-pulse w-14" />
                                        </div>
                                        <div class="h-2.5 rounded bg-stone-100 dark:bg-stone-800 animate-pulse w-64" />
                                    </div>
                                    <div class="h-2.5 rounded bg-stone-100 dark:bg-stone-800 animate-pulse w-20 shrink-0 mt-1" />
                                </div>
                            </li>
                        }).collect::<Vec<_>>()}
                    </ul>
                }>
                    {move || {
                        nodes.get().map(|result| {
                            match result {
                                Ok(list) if list.is_empty() => view! {
                                    <div class="flex flex-col items-center justify-center h-full gap-3 p-12">
                                        <span
                                            class="material-symbols-outlined text-stone-300 dark:text-stone-700"
                                            style="font-size: 48px;"
                                        >
                                            "description"
                                        </span>
                                        <p class="text-stone-400 dark:text-stone-600 text-sm text-center">
                                            "No nodes found."
                                        </p>
                                    </div>
                                }.into_any(),
                                Ok(list) => {
                                    let sorted = sort_key.get().sort_nodes(list);
                                    view! {
                                        <NodeCards nodes=sorted current_view=current_view />
                                    }.into_any()
                                }
                                Err(e) => view! {
                                    <div class="p-6 text-red-500 text-sm">
                                        {format!("Error: {e}")}
                                    </div>
                                }.into_any(),
                            }
                        })
                    }}
                </Suspense>
            </div>
        </div>
    }
}

#[component]
fn NodeCards(nodes: Vec<Node>, current_view: RwSignal<View>) -> impl IntoView {
    let tag_filter =
        use_context::<RwSignal<Option<Tag>>>().unwrap_or_else(|| RwSignal::new(None));

    view! {
        <ul class="divide-y divide-stone-200 dark:divide-stone-800">
            {nodes.into_iter().map(|node| {
                let id = node.id;
                let node_type = format!("{:?}", node.node_type).to_lowercase();
                let status = format!("{:?}", node.status).to_lowercase();
                let updated = node.updated_at.format("%Y-%m-%d %H:%M").to_string();
                let tags = node.tags.clone();
                let status_class = match status.as_str() {
                    "published" => "bg-green-100 text-green-700 dark:bg-green-900 dark:text-green-300",
                    "archived"  => "bg-yellow-100 text-yellow-700 dark:bg-yellow-900 dark:text-yellow-300",
                    _           => "bg-stone-100 text-stone-600 dark:bg-stone-800 dark:text-stone-400",
                };
                view! {
                    <li
                        class="px-6 py-4 hover:bg-stone-100 dark:hover:bg-stone-900 cursor-pointer transition-colors"
                        on:click=move |_| current_view.set(View::NodeDetail(id))
                    >
                        <div class="flex items-start justify-between gap-3">
                            <div class="min-w-0 flex-1">
                                <div class="flex items-center gap-2 flex-wrap">
                                    <span class="text-sm font-medium text-stone-900 dark:text-stone-100 truncate max-w-xs">
                                        {node.title.clone()}
                                    </span>
                                    <span class="px-2 py-0.5 text-xs rounded-full bg-amber-100 text-amber-700 dark:bg-amber-900 dark:text-amber-300 whitespace-nowrap">
                                        {node_type}
                                    </span>
                                    <span class=format!("px-2 py-0.5 text-xs rounded-full whitespace-nowrap {status_class}")>
                                        {status}
                                    </span>
                                    {tags.into_iter().map(|tag| {
                                        let tag_for_filter = tag.clone();
                                        let color = tag.color.clone();
                                        let name = tag.name.clone();
                                        let title = format!("Filter by tag: {name}");
                                        view! {
                                            <button
                                                class="px-2 py-0.5 text-xs rounded-full text-white
                                                    hover:opacity-80 transition-opacity whitespace-nowrap"
                                                style=format!("background-color: {color}")
                                                title=title
                                                on:click=move |ev| {
                                                    ev.stop_propagation();
                                                    tag_filter.set(Some(tag_for_filter.clone()));
                                                }
                                            >
                                                {name}
                                            </button>
                                        }
                                    }).collect::<Vec<_>>()}
                                </div>
                                {node.body.as_deref().and_then(body_preview).map(|preview| view! {
                                    <p class="text-xs text-stone-500 dark:text-stone-400 mt-1 truncate">
                                        {preview}
                                    </p>
                                })}
                            </div>
                            <span class="text-xs text-stone-400 shrink-0 mt-0.5 whitespace-nowrap">{updated}</span>
                        </div>
                    </li>
                }
            }).collect::<Vec<_>>()}
        </ul>
    }
}
