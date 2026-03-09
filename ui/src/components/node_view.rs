use common::id::NodeId;
use leptos::prelude::*;
use pulldown_cmark::{html, Options, Parser};

use crate::app::View;

fn render_markdown(source: &str) -> String {
    let opts = Options::ENABLE_STRIKETHROUGH | Options::ENABLE_TABLES | Options::ENABLE_TASKLISTS;
    let parser = Parser::new_ext(source, opts);
    let mut html_out = String::new();
    html::push_html(&mut html_out, parser);
    ammonia::clean(&html_out)
}

#[component]
pub fn NodeView(id: NodeId) -> impl IntoView {
    let current_view =
        use_context::<RwSignal<View>>().expect("View signal must be provided");
    let refresh =
        use_context::<RwSignal<u32>>().expect("refresh signal must be provided");

    let node = LocalResource::new(move || {
        let id = id;
        async move { crate::api::fetch_node(id).await }
    });

    let deleting = RwSignal::new(false);

    let on_delete = move |_| {
        deleting.set(true);
        let id = id;
        wasm_bindgen_futures::spawn_local(async move {
            if crate::api::delete_node(id).await.is_ok() {
                refresh.update(|n| *n += 1);
                current_view.set(View::NodeList);
            }
            deleting.set(false);
        });
    };

    view! {
        <Suspense fallback=move || view! {
            <div class="p-6 text-gray-400 text-sm">"Loading node..."</div>
        }>
            {move || {
                node.get().map(|result| {
                    match result {
                        Ok(n) => {
                            let body_html = render_markdown(n.body.as_deref().unwrap_or(""));
                            let node_type = format!("{:?}", n.node_type).to_lowercase();
                            let status = format!("{:?}", n.status).to_lowercase();
                            let edit_id = n.id;
                            view! {
                                <div class="flex flex-col h-full">
                                    <div class="flex items-center justify-between px-6 py-4 border-b border-gray-200 dark:border-gray-800">
                                        <div class="flex items-center gap-3">
                                            <button
                                                class="text-gray-400 hover:text-gray-600 dark:hover:text-gray-300"
                                                on:click=move |_| current_view.set(View::NodeList)
                                            >
                                                <span class="material-symbols-outlined">"arrow_back"</span>
                                            </button>
                                            <h1 class="text-lg font-semibold text-gray-900 dark:text-gray-100">
                                                {n.title.clone()}
                                            </h1>
                                            <span class="px-2 py-0.5 text-xs rounded-full bg-blue-100 text-blue-700 dark:bg-blue-900 dark:text-blue-300">
                                                {node_type}
                                            </span>
                                            <span class="px-2 py-0.5 text-xs rounded-full bg-gray-100 text-gray-600 dark:bg-gray-800 dark:text-gray-400">
                                                {status}
                                            </span>
                                        </div>
                                        <div class="flex items-center gap-2">
                                            <button
                                                class="px-3 py-1.5 bg-blue-600 hover:bg-blue-700 text-white text-sm font-medium rounded-lg transition-colors"
                                                on:click=move |_| current_view.set(View::NodeEdit(edit_id))
                                            >
                                                "Edit"
                                            </button>
                                            <button
                                                class="px-3 py-1.5 bg-red-600 hover:bg-red-700 text-white text-sm font-medium rounded-lg transition-colors"
                                                on:click=on_delete
                                                disabled=move || deleting.get()
                                            >
                                                {move || if deleting.get() { "Deleting..." } else { "Delete" }}
                                            </button>
                                        </div>
                                    </div>
                                    <div class="flex-1 overflow-auto p-6">
                                        <div class="prose max-w-none dark:prose-invert" inner_html=body_html />
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
    }
}
