/// Tag chips + autocomplete input.
///
/// Phase 1 stub — Phase 4 wires the tag API.
use leptos::prelude::*;

#[component]
pub fn TagBar() -> impl IntoView {
    view! {
        <div class="flex flex-wrap gap-1 px-4 py-2">
            <span class="text-xs text-gray-400 dark:text-gray-600">"No tags"</span>
        </div>
    }
}
