use common::{
    id::NodeId,
    note::{CreateNoteRequest, UpdateNoteRequest},
};
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
                                let note_id = note.id;

                                // Show "· edited" if updated_at is meaningfully after created_at.
                                let edited = (note.updated_at - note.created_at).num_seconds() > 2;
                                let ts = {
                                    let base = note.created_at.format("%b %-d, %Y %H:%M").to_string();
                                    if edited { format!("{base} · edited") } else { base }
                                };

                                // Per-note signals — RwSignal is Copy so all closures stay FnMut.
                                let editing    = RwSignal::new(false);
                                let edit_body  = RwSignal::new(note.body.clone());
                                let orig_body  = RwSignal::new(note.body.clone());
                                let save_error = RwSignal::new(Option::<String>::None);

                                let body_display = note.body.clone();
                                let ts_display   = ts.clone();

                                view! {
                                    <div class="rounded-lg bg-stone-50 dark:bg-stone-900/50
                                        border border-stone-200 dark:border-stone-700 px-3 py-2.5">

                                        // ── Display mode ──────────────────────────────
                                        {move || (!editing.get()).then({
                                            let bd = body_display.clone();
                                            let td = ts_display.clone();
                                            move || view! {
                                                <p class="text-sm text-stone-800 dark:text-stone-200 whitespace-pre-wrap">
                                                    {bd.clone()}
                                                </p>
                                                <div class="flex items-center justify-between mt-1.5">
                                                    <p class="text-xs text-stone-400 dark:text-stone-600">
                                                        {td.clone()}
                                                    </p>
                                                    {is_owner.then(|| view! {
                                                        <button
                                                            class="p-0.5 rounded text-stone-300 dark:text-stone-600
                                                                hover:text-amber-500 dark:hover:text-amber-400
                                                                transition-colors"
                                                            title="Edit note"
                                                            on:click=move |_| editing.set(true)
                                                        >
                                                            <span class="material-symbols-outlined"
                                                                style="font-size: 14px;">{"edit"}</span>
                                                        </button>
                                                    })}
                                                </div>
                                            }
                                        })}

                                        // ── Edit mode ─────────────────────────────────
                                        {move || editing.get().then(move || view! {
                                            <textarea
                                                class="w-full bg-transparent text-sm text-stone-800 dark:text-stone-200
                                                    resize-none focus:outline-none"
                                                rows="4"
                                                prop:value=move || edit_body.get()
                                                on:input=move |ev| edit_body.set(event_target_value(&ev))
                                                on:keydown=move |ev: web_sys::KeyboardEvent| {
                                                    if ev.key() == "Escape" {
                                                        editing.set(false);
                                                        edit_body.set(orig_body.get_untracked());
                                                    }
                                                    if ev.key() == "Enter" && (ev.ctrl_key() || ev.meta_key()) {
                                                        let new_body = edit_body.get_untracked().trim().to_string();
                                                        if new_body.is_empty() { return; }
                                                        let req = UpdateNoteRequest { body: new_body };
                                                        wasm_bindgen_futures::spawn_local(async move {
                                                            match crate::api::update_note(note_id, &req).await {
                                                                Ok(_) => {
                                                                    editing.set(false);
                                                                    save_error.set(None);
                                                                    refresh.update(|n| *n += 1);
                                                                }
                                                                Err(e) => save_error.set(Some(format!("{e}"))),
                                                            }
                                                        });
                                                    }
                                                }
                                            />
                                            {move || save_error.get().map(|msg| view! {
                                                <p class="text-xs text-red-500 mt-1">{msg}</p>
                                            })}
                                            <div class="flex items-center justify-end gap-2 mt-2">
                                                <button
                                                    class="px-2 py-1 text-xs text-stone-500 dark:text-stone-400
                                                        hover:text-stone-700 dark:hover:text-stone-200 transition-colors"
                                                    on:click=move |_| {
                                                        editing.set(false);
                                                        edit_body.set(orig_body.get_untracked());
                                                    }
                                                >
                                                    "Cancel"
                                                </button>
                                                <button
                                                    class="px-3 py-1 text-xs font-medium rounded
                                                        bg-amber-500 hover:bg-amber-600 text-white transition-colors
                                                        disabled:opacity-40"
                                                    disabled=move || edit_body.get().trim().is_empty()
                                                    on:click=move |_| {
                                                        let new_body = edit_body.get_untracked().trim().to_string();
                                                        if new_body.is_empty() { return; }
                                                        let req = UpdateNoteRequest { body: new_body };
                                                        wasm_bindgen_futures::spawn_local(async move {
                                                            match crate::api::update_note(note_id, &req).await {
                                                                Ok(_) => {
                                                                    editing.set(false);
                                                                    save_error.set(None);
                                                                    refresh.update(|n| *n += 1);
                                                                }
                                                                Err(e) => save_error.set(Some(format!("{e}"))),
                                                            }
                                                        });
                                                    }
                                                >
                                                    "Save"
                                                </button>
                                            </div>
                                        })}
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
