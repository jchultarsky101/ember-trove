use std::collections::HashMap;

use common::edge::{CreateEdgeRequest, EdgeType, EdgeWithTitles};
use common::id::NodeId;
use common::node::NodeTitleEntry;
use leptos::{html::Input, prelude::*};
use pulldown_cmark::{Options, Parser, html};

use crate::app::View;
use crate::components::attachment_panel::AttachmentPanel;
use crate::components::modals::delete_confirm::DeleteConfirmModal;
use crate::components::node_meta::{status_color, status_icon, status_label, type_icon, type_label};
use crate::components::permission_dialog::PermissionPanel;
use crate::components::tag_bar::TagBar;
use crate::auth::{AuthStatus, use_auth_state};
use crate::components::note_panel::NotePanel;
use crate::components::task_panel::TaskPanel;
use crate::components::toast::{ToastLevel, push_toast};
use crate::wikilink::preprocess_wikilinks;

/// Render markdown with wiki-link resolution.
///
/// `[[title]]` and `[[title|display]]` are first replaced with HTML anchors
/// (resolved) or `<span>` tags (unresolved) by `preprocess_wikilinks`, then
/// the result is passed through pulldown-cmark. Ammonia is configured to
/// preserve the `class` and `data-node-id` attributes added by the preprocessor.
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

#[component]
pub fn NodeView(id: NodeId) -> impl IntoView {
    let current_view = use_context::<RwSignal<View>>().expect("View signal must be provided");
    let refresh = use_context::<RwSignal<u32>>().expect("refresh signal must be provided");

    let node = LocalResource::new(move || {
        let id = id;
        async move { crate::api::fetch_node(id).await }
    });

    // Fetch all node titles for wiki-link resolution.
    let titles = LocalResource::new(|| async move { crate::api::fetch_node_titles().await });

    let auth_state = use_auth_state();
    let deleting = RwSignal::new(false);
    let show_delete_confirm = RwSignal::new(false);

    // Derive node title for the confirm dialog (empty string until node loads).
    let delete_item_name = Memo::new(move |_| {
        node.get()
            .and_then(|r| r.ok())
            .map(|n| n.title.clone())
            .unwrap_or_default()
    });

    let do_delete = move || {
        deleting.set(true);
        show_delete_confirm.set(false);
        let id = id;
        wasm_bindgen_futures::spawn_local(async move {
            match crate::api::delete_node(id).await {
                Ok(_) => {
                    push_toast(ToastLevel::Success, "Node deleted.");
                    refresh.update(|n| *n += 1);
                    current_view.set(View::NodeList);
                }
                Err(e) => {
                    push_toast(ToastLevel::Error, format!("Delete failed: {e}"));
                }
            }
            deleting.set(false);
        });
    };

    view! {
        <Suspense fallback=move || view! {
            <div class="p-6 text-stone-400 text-sm">"Loading node..."</div>
        }>
            {move || {
                node.get().map(|result| {
                    match result {
                        Ok(n) => {
                            // Build title → NodeId map from fetched titles (empty map if not
                            // yet loaded — wiki-links will render as unresolved on first paint
                            // and resolve on the next reactive tick).
                            let title_map = titles
                                .get()
                                .and_then(|r| r.ok())
                                .map(|entries| build_title_map(&entries))
                                .unwrap_or_default();
                            let body_html = render_markdown(n.body.as_deref().unwrap_or(""), &title_map);
                            let node_type = format!("{:?}", n.node_type).to_lowercase();
                            let status = format!("{:?}", n.status).to_lowercase();
                            let edit_id = n.id;
                            // Real ownership check: compare the JWT sub with the node's owner_id.
                            let is_owner = if let AuthStatus::Authenticated(ref user) =
                                auth_state.get_untracked()
                            {
                                user.sub == n.owner_id
                            } else {
                                false
                            };

                            // Click delegation: intercept clicks on `.wikilink` anchors and
                            // navigate in-app instead of following the href.
                            let handle_wikilink_click = move |ev: leptos::ev::MouseEvent| {
                                use wasm_bindgen::JsCast;
                                if let Some(link) = ev
                                    .target()
                                    .and_then(|t| t.dyn_into::<web_sys::Element>().ok())
                                    .and_then(|el| el.closest(".wikilink").ok())
                                    .flatten()
                                {
                                    ev.prevent_default();
                                    if let Some(raw) = link.get_attribute("data-node-id")
                                        && let Ok(id) = raw.parse::<uuid::Uuid>() {
                                        current_view.set(View::NodeDetail(NodeId(id)));
                                    }
                                }
                            };

                            view! {
                                <div class="flex flex-col h-full">
                                    <div class="flex items-center justify-between px-6 py-4 border-b border-stone-200 dark:border-stone-800">
                                        <div class="flex items-center gap-3">
                                            <button
                                                class="text-stone-400 hover:text-stone-600 dark:hover:text-stone-300"
                                                on:click=move |_| current_view.set(View::NodeList)
                                            >
                                                <span class="material-symbols-outlined">"arrow_back"</span>
                                            </button>
                                            <h1 class="text-lg font-semibold text-stone-900 dark:text-stone-100">
                                                {n.title.clone()}
                                            </h1>
                                            // Type icon — same encoding as NodeList
                                            <span
                                                class="material-symbols-outlined text-stone-400 dark:text-stone-500"
                                                style="font-size: 18px;"
                                                title=type_label(&node_type)
                                            >
                                                {type_icon(&node_type)}
                                            </span>
                                            // Status icon — semantic colour, same as NodeList
                                            <span
                                                class="material-symbols-outlined"
                                                style=format!("font-size: 18px; {}",
                                                    status_color(&status))
                                                title=status_label(&status)
                                            >
                                                {status_icon(&status)}
                                            </span>
                                        </div>
                                        <div class="flex items-center gap-1">
                                            <button
                                                class="p-1.5 rounded-lg text-stone-400 hover:text-stone-600
                                                    dark:hover:text-stone-300 hover:bg-stone-100
                                                    dark:hover:bg-stone-800 transition-colors"
                                                on:click=move |_| current_view.set(View::NodeEdit(edit_id))
                                                title="Edit"
                                            >
                                                <span class="material-symbols-outlined">"edit"</span>
                                            </button>
                                            <button
                                                class="p-1.5 rounded-lg text-stone-400 hover:text-red-600
                                                    dark:hover:text-red-400 hover:bg-red-50
                                                    dark:hover:bg-red-900/30 transition-colors
                                                    disabled:opacity-30"
                                                on:click=move |_| show_delete_confirm.set(true)
                                                disabled=move || deleting.get()
                                                title=move || if deleting.get() { "Deleting…" } else { "Delete" }
                                            >
                                                <span class="material-symbols-outlined">
                                                    {move || if deleting.get() { "hourglass_empty" } else { "delete" }}
                                                </span>
                                            </button>
                                        </div>
                                    </div>
                                    // Tags
                                    <div class="border-b border-stone-200 dark:border-stone-800">
                                        <TagBar node_id=id />
                                    </div>
                                    <div class="flex-1 overflow-auto p-6">
                                        <div
                                            class="prose max-w-2xl dark:prose-invert"
                                            inner_html=body_html
                                            on:click=handle_wikilink_click
                                        />
                                        // Task panel — shown for all node types
                                        <TaskPanel node_id=id />
                                        // Note panel — append-only; owner can add, everyone can read
                                        <NotePanel node_id=id is_owner=is_owner />
                                        <EdgePanel node_id=id />
                                        <BacklinksPanel node_id=id />
                                        <AttachmentPanel node_id=id />
                                        <PermissionPanel node_id=id is_owner=is_owner />
                                    </div>
                                </div>
                            }.into_any()
                        }
                        Err(e) => view! {
                            <div class="p-6 text-red-500 text-sm">{format!("Error: {e}")}</div>
                        }.into_any(),
                    }
                })
            }}
        </Suspense>

        <DeleteConfirmModal
            show=show_delete_confirm.read_only()
            item_name=Signal::derive(move || delete_item_name.get())
            on_confirm=Callback::new(move |_| do_delete())
            on_cancel=Callback::new(move |_| show_delete_confirm.set(false))
        />
    }
}

/// Shows edges (incoming + outgoing) for a node, with ability to add/remove.
#[component]
fn EdgePanel(node_id: NodeId) -> impl IntoView {
    let current_view = use_context::<RwSignal<View>>().expect("View signal must be provided");
    let open = RwSignal::new(false);
    let refresh_edges = RwSignal::new(0u32);
    let show_add = RwSignal::new(false);
    let target_id_input = RwSignal::new(String::new());
    let edge_type_input = RwSignal::new("references".to_string());
    let label_input = RwSignal::new(String::new());
    let error_msg = RwSignal::new(Option::<String>::None);
    let node_search_query = RwSignal::new(String::new());
    let search_open = RwSignal::new(false);
    let search_input_ref = NodeRef::<Input>::new();

    let picker_results: LocalResource<Vec<common::search::SearchResult>> =
        LocalResource::new(move || {
            let q = node_search_query.get();
            async move {
                if q.len() < 2 {
                    return Vec::new();
                }
                crate::api::search_nodes(&q, false, None, &[], "or", 1, 6)
                    .await
                    .map(|r| r.results)
                    .unwrap_or_default()
            }
        });

    let edges: LocalResource<Result<Vec<EdgeWithTitles>, crate::error::UiError>> =
        LocalResource::new(move || {
            let _ = refresh_edges.get();
            let node_id = node_id;
            async move { crate::api::fetch_edges_for_node(node_id).await }
        });

    let on_add_edge = move |_| {
        let target_str = target_id_input.get_untracked().trim().to_string();
        let et = edge_type_input.get_untracked();
        let label = label_input.get_untracked().trim().to_string();
        error_msg.set(None);

        let target_uuid = match uuid::Uuid::parse_str(&target_str) {
            Ok(u) => u,
            Err(_) => {
                error_msg.set(Some("Invalid target node UUID".to_string()));
                return;
            }
        };

        let edge_type = match et.as_str() {
            "references" => EdgeType::References,
            "contains" => EdgeType::Contains,
            "related_to" => EdgeType::RelatedTo,
            "depends_on" => EdgeType::DependsOn,
            "derived_from" => EdgeType::DerivedFrom,
            _ => EdgeType::References,
        };

        let req = CreateEdgeRequest {
            source_id: node_id,
            target_id: common::id::NodeId(target_uuid),
            edge_type,
            label: if label.is_empty() { None } else { Some(label) },
        };

        wasm_bindgen_futures::spawn_local(async move {
            match crate::api::create_edge(&req).await {
                Ok(_) => {
                    show_add.set(false);
                    target_id_input.set(String::new());
                    node_search_query.set(String::new());
                    if let Some(el) = search_input_ref.get_untracked() {
                        el.set_value("");
                    }
                    label_input.set(String::new());
                    refresh_edges.update(|n| *n += 1);
                }
                Err(e) => error_msg.set(Some(format!("{e}"))),
            }
        });
    };

    view! {
        <div class="mt-8 border-t border-stone-200 dark:border-stone-700 pt-6">
            <div class="flex items-center justify-between">
                <button
                    class="flex items-center gap-1 text-left cursor-pointer"
                    on:click=move |_| open.update(|v| *v = !*v)
                >
                    <span
                        class="material-symbols-outlined text-stone-400 dark:text-stone-500"
                        style="font-size: 16px;"
                    >
                        {move || if open.get() { "expand_more" } else { "chevron_right" }}
                    </span>
                    <h2 class="text-sm font-semibold text-stone-700 dark:text-stone-300">
                        "Connections"
                    </h2>
                </button>
                {move || open.get().then(|| view! {
                    <button
                        class="p-1.5 rounded-lg text-stone-400 hover:text-stone-600
                            dark:hover:text-stone-300 hover:bg-stone-100
                            dark:hover:bg-stone-800 transition-colors"
                        on:click=move |_| show_add.update(|v| *v = !*v)
                        title=move || if show_add.get() { "Cancel" } else { "Add Edge" }
                    >
                        <span class="material-symbols-outlined" style="font-size: 16px;">
                            {move || if show_add.get() { "close" } else { "add_link" }}
                        </span>
                    </button>
                })}
            </div>

            // Add edge form + edge list (only when expanded)
            {move || open.get().then(|| view! {
            <div class="mt-4">
            // Add edge form
            {move || show_add.get().then(|| view! {
                <div class="mb-4 p-3 bg-stone-50 dark:bg-stone-900 rounded-lg space-y-2">
                    <div class="flex gap-2">
                        <div class="relative flex-1">
                            <input
                                type="text"
                                node_ref=search_input_ref
                                class="w-full px-2 py-1 text-xs rounded border border-stone-300 dark:border-stone-600
                                    bg-transparent text-stone-900 dark:text-stone-100 focus:outline-none
                                    focus:ring-1 focus:ring-amber-500"
                                placeholder="Search for a node…"
                                on:input=move |ev| {
                                    let v = event_target_value(&ev);
                                    node_search_query.set(v.clone());
                                    target_id_input.set(v);
                                    search_open.set(true);
                                }
                                on:focus=move |_| search_open.set(true)
                                on:blur=move |_| search_open.set(false)
                            />
                            <Suspense fallback=|| ()>
                            {move || {
                                let results = picker_results.get().unwrap_or_default();
                                if !search_open.get() || results.is_empty() {
                                    return None;
                                }
                                Some(view! {
                                    <div class="absolute z-30 top-full left-0 right-0 mt-0.5
                                        bg-white dark:bg-stone-900 border border-stone-200 dark:border-stone-700
                                        rounded-lg shadow-lg overflow-hidden">
                                        {results.into_iter().map(|r| {
                                            let id_str = r.node_id.to_string();
                                            let title = r.title;
                                            let title2 = title.clone();
                                            view! {
                                                <button
                                                    class="w-full text-left px-3 py-1.5 text-xs
                                                        hover:bg-stone-100 dark:hover:bg-stone-800
                                                        text-stone-700 dark:text-stone-300"
                                                    on:mousedown=move |_| {
                                                        target_id_input.set(id_str.clone());
                                                        node_search_query.set(title.clone());
                                                        if let Some(el) = search_input_ref.get_untracked() {
                                                            el.set_value(&title);
                                                        }
                                                        search_open.set(false);
                                                    }
                                                >
                                                    {title2}
                                                </button>
                                            }
                                        }).collect::<Vec<_>>()}
                                    </div>
                                })
                            }}
                            </Suspense></div>
                        <select
                            class="px-2 py-1 text-xs rounded border border-stone-300 dark:border-stone-600
                                bg-stone-50 dark:bg-stone-800 text-stone-700 dark:text-stone-300
                                focus:outline-none"
                            prop:value=move || edge_type_input.get()
                            on:change=move |ev| edge_type_input.set(event_target_value(&ev))
                        >
                            <option value="references">"References"</option>
                            <option value="contains">"Contains"</option>
                            <option value="related_to">"Related To"</option>
                            <option value="depends_on">"Depends On"</option>
                            <option value="derived_from">"Derived From"</option>
                        </select>
                    </div>
                    <div class="flex gap-2">
                        <input
                            type="text"
                            class="flex-1 px-2 py-1 text-xs rounded border border-stone-300 dark:border-stone-600
                                bg-transparent text-stone-900 dark:text-stone-100 focus:outline-none
                                focus:ring-1 focus:ring-amber-500"
                            placeholder="Label (optional)..."
                            prop:value=move || label_input.get()
                            on:input=move |ev| label_input.set(event_target_value(&ev))
                        />
                        <button
                            class="p-1.5 rounded-lg text-stone-400 hover:text-stone-600
                                dark:hover:text-stone-300 hover:bg-stone-100
                                dark:hover:bg-stone-800 transition-colors"
                            on:click=on_add_edge
                            title="Add edge"
                        >
                            <span class="material-symbols-outlined" style="font-size: 16px;">"check"</span>
                        </button>
                    </div>
                    {move || error_msg.get().map(|msg| view! {
                        <div class="text-xs text-red-500">{msg}</div>
                    })}
                </div>
            })}

            // Edge list
            <Suspense fallback=|| view! {
                <div class="text-xs text-stone-400">"Loading edges..."</div>
            }>
                {move || {
                    edges.get().map(|result| {
                        match result {
                            Ok(edge_list) if edge_list.is_empty() => {
                                view! {
                                    <div class="flex flex-col items-center gap-2 py-6">
                                        <span
                                            class="material-symbols-outlined text-stone-300 dark:text-stone-700"
                                            style="font-size: 32px;"
                                        >
                                            "hub"
                                        </span>
                                        <p class="text-xs text-stone-400 dark:text-stone-600">
                                            "No connections yet."
                                        </p>
                                    </div>
                                }.into_any()
                            }
                            Ok(edge_list) => {
                                view! {
                                    <div class="space-y-1">
                                        {edge_list.into_iter().map(|edge| {
                                            let edge_id = edge.id;
                                            let is_outgoing = edge.source_id == node_id;
                                            let other_id = if is_outgoing { edge.target_id } else { edge.source_id };
                                            let other_title = if is_outgoing { edge.target_title.clone() } else { edge.source_title.clone() };
                                            let direction = if is_outgoing { "\u{2192}" } else { "\u{2190}" };
                                            let edge_type_label = format!("{:?}", edge.edge_type).to_lowercase().replace('_', " ");
                                            let label = edge.label.clone().unwrap_or_default();
                                            view! {
                                                <div class="flex items-center justify-between py-1.5 px-2 rounded
                                                    hover:bg-stone-50 dark:hover:bg-stone-800/50 group">
                                                    <button
                                                        class="flex items-center gap-2 text-xs text-stone-600 dark:text-stone-400
                                                            hover:text-amber-600 dark:hover:text-amber-400 min-w-0"
                                                        on:click=move |_| current_view.set(View::NodeDetail(other_id))
                                                    >
                                                        <span class="shrink-0">{direction}</span>
                                                        <span class="text-stone-400 dark:text-stone-500 shrink-0">{edge_type_label}</span>
                                                        <span class="font-medium truncate">{other_title}</span>
                                                        {(!label.is_empty()).then(|| view! {
                                                            <span class="italic text-stone-400 shrink-0">{format!("({label})")}</span>
                                                        })}
                                                    </button>
                                                    <button
                                                        class="opacity-0 group-hover:opacity-100 text-red-400 hover:text-red-600
                                                            text-xs transition-opacity"
                                                        on:click=move |_| {
                                                            wasm_bindgen_futures::spawn_local(async move {
                                                                let _ = crate::api::delete_edge(edge_id).await;
                                                                refresh_edges.update(|n| *n += 1);
                                                            });
                                                        }
                                                    >
                                                        "\u{00d7}"
                                                    </button>
                                                </div>
                                            }
                                        }).collect::<Vec<_>>()}
                                    </div>
                                }.into_any()
                            }
                            Err(e) => view! {
                                <div class="text-xs text-red-500">{format!("Error: {e}")}</div>
                            }.into_any(),
                        }
                    })
                }}
            </Suspense>
            </div>  // close mt-4
            })}  // close open.then
        </div>
    }
}

/// Shows nodes that link to this node (incoming edges from other nodes).
#[component]
fn BacklinksPanel(node_id: NodeId) -> impl IntoView {
    let current_view = use_context::<RwSignal<View>>().expect("View signal must be provided");
    let open = RwSignal::new(false);

    let backlinks = LocalResource::new(move || {
        let node_id = node_id;
        async move { crate::api::fetch_backlinks(node_id).await }
    });

    view! {
        <div class="mt-8 border-t border-stone-200 dark:border-stone-700 pt-6">
            <button
                class="flex items-center gap-1 text-left cursor-pointer"
                on:click=move |_| open.update(|v| *v = !*v)
            >
                <span
                    class="material-symbols-outlined text-stone-400 dark:text-stone-500"
                    style="font-size: 16px;"
                >
                    {move || if open.get() { "expand_more" } else { "chevron_right" }}
                </span>
                <h2 class="text-sm font-semibold text-stone-700 dark:text-stone-300">
                    "Linked Here"
                </h2>
            </button>
            {move || open.get().then(|| view! {
                <div class="mt-4">
                <Suspense fallback=|| view! {
                    <div class="text-xs text-stone-400">"Loading backlinks..."</div>
                }>
                    {move || {
                        backlinks.get().map(|result| {
                            match result {
                                Ok(nodes) if nodes.is_empty() => view! {
                                    <div class="flex flex-col items-center gap-2 py-6">
                                        <span
                                            class="material-symbols-outlined text-stone-300 dark:text-stone-700"
                                            style="font-size: 32px;"
                                        >
                                            "link_off"
                                        </span>
                                        <p class="text-xs text-stone-400 dark:text-stone-600">
                                            "No other notes link here."
                                        </p>
                                    </div>
                                }.into_any(),
                                Ok(nodes) => view! {
                                    <div class="space-y-1">
                                        {nodes.into_iter().map(|node| {
                                            let node_id = node.id;
                                            let title = node.title.clone();
                                            let node_type = format!("{:?}", node.node_type).to_lowercase();
                                            view! {
                                                <button
                                                    class="flex items-center gap-2 w-full text-left py-1.5 px-2 rounded
                                                        text-xs hover:bg-stone-50 dark:hover:bg-stone-800/50
                                                        text-stone-600 dark:text-stone-400
                                                        hover:text-amber-600 dark:hover:text-amber-400"
                                                    on:click=move |_| current_view.set(View::NodeDetail(node_id))
                                                >
                                                    <span class="material-symbols-outlined text-stone-400 dark:text-stone-600"
                                                        style="font-size: 14px;">"arrow_back"</span>
                                                    <span class="font-medium truncate">{title}</span>
                                                    <span class="text-stone-400 dark:text-stone-600 shrink-0">
                                                        {format!("({node_type})")}
                                                    </span>
                                                </button>
                                            }
                                        }).collect::<Vec<_>>()}
                                    </div>
                                }.into_any(),
                                Err(e) => view! {
                                    <div class="text-xs text-red-500">{format!("Error: {e}")}</div>
                                }.into_any(),
                            }
                        })
                    }}
                </Suspense>
                </div>
            })}
        </div>
    }
}
