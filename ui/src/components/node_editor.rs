use std::collections::HashMap;

use common::{
    id::NodeId,
    node::{CreateNodeRequest, NodeStatus, NodeType, NodeTitleEntry, UpdateNodeRequest},
};
use leptos::prelude::*;
use pulldown_cmark::{Options, Parser, html};

use crate::app::View;
use crate::wikilink::preprocess_wikilinks;

fn render_markdown(source: &str, title_map: &HashMap<String, NodeId>) -> String {
    let preprocessed = preprocess_wikilinks(source, title_map);
    let opts = Options::ENABLE_STRIKETHROUGH | Options::ENABLE_TABLES | Options::ENABLE_TASKLISTS;
    let parser = Parser::new_ext(&preprocessed, opts);
    let mut html_out = String::new();
    html::push_html(&mut html_out, parser);
    ammonia::Builder::new()
        .add_tag_attributes("a", &["class", "data-node-id"])
        .add_tags(&["span"])
        .add_tag_attributes("span", &["class"])
        .clean(&html_out)
        .to_string()
}

fn build_title_map(entries: &[NodeTitleEntry]) -> HashMap<String, NodeId> {
    entries.iter().map(|e| (e.title.clone(), e.id)).collect()
}

fn parse_status(s: &str) -> NodeStatus {
    match s {
        "published" => NodeStatus::Published,
        "archived" => NodeStatus::Archived,
        _ => NodeStatus::Draft,
    }
}

/// Return the partial wiki-link query being typed at the cursor, if any.
///
/// Looks backwards from `cursor` for an unclosed `[[`. Returns the text
/// typed after `[[` up to the cursor, or `None` if the cursor is not inside
/// an open wiki-link context.
fn wikilink_query_at(text: &str, cursor: usize) -> Option<String> {
    let before = &text[..cursor.min(text.len())];
    // Find the last `[[` that has not been closed.
    let open = before.rfind("[[")?;
    let after_open = &before[open + 2..];
    // If there's already a closing `]]` or a newline between `[[` and cursor,
    // we are not in a wiki-link context.
    if after_open.contains("]]") || after_open.contains('\n') {
        return None;
    }
    Some(after_open.to_string())
}

#[component]
pub fn NodeEditor(node: Option<NodeId>) -> impl IntoView {
    let current_view = use_context::<RwSignal<View>>().expect("View signal must be provided");
    let refresh = use_context::<RwSignal<u32>>().expect("refresh signal must be provided");

    let title = RwSignal::new(String::new());
    let body = RwSignal::new(String::new());
    let node_type = RwSignal::new("article".to_string());
    let status = RwSignal::new("draft".to_string());
    let saving = RwSignal::new(false);
    let error_msg = RwSignal::new(Option::<String>::None);

    // Wiki-link autocomplete state.
    let wikilink_query = RwSignal::new(Option::<String>::None);
    let textarea_ref = NodeRef::<leptos::html::Textarea>::new();

    // Fetch all node titles for wiki-link autocomplete and preview.
    let titles_resource =
        LocalResource::new(|| async move { crate::api::fetch_node_titles().await });

    // If editing, fetch existing node data.
    if let Some(id) = node {
        wasm_bindgen_futures::spawn_local(async move {
            if let Ok(n) = crate::api::fetch_node(id).await {
                title.set(n.title);
                body.set(n.body.unwrap_or_default());
                node_type.set(format!("{:?}", n.node_type).to_lowercase());
                status.set(format!("{:?}", n.status).to_lowercase());
            }
        });
    }

    let on_save = move |_| {
        saving.set(true);
        error_msg.set(None);
        let t = title.get_untracked();
        let b = body.get_untracked();
        let nt_str = node_type.get_untracked();
        let st_str = status.get_untracked();

        wasm_bindgen_futures::spawn_local(async move {
            let result = if let Some(id) = node {
                let req = UpdateNodeRequest {
                    title: Some(t),
                    body: Some(b),
                    metadata: None,
                    status: Some(parse_status(&st_str)),
                };
                crate::api::update_node(id, &req).await
            } else {
                let nt = match nt_str.as_str() {
                    "project" => NodeType::Project,
                    "area" => NodeType::Area,
                    "resource" => NodeType::Resource,
                    "reference" => NodeType::Reference,
                    _ => NodeType::Article,
                };
                let req = CreateNodeRequest {
                    title: t,
                    node_type: nt,
                    body: Some(b),
                    metadata: serde_json::Value::Object(serde_json::Map::new()),
                    status: Some(parse_status(&st_str)),
                };
                crate::api::create_node(&req).await
            };

            match result {
                Ok(saved_node) => {
                    refresh.update(|n| *n += 1);
                    current_view.set(View::NodeDetail(saved_node.id));
                }
                Err(e) => {
                    error_msg.set(Some(format!("{e}")));
                }
            }
            saving.set(false);
        });
    };

    // Detect [[query at cursor on every keystroke.
    let on_body_input = move |ev: leptos::ev::Event| {
        let val = event_target_value(&ev);
        body.set(val.clone());

        let query = textarea_ref
            .get()
            .and_then(|el| el.selection_start().ok().flatten())
            .and_then(|cursor| wikilink_query_at(&val, cursor as usize));
        wikilink_query.set(query);
    };

    // Insert the selected title at the cursor, replacing the open [[query.
    let on_select_title = move |selected: String| {
        wikilink_query.set(None);
        let current = body.get_untracked();
        let cursor = textarea_ref
            .get()
            .and_then(|el| el.selection_start().ok().flatten())
            .unwrap_or(0) as usize;
        let before = &current[..cursor.min(current.len())];
        if let Some(open_pos) = before.rfind("[[") {
            let new_val = format!(
                "[[{}]]{}",
                selected,
                &current[cursor..],
            );
            let prefix = &current[..open_pos];
            let new_val = format!("{prefix}{new_val}");
            let new_cursor = open_pos + 2 + selected.len() + 2;
            body.set(new_val.clone());
            // Defer cursor placement until after Leptos re-renders the textarea.
            if let Some(el) = textarea_ref.get() {
                el.set_value(&new_val);
                let _ = el.set_selection_start(Some(new_cursor as u32));
                let _ = el.set_selection_end(Some(new_cursor as u32));
                let _ = el.focus();
            }
        }
    };

    let preview_html = move || {
        let title_map = titles_resource
            .get()
            .and_then(|r| r.ok())
            .map(|entries| build_title_map(&entries))
            .unwrap_or_default();
        render_markdown(&body.get(), &title_map)
    };

    view! {
        <div class="flex flex-col h-full">
            // Header
            <div class="flex items-center justify-between px-6 py-4 border-b border-gray-200 dark:border-gray-800">
                <div class="flex items-center gap-3 flex-1">
                    <button
                        class="text-gray-400 hover:text-gray-600 dark:hover:text-gray-300"
                        on:click=move |_| current_view.set(View::NodeList)
                    >
                        <span class="material-symbols-outlined">"arrow_back"</span>
                    </button>
                    <input
                        type="text"
                        class="flex-1 text-lg font-semibold bg-transparent text-gray-900 dark:text-gray-100
                            focus:outline-none placeholder-gray-400"
                        placeholder="Node title..."
                        prop:value=move || title.get()
                        on:input=move |ev| title.set(event_target_value(&ev))
                    />
                </div>
                <div class="flex items-center gap-2">
                    <select
                        class="text-sm bg-gray-100 dark:bg-gray-800 text-gray-700 dark:text-gray-300
                            rounded-lg px-2 py-1.5 focus:outline-none"
                        prop:value=move || node_type.get()
                        on:change=move |ev| node_type.set(event_target_value(&ev))
                    >
                        <option value="article">"Article"</option>
                        <option value="project">"Project"</option>
                        <option value="area">"Area"</option>
                        <option value="resource">"Resource"</option>
                        <option value="reference">"Reference"</option>
                    </select>
                    <select
                        class="text-sm bg-gray-100 dark:bg-gray-800 text-gray-700 dark:text-gray-300
                            rounded-lg px-2 py-1.5 focus:outline-none"
                        prop:value=move || status.get()
                        on:change=move |ev| status.set(event_target_value(&ev))
                    >
                        <option value="draft">"Draft"</option>
                        <option value="published">"Published"</option>
                        <option value="archived">"Archived"</option>
                    </select>
                    <button
                        class="p-1.5 rounded-lg text-gray-400 hover:text-green-600 dark:hover:text-green-400
                            hover:bg-green-50 dark:hover:bg-green-900/30 transition-colors"
                        on:click=on_save
                        disabled=move || saving.get()
                        title=move || if saving.get() { "Saving\u{2026}" } else { "Save" }
                    >
                        <span class="material-symbols-outlined">
                            {move || if saving.get() { "hourglass_empty" } else { "check" }}
                        </span>
                    </button>
                    <button
                        class="p-1.5 rounded-lg text-gray-400 hover:text-gray-600 dark:hover:text-gray-300
                            hover:bg-gray-100 dark:hover:bg-gray-800 transition-colors"
                        on:click=move |_| current_view.set(View::NodeList)
                        title="Cancel"
                    >
                        <span class="material-symbols-outlined">"close"</span>
                    </button>
                </div>
            </div>
            // Error banner
            {move || error_msg.get().map(|msg| view! {
                <div class="px-6 py-2 bg-red-50 dark:bg-red-900/20 text-red-600 dark:text-red-400 text-sm">
                    {msg}
                </div>
            })}
            // Split editor + preview
            <div class="flex flex-1 divide-x divide-gray-200 dark:divide-gray-700 min-h-0">
                // Editor pane (relative so the autocomplete dropdown can be positioned)
                <div class="flex-1 flex flex-col relative">
                    <textarea
                        node_ref=textarea_ref
                        class="flex-1 p-4 font-mono text-sm resize-none bg-transparent
                            text-gray-900 dark:text-gray-100 focus:outline-none"
                        placeholder="Write in Markdown… use [[Node Title]] to link nodes"
                        prop:value=move || body.get()
                        on:input=on_body_input
                        // Close dropdown on Escape
                        on:keydown=move |ev: leptos::ev::KeyboardEvent| {
                            if ev.key() == "Escape" {
                                wikilink_query.set(None);
                            }
                        }
                    />
                    // Wiki-link autocomplete dropdown
                    {move || {
                        let query = wikilink_query.get()?;
                        let entries = titles_resource.get().and_then(|r| r.ok()).unwrap_or_default();
                        let q_lower = query.to_lowercase();
                        let matches: Vec<String> = entries
                            .iter()
                            .filter(|e| e.title.to_lowercase().contains(&q_lower))
                            .take(8)
                            .map(|e| e.title.clone())
                            .collect();
                        if matches.is_empty() {
                            return None;
                        }
                        Some(view! {
                            <div class="absolute bottom-4 left-4 z-50 w-72
                                bg-white dark:bg-gray-900
                                border border-gray-200 dark:border-gray-700
                                rounded-lg shadow-xl overflow-hidden">
                                <div class="px-3 py-1.5 text-xs text-gray-400 border-b border-gray-100 dark:border-gray-800">
                                    "Link to node — " {query.clone()}
                                </div>
                                {matches.into_iter().map(|t| {
                                    let t_clone = t.clone();
                                    let select = on_select_title;
                                    view! {
                                        <button
                                            type="button"
                                            class="w-full text-left px-3 py-2 text-sm
                                                text-gray-800 dark:text-gray-200
                                                hover:bg-blue-50 dark:hover:bg-blue-900/30
                                                hover:text-blue-700 dark:hover:text-blue-300
                                                transition-colors"
                                            on:click=move |ev| {
                                                ev.prevent_default();
                                                ev.stop_propagation();
                                                select(t_clone.clone());
                                            }
                                        >
                                            <span class="material-symbols-outlined text-xs mr-1 align-middle">"link"</span>
                                            {t.clone()}
                                        </button>
                                    }
                                }).collect_view()}
                            </div>
                        })
                    }}
                </div>
                <div class="flex-1 overflow-auto p-6">
                    <div class="prose max-w-none dark:prose-invert" inner_html=preview_html />
                </div>
            </div>
        </div>
    }
}
