/// Filterable, sortable list of nodes.
///
/// Phase 1 renders an empty placeholder; Phase 3 wires the API.
use leptos::prelude::*;

#[component]
pub fn NodeList() -> impl IntoView {
    view! {
        <div class="flex flex-col h-full">
            <div class="flex items-center justify-between px-6 py-4 border-b border-gray-200 dark:border-gray-800">
                <h1 class="text-lg font-semibold text-gray-900 dark:text-gray-100">"Nodes"</h1>
                <button class="px-3 py-1.5 bg-blue-600 hover:bg-blue-700 text-white text-sm
                    font-medium rounded-lg transition-colors">
                    "New Node"
                </button>
            </div>
            <div class="flex-1 flex items-center justify-center">
                <p class="text-gray-400 dark:text-gray-600 text-sm">
                    "No nodes yet. Create your first node to get started."
                </p>
            </div>
        </div>
    }
}
