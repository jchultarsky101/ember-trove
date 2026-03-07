/// Read-only rendered Markdown view.
///
/// Phase 1 stub — Phase 3 fetches the node and renders it.
use leptos::prelude::*;

#[component]
pub fn NodeView() -> impl IntoView {
    view! {
        <div class="p-6">
            <p class="text-gray-400 dark:text-gray-600 text-sm">"Select a node to read it."</p>
        </div>
    }
}
