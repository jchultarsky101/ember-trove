#![allow(dead_code)]
/// Modal for creating a new node (title, type).
///
/// Phase 1 stub — Phase 3 wires the API call.
use leptos::prelude::*;

#[component]
pub fn CreateNodeModal(#[prop(into)] show: Signal<bool>, on_close: Callback<()>) -> impl IntoView {
    view! {
        <Show when=move || show.get()>
            <div class="fixed inset-0 z-50 flex items-center justify-center bg-black/40">
                <div class="bg-white dark:bg-gray-900 rounded-xl shadow-xl p-6 w-full max-w-md">
                    <h2 class="text-lg font-semibold text-gray-900 dark:text-gray-100 mb-4">
                        "Create Node"
                    </h2>
                    <p class="text-sm text-gray-500 dark:text-gray-400 mb-4">
                        "Node creation coming in Phase 3."
                    </p>
                    <div class="flex justify-end">
                        <button
                            class="px-4 py-2 text-sm text-gray-700 dark:text-gray-300
                                hover:bg-gray-100 dark:hover:bg-gray-800 rounded-lg transition-colors"
                            on:click=move |_| on_close.run(())
                        >
                            "Cancel"
                        </button>
                    </div>
                </div>
            </div>
        </Show>
    }
}
