#![allow(dead_code)]
/// Search-and-select modal for linking nodes with a typed edge.
///
/// Phase 1 stub — Phase 4 wires the node search + edge creation.
use leptos::prelude::*;

#[component]
pub fn LinkPickerModal(
    #[prop(into)] show: Signal<bool>,
    on_close: Callback<()>,
) -> impl IntoView {
    let query = RwSignal::new(String::new());

    view! {
        <Show when=move || show.get()>
            <div class="fixed inset-0 z-50 flex items-center justify-center bg-black/40">
                <div class="bg-white dark:bg-gray-900 rounded-xl shadow-xl p-6 w-full max-w-lg">
                    <h2 class="text-lg font-semibold text-gray-900 dark:text-gray-100 mb-4">
                        "Link Node"
                    </h2>
                    <input
                        type="search"
                        class="w-full px-4 py-2 text-sm bg-gray-100 dark:bg-gray-800
                            border border-transparent rounded-lg focus:outline-none
                            focus:ring-2 focus:ring-blue-500 dark:text-gray-100 mb-4"
                        placeholder="Search for a node to link…"
                        prop:value=move || query.get()
                        on:input=move |ev| query.set(event_target_value(&ev))
                    />
                    <p class="text-sm text-gray-500 dark:text-gray-400 mb-4">
                        "Link picker coming in Phase 4."
                    </p>
                    <div class="flex justify-end">
                        <button
                            class="px-4 py-2 text-sm text-gray-700 dark:text-gray-300
                                hover:bg-gray-100 dark:hover:bg-gray-800 rounded-lg"
                            on:click=move |_| on_close.run(())
                        >
                            "Close"
                        </button>
                    </div>
                </div>
            </div>
        </Show>
    }
}
