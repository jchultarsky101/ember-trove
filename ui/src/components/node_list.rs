use common::node::Node;
use leptos::prelude::*;

use crate::app::View;

#[component]
pub fn NodeList() -> impl IntoView {
    let current_view = use_context::<RwSignal<View>>().expect("View signal must be provided");
    let refresh = use_context::<RwSignal<u32>>().expect("refresh signal must be provided");

    let nodes = LocalResource::new(move || {
        let _ = refresh.get();
        async move { crate::api::fetch_nodes().await }
    });

    view! {
        <div class="flex flex-col h-full">
            <div class="flex items-center justify-between px-6 py-4 border-b border-gray-200 dark:border-gray-800">
                <h1 class="text-lg font-semibold text-gray-900 dark:text-gray-100">"Nodes"</h1>
                <button
                    class="px-3 py-1.5 bg-blue-600 hover:bg-blue-700 text-white text-sm
                        font-medium rounded-lg transition-colors"
                    on:click=move |_| current_view.set(View::NodeCreate)
                >
                    "New Node"
                </button>
            </div>
            <div class="flex-1 overflow-auto">
                <Suspense fallback=move || view! {
                    <div class="p-6 text-gray-400 text-sm">"Loading nodes..."</div>
                }>
                    {move || {
                        nodes.get().map(|result| {
                            match result {
                                Ok(list) if list.is_empty() => view! {
                                    <div class="flex-1 flex items-center justify-center p-6">
                                        <p class="text-gray-400 dark:text-gray-600 text-sm">
                                            "No nodes yet. Create your first node to get started."
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
                view! {
                    <li
                        class="px-6 py-4 hover:bg-gray-100 dark:hover:bg-gray-900 cursor-pointer transition-colors"
                        on:click=move |_| current_view.set(View::NodeDetail(id))
                    >
                        <div class="flex items-center justify-between">
                            <div class="flex items-center gap-2">
                                <span class="text-sm font-medium text-gray-900 dark:text-gray-100">
                                    {node.title.clone()}
                                </span>
                                <span class="px-2 py-0.5 text-xs rounded-full bg-blue-100 text-blue-700 dark:bg-blue-900 dark:text-blue-300">
                                    {node_type}
                                </span>
                                <span class="px-2 py-0.5 text-xs rounded-full bg-gray-100 text-gray-600 dark:bg-gray-800 dark:text-gray-400">
                                    {status}
                                </span>
                            </div>
                            <span class="text-xs text-gray-400">{updated}</span>
                        </div>
                    </li>
                }
            }).collect::<Vec<_>>()}
        </ul>
    }
}
