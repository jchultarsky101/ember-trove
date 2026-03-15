use common::{id::TagId, node::Node};
use leptos::prelude::*;

use crate::app::View;


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

#[component]
pub fn NodeList() -> impl IntoView {
    let current_view = use_context::<RwSignal<View>>().expect("View signal must be provided");
    let refresh = use_context::<RwSignal<u32>>().expect("refresh signal must be provided");
    // tag_filter: provided by App; None = no filter
    let tag_filter =
        use_context::<RwSignal<Option<TagId>>>().unwrap_or_else(|| RwSignal::new(None));

    // None = "All", Some("draft") | Some("published") | Some("archived")
    let status_filter: RwSignal<Option<String>> = RwSignal::new(None);

    let nodes = LocalResource::new(move || {
        let _ = refresh.get();
        let status = status_filter.get();
        let tag = tag_filter.get();
        async move {
            crate::api::fetch_nodes_filtered(status.as_deref(), tag.map(|t| t.0)).await
        }
    });

    view! {
        <div class="flex flex-col h-full">
            <div class="flex items-center justify-between px-6 py-4 border-b border-gray-200 dark:border-gray-800">
                <h1 class="text-lg font-semibold text-gray-900 dark:text-gray-100">"Nodes"</h1>
                <div class="flex items-center gap-2">
                    // Active tag-filter badge with dismiss
                    {move || tag_filter.get().map(|_| view! {
                        <button
                            class="flex items-center gap-1 px-2 py-0.5 text-xs rounded-full
                                bg-blue-100 text-blue-700 dark:bg-blue-900 dark:text-blue-300
                                hover:bg-blue-200 dark:hover:bg-blue-800 transition-colors"
                            on:click=move |_| tag_filter.set(None)
                            title="Clear tag filter"
                        >
                            <span class="material-symbols-outlined" style="font-size:12px;">"label"</span>
                            "\u{00d7} tag"
                        </button>
                    })}
                    <button
                        class="p-1.5 rounded-lg text-gray-400 hover:text-gray-600 dark:hover:text-gray-300
                            hover:bg-gray-100 dark:hover:bg-gray-800 transition-colors"
                        on:click=move |_| current_view.set(View::NodeCreate)
                        title="New node"
                    >
                        <span class="material-symbols-outlined">"add"</span>
                    </button>
                </div>
            </div>

            // Status filter pills
            <div class="flex gap-1 px-6 py-2 border-b border-gray-100 dark:border-gray-800">
                {[("All", None), ("Draft", Some("draft")), ("Published", Some("published")), ("Archived", Some("archived"))].iter().map(|&(label, value)| {
                    let value_owned: Option<String> = value.map(|s| s.to_string());
                    let value_cmp = value_owned.clone();
                    view! {
                        <button
                            class=move || {
                                let active = status_filter.get() == value_cmp;
                                let base = "px-2.5 py-0.5 text-xs rounded-full font-medium transition-colors";
                                if active {
                                    format!("{base} bg-blue-600 text-white")
                                } else {
                                    format!("{base} bg-gray-100 dark:bg-gray-800 text-gray-600 dark:text-gray-400 hover:bg-gray-200 dark:hover:bg-gray-700")
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
                    <div class="p-6 text-gray-400 text-sm">"Loading nodes..."</div>
                }>
                    {move || {
                        nodes.get().map(|result| {
                            match result {
                                Ok(list) if list.is_empty() => view! {
                                    <div class="flex flex-col items-center justify-center h-full gap-3 p-12">
                                        <span
                                            class="material-symbols-outlined text-gray-300 dark:text-gray-700"
                                            style="font-size: 48px;"
                                        >
                                            "description"
                                        </span>
                                        <p class="text-gray-400 dark:text-gray-600 text-sm text-center">
                                            "No nodes found."
                                        </p>
                                    </div>
                                }.into_any(),
                                Ok(list) => view! {
                                    <NodeCards nodes=list current_view=current_view />
                                }.into_any(),
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
    view! {
        <ul class="divide-y divide-gray-200 dark:divide-gray-800">
            {nodes.into_iter().map(|node| {
                let id = node.id;
                let node_type = format!("{:?}", node.node_type).to_lowercase();
                let status = format!("{:?}", node.status).to_lowercase();
                let updated = node.updated_at.format("%Y-%m-%d %H:%M").to_string();
                let tags = node.tags.clone();
                // Status-specific badge colour
                let status_class = match status.as_str() {
                    "published" => "bg-green-100 text-green-700 dark:bg-green-900 dark:text-green-300",
                    "archived"  => "bg-yellow-100 text-yellow-700 dark:bg-yellow-900 dark:text-yellow-300",
                    _           => "bg-gray-100 text-gray-600 dark:bg-gray-800 dark:text-gray-400",
                };
                view! {
                    <li
                        class="px-6 py-4 hover:bg-gray-100 dark:hover:bg-gray-900 cursor-pointer transition-colors"
                        on:click=move |_| current_view.set(View::NodeDetail(id))
                    >
                        <div class="flex items-start justify-between gap-3">
                            <div class="min-w-0 flex-1">
                                <div class="flex items-center gap-2 flex-wrap">
                                    <span class="text-sm font-medium text-gray-900 dark:text-gray-100">
                                        {node.title.clone()}
                                    </span>
                                    <span class="px-2 py-0.5 text-xs rounded-full bg-blue-100 text-blue-700 dark:bg-blue-900 dark:text-blue-300">
                                        {node_type}
                                    </span>
                                    <span class=format!("px-2 py-0.5 text-xs rounded-full {status_class}")>
                                        {status}
                                    </span>
                                    {tags.into_iter().map(|tag| view! {
                                        <span class="px-2 py-0.5 text-xs rounded-full bg-purple-100 text-purple-700 dark:bg-purple-900 dark:text-purple-300">
                                            {tag.name}
                                        </span>
                                    }).collect::<Vec<_>>()}
                                </div>
                                {node.body.as_deref().and_then(body_preview).map(|preview| view! {
                                    <p class="text-xs text-gray-500 dark:text-gray-400 mt-1 truncate">
                                        {preview}
                                    </p>
                                })}
                            </div>
                            <span class="text-xs text-gray-400 shrink-0 mt-0.5">{updated}</span>
                        </div>
                    </li>
                }
            }).collect::<Vec<_>>()}
        </ul>
    }
}
