use common::{
    id::NodeId,
    node::{CreateNodeRequest, NodeType, UpdateNodeRequest},
};
use leptos::prelude::*;
use pulldown_cmark::{Options, Parser, html};

use crate::app::View;

fn render_markdown(source: &str) -> String {
    let opts = Options::ENABLE_STRIKETHROUGH | Options::ENABLE_TABLES | Options::ENABLE_TASKLISTS;
    let parser = Parser::new_ext(source, opts);
    let mut html_out = String::new();
    html::push_html(&mut html_out, parser);
    ammonia::clean(&html_out)
}

#[component]
pub fn NodeEditor(node: Option<NodeId>) -> impl IntoView {
    let current_view = use_context::<RwSignal<View>>().expect("View signal must be provided");
    let refresh = use_context::<RwSignal<u32>>().expect("refresh signal must be provided");

    let title = RwSignal::new(String::new());
    let body = RwSignal::new(String::new());
    let node_type = RwSignal::new("article".to_string());
    let saving = RwSignal::new(false);
    let error_msg = RwSignal::new(Option::<String>::None);

    // If editing, fetch existing node data.
    if let Some(id) = node {
        wasm_bindgen_futures::spawn_local(async move {
            if let Ok(n) = crate::api::fetch_node(id).await {
                title.set(n.title);
                body.set(n.body.unwrap_or_default());
                node_type.set(format!("{:?}", n.node_type).to_lowercase());
            }
        });
    }

    let on_save = move |_| {
        saving.set(true);
        error_msg.set(None);
        let t = title.get_untracked();
        let b = body.get_untracked();
        let nt_str = node_type.get_untracked();

        wasm_bindgen_futures::spawn_local(async move {
            let result = if let Some(id) = node {
                // Update existing node.
                let req = UpdateNodeRequest {
                    title: Some(t),
                    body: Some(b),
                    metadata: None,
                    status: None,
                };
                crate::api::update_node(id, &req).await
            } else {
                // Create new node.
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
                    status: None,
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

    let preview_html = move || render_markdown(&body.get());

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
                    <button
                        class="p-1.5 rounded-lg text-gray-400 hover:text-green-600 dark:hover:text-green-400
                            hover:bg-green-50 dark:hover:bg-green-900/30 transition-colors"
                        on:click=on_save
                        disabled=move || saving.get()
                        title=move || if saving.get() { "Saving…" } else { "Save" }
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
                <div class="flex-1 flex flex-col">
                    <textarea
                        class="flex-1 p-4 font-mono text-sm resize-none bg-transparent
                            text-gray-900 dark:text-gray-100 focus:outline-none"
                        placeholder="Write in Markdown..."
                        prop:value=move || body.get()
                        on:input=move |ev| body.set(event_target_value(&ev))
                    />
                </div>
                <div class="flex-1 overflow-auto p-6">
                    <div class="prose max-w-none dark:prose-invert" inner_html=preview_html />
                </div>
            </div>
        </div>
    }
}
