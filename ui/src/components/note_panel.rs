use common::{id::NodeId, note::CreateNoteRequest};
use leptos::prelude::*;

#[component]
pub fn NotePanel(node_id: NodeId, is_owner: bool) -> impl IntoView {
    let refresh = RwSignal::new(0u32);
    let show_form = RwSignal::new(false);
    let body = RwSignal::new(String::new());

    let notes_resource = LocalResource::new(move || {
        let _ = refresh.get();
        async move { crate::api::fetch_notes(node_id).await }
    });

    let do_add = move || {
        let text = body.get_untracked();
        let trimmed = text.trim().to_string();
        if trimmed.is_empty() {
            return;
        }
        let req = CreateNoteRequest { body: trimmed };
        wasm_bindgen_futures::spawn_local(async move {
            if crate::api::create_note(node_id, &req).await.is_ok() {
                body.set(String::new());
                show_form.set(false);
                refresh.update(|n| *n += 1);
            }
        });
    };

    let on_add = move |_| do_add();

    let on_keydown = move |ev: web_sys::KeyboardEvent| {
        // Ctrl+Enter or Cmd+Enter submits
        if ev.key() == "Enter" && (ev.ctrl_key() || ev.meta_key()) {
            do_add();
        }
    };

    view! {
        <div class="border-t border-stone-200 dark:border-stone-800 pt-6">
            // Header
            <div class="flex items-center justify-between mb-3">
                <div class="flex items-center gap-2">
                    <span class="material-symbols-outlined text-stone-400 dark:text-stone-500"
                        style="font-size: 18px;">{"sticky_note_2"}</span>
                    <h3 class="text-sm font-semibold text-stone-700 dark:text-stone-300 uppercase tracking-wider">
                        "Notes"
                    </h3>
                </div>
                {move || is_owner.then(|| view! {
                    <button
                        class="flex items-center gap-1 text-xs text-amber-600 dark:text-amber-400
                            hover:text-amber-700 dark:hover:text-amber-300 transition-colors cursor-pointer"
                        on:click=move |_| show_form.update(|v| *v = !*v)
                    >
                        <span class="material-symbols-outlined" style="font-size: 16px;">
                            {move || if show_form.get() { "close" } else { "add" }}
                        </span>
                        {move || if show_form.get() { "Cancel" } else { "Add note" }}
                    </button>
                })}
            </div>

            // Add note form (owner only, toggled)
            {move || (is_owner && show_form.get()).then(|| view! {
                <div class="mb-4 rounded-lg bg-stone-50 dark:bg-stone-900/50
                    border border-stone-200 dark:border-stone-700 p-3">
                    <textarea
                        class="w-full bg-transparent text-sm text-stone-800 dark:text-stone-200
                            placeholder:text-stone-400 dark:placeholder:text-stone-600
                            resize-none focus:outline-none"
                        rows="3"
                        placeholder="Write a note… (Ctrl+Enter to save)"
                        prop:value=move || body.get()
                        on:input=move |ev| body.set(event_target_value(&ev))
                        on:keydown=on_keydown
                    />
                    <div class="flex justify-end mt-2">
                        <button
                            class="px-3 py-1 text-xs font-medium rounded
                                bg-amber-500 hover:bg-amber-600 text-white transition-colors cursor-pointer
                                disabled:opacity-40 disabled:cursor-not-allowed"
                            disabled=move || body.get().trim().is_empty()
                            on:click=on_add
                        >
                            "Save note"
                        </button>
                    </div>
                </div>
            })}

            // Notes list
            <Suspense fallback=move || view! {
                <p class="text-xs text-stone-400">"Loading…"</p>
            }>
                {move || {
                    let notes = notes_resource.get()
                        .and_then(|r| r.ok())
                        .unwrap_or_default();

                    if notes.is_empty() {
                        return view! {
                            <p class="text-xs text-stone-400 dark:text-stone-600 italic py-2">
                                "No notes yet."
                            </p>
                        }.into_any();
                    }

                    view! {
                        <div class="space-y-3">
                            {notes.into_iter().map(|note| {
                                let ts = note.created_at.format("%b %-d, %Y %H:%M").to_string();
                                let body_text = note.body.clone();
                                view! {
                                    <div class="rounded-lg bg-stone-50 dark:bg-stone-900/50
                                        border border-stone-200 dark:border-stone-700 px-3 py-2.5">
                                        <p class="text-sm text-stone-800 dark:text-stone-200 whitespace-pre-wrap">
                                            {body_text}
                                        </p>
                                        <p class="text-xs text-stone-400 dark:text-stone-600 mt-1.5">
                                            {ts}
                                        </p>
                                    </div>
                                }
                            }).collect_view()}
                        </div>
                    }.into_any()
                }}
            </Suspense>
        </div>
    }
}
