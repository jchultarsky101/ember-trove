/// Full-text + fuzzy search bar with results dropdown.
///
/// Phase 1 stub — Phase 5 wires the search API.
use leptos::prelude::*;

#[component]
pub fn SearchBar() -> impl IntoView {
    let query = RwSignal::new(String::new());

    view! {
        <div class="relative">
            <input
                type="search"
                class="w-full px-4 py-2 pl-10 text-sm bg-gray-100 dark:bg-gray-800
                    border border-transparent rounded-lg focus:outline-none
                    focus:ring-2 focus:ring-blue-500 dark:text-gray-100"
                placeholder="Search nodes…"
                prop:value=move || query.get()
                on:input=move |ev| query.set(event_target_value(&ev))
            />
            <span class="material-symbols-outlined absolute left-3 top-1/2 -translate-y-1/2
                text-gray-400 pointer-events-none">
                "search"
            </span>
        </div>
    }
}
