use common::node::{CreateNodeRequest, NodeType};
use leptos::prelude::*;
use wasm_bindgen_futures::spawn_local;

use crate::{
    api::create_node,
    app::View,
};

/// Quick-capture modal — lightweight node creation without leaving the current view.
///
/// Fields: title (required), node type (select), body (optional textarea).
/// On success: bumps the global `refresh` signal and navigates to the new node.
/// Closes on Escape or Cancel.
#[component]
pub fn CreateNodeModal(
    /// Whether the modal is visible.
    #[prop(into)]
    show: Signal<bool>,
    /// Called when the modal should close (cancel or successful save).
    on_close: Callback<()>,
) -> impl IntoView {
    let title = RwSignal::new(String::new());
    let body = RwSignal::new(String::new());
    let node_type_str = RwSignal::new("article".to_string());
    let loading = RwSignal::new(false);
    let error: RwSignal<Option<String>> = RwSignal::new(None);

    let refresh = use_context::<RwSignal<u32>>().expect("refresh signal must be provided");
    let current_view =
        use_context::<RwSignal<View>>().expect("View signal must be provided");

    // Reset fields every time the modal opens.
    Effect::new(move |_| {
        if show.get() {
            title.set(String::new());
            body.set(String::new());
            node_type_str.set("article".to_string());
            error.set(None);
            loading.set(false);
        }
    });

    let handle_submit = move || {
        let t = title.get_untracked();
        if t.trim().is_empty() {
            error.set(Some("Title is required.".to_string()));
            return;
        }
        let node_type = match node_type_str.get_untracked().as_str() {
            "project"   => NodeType::Project,
            "area"      => NodeType::Area,
            "resource"  => NodeType::Resource,
            "reference" => NodeType::Reference,
            _           => NodeType::Article,
        };
        let b = body.get_untracked();
        let body_opt = if b.trim().is_empty() { None } else { Some(b) };
        let req = CreateNodeRequest {
            title: t.trim().to_string(),
            node_type,
            body: body_opt,
            metadata: serde_json::Value::Object(serde_json::Map::new()),
            status: None,
        };
        loading.set(true);
        error.set(None);
        spawn_local(async move {
            match create_node(&req).await {
                Ok(node) => {
                    loading.set(false);
                    refresh.update(|n| *n += 1);
                    current_view.set(View::NodeDetail(node.id));
                    on_close.run(());
                }
                Err(e) => {
                    loading.set(false);
                    error.set(Some(e.to_string()));
                }
            }
        });
    };

    // Keyboard handler: Escape closes, Ctrl+Enter submits.
    let handle_keydown = move |ev: web_sys::KeyboardEvent| {
        if ev.key() == "Escape" {
            on_close.run(());
        } else if ev.key() == "Enter" && (ev.ctrl_key() || ev.meta_key()) {
            handle_submit();
        }
    };

    view! {
        <Show when=move || show.get()>
            // Backdrop
            <div
                class="fixed inset-0 z-40 bg-black/50 backdrop-blur-sm"
                on:click=move |_| on_close.run(())
            />
            // Modal panel
            <div
                class="fixed inset-0 z-50 flex items-center justify-center p-4"
                on:keydown=handle_keydown
            >
                <div class="bg-white dark:bg-stone-900 rounded-2xl shadow-2xl
                            border border-stone-200 dark:border-stone-700
                            w-full max-w-lg flex flex-col gap-4 p-6"
                    // Prevent backdrop click from closing when clicking inside
                    on:click=|ev| ev.stop_propagation()
                >
                    // Header
                    <div class="flex items-center justify-between">
                        <h2 class="text-lg font-semibold text-stone-900 dark:text-stone-100">
                            "Quick Capture"
                        </h2>
                        <button
                            class="w-7 h-7 flex items-center justify-center rounded-lg
                                   text-stone-400 hover:text-stone-600 dark:hover:text-stone-300
                                   hover:bg-stone-100 dark:hover:bg-stone-800 transition-colors"
                            on:click=move |_| on_close.run(())
                            title="Close (Esc)"
                        >
                            <span class="material-symbols-outlined" style="font-size: 18px;">
                                "close"
                            </span>
                        </button>
                    </div>

                    // Title input
                    <div class="flex flex-col gap-1">
                        <label class="text-xs font-medium text-stone-500 dark:text-stone-400 uppercase tracking-wide">
                            "Title"
                            <span class="text-red-400 ml-0.5">"*"</span>
                        </label>
                        <input
                            type="text"
                            placeholder="Node title…"
                            autofocus
                            class="w-full px-3 py-2 rounded-lg text-sm
                                   bg-stone-50 dark:bg-stone-800
                                   border border-stone-200 dark:border-stone-700
                                   text-stone-900 dark:text-stone-100
                                   placeholder-stone-400 dark:placeholder-stone-500
                                   focus:outline-none focus:ring-2 focus:ring-amber-500 dark:focus:ring-amber-400
                                   transition-colors"
                            prop:value=move || title.get()
                            on:input=move |ev| title.set(event_target_value(&ev))
                        />
                    </div>

                    // Type select
                    <div class="flex flex-col gap-1">
                        <label class="text-xs font-medium text-stone-500 dark:text-stone-400 uppercase tracking-wide">
                            "Type"
                        </label>
                        <select
                            class="w-full px-3 py-2 rounded-lg text-sm
                                   bg-stone-50 dark:bg-stone-800
                                   border border-stone-200 dark:border-stone-700
                                   text-stone-900 dark:text-stone-100
                                   focus:outline-none focus:ring-2 focus:ring-amber-500 dark:focus:ring-amber-400
                                   transition-colors cursor-pointer"
                            prop:value=move || node_type_str.get()
                            on:change=move |ev| node_type_str.set(event_target_value(&ev))
                        >
                            <option value="article">"Article"</option>
                            <option value="project">"Project"</option>
                            <option value="area">"Area"</option>
                            <option value="resource">"Resource"</option>
                            <option value="reference">"Reference"</option>
                        </select>
                    </div>

                    // Body textarea (optional)
                    <div class="flex flex-col gap-1">
                        <label class="text-xs font-medium text-stone-500 dark:text-stone-400 uppercase tracking-wide">
                            "Notes "
                            <span class="normal-case font-normal text-stone-400 dark:text-stone-500">"(optional)"</span>
                        </label>
                        <textarea
                            rows="4"
                            placeholder="Start writing… (Markdown supported)"
                            class="w-full px-3 py-2 rounded-lg text-sm resize-none
                                   bg-stone-50 dark:bg-stone-800
                                   border border-stone-200 dark:border-stone-700
                                   text-stone-900 dark:text-stone-100
                                   placeholder-stone-400 dark:placeholder-stone-500
                                   focus:outline-none focus:ring-2 focus:ring-amber-500 dark:focus:ring-amber-400
                                   transition-colors font-mono"
                            prop:value=move || body.get()
                            on:input=move |ev| body.set(event_target_value(&ev))
                        />
                    </div>

                    // Error banner
                    {move || error.get().map(|msg| view! {
                        <p class="text-sm text-red-500 dark:text-red-400">{msg}</p>
                    })}

                    // Actions
                    <div class="flex items-center justify-between pt-1">
                        <span class="text-xs text-stone-400 dark:text-stone-500">
                            "Ctrl+Enter to save · Esc to cancel"
                        </span>
                        <div class="flex gap-2">
                            <button
                                class="px-4 py-2 text-sm rounded-lg
                                       text-stone-600 dark:text-stone-400
                                       hover:bg-stone-100 dark:hover:bg-stone-800
                                       transition-colors"
                                on:click=move |_| on_close.run(())
                                disabled=move || loading.get()
                            >
                                "Cancel"
                            </button>
                            <button
                                class="px-4 py-2 text-sm font-medium rounded-lg
                                       bg-amber-600 hover:bg-amber-700
                                       text-white
                                       disabled:opacity-50 disabled:cursor-not-allowed
                                       transition-colors flex items-center gap-1.5"
                                on:click=move |_| handle_submit()
                                disabled=move || loading.get()
                            >
                                {move || if loading.get() {
                                    view! {
                                        <span class="material-symbols-outlined animate-spin"
                                              style="font-size: 16px;">"progress_activity"</span>
                                        "Saving…"
                                    }.into_any()
                                } else {
                                    view! {
                                        <span class="material-symbols-outlined"
                                              style="font-size: 16px;">"add"</span>
                                        "Create"
                                    }.into_any()
                                }}
                            </button>
                        </div>
                    </div>
                </div>
            </div>
        </Show>
    }
}
