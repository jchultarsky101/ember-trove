/// Confirmation dialog for destructive deletes.
use leptos::prelude::*;

#[component]
pub fn DeleteConfirmModal(
    #[prop(into)] show: Signal<bool>,
    #[prop(into)] item_name: Signal<String>,
    on_confirm: Callback<()>,
    on_cancel: Callback<()>,
) -> impl IntoView {
    view! {
        <Show when=move || show.get()>
            <div class="fixed inset-0 z-50 flex items-center justify-center bg-black/40">
                <div class="bg-white dark:bg-gray-900 rounded-xl shadow-xl p-6 w-full max-w-sm">
                    <h2 class="text-lg font-semibold text-gray-900 dark:text-gray-100 mb-2">
                        "Delete?"
                    </h2>
                    <p class="text-sm text-gray-600 dark:text-gray-400 mb-6">
                        "Are you sure you want to delete "
                        <strong>{move || item_name.get()}</strong>
                        "? This cannot be undone."
                    </p>
                    <div class="flex justify-end gap-3">
                        <button
                            class="px-4 py-2 text-sm text-gray-700 dark:text-gray-300
                                hover:bg-gray-100 dark:hover:bg-gray-800 rounded-lg"
                            on:click=move |_| on_cancel.run(())
                        >
                            "Cancel"
                        </button>
                        <button
                            class="px-4 py-2 text-sm bg-red-600 hover:bg-red-700 text-white
                                rounded-lg transition-colors"
                            on:click=move |_| on_confirm.run(())
                        >
                            "Delete"
                        </button>
                    </div>
                </div>
            </div>
        </Show>
    }
}
