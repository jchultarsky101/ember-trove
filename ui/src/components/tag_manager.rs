use common::tag::{CreateTagRequest, UpdateTagRequest};
use leptos::prelude::*;

use crate::components::modals::delete_confirm::DeleteConfirmModal;
use crate::components::toast::{ToastLevel, push_toast};

/// Full-page tag management: list all tags, create new, edit name/color, delete.
#[component]
pub fn TagManager() -> impl IntoView {
    let refresh = RwSignal::new(0u32);
    let new_name = RwSignal::new(String::new());
    let new_color = RwSignal::new("#d97706".to_string());
    let error_msg = RwSignal::new(Option::<String>::None);
    let editing_id = RwSignal::new(Option::<common::id::TagId>::None);
    let edit_name = RwSignal::new(String::new());
    let edit_color = RwSignal::new(String::new());

    // Delete confirmation
    let delete_confirm_id: RwSignal<Option<common::id::TagId>> = RwSignal::new(None);
    let delete_confirm_name = RwSignal::new(String::new());

    let tags = LocalResource::new(move || {
        let _ = refresh.get();
        async move { crate::api::fetch_tags().await }
    });

    let do_create = move || {
        let name = new_name.get_untracked().trim().to_string();
        if name.is_empty() {
            return;
        }
        let color = new_color.get_untracked();
        error_msg.set(None);

        wasm_bindgen_futures::spawn_local(async move {
            let req = CreateTagRequest { name: name.clone(), color };
            match crate::api::create_tag(&req).await {
                Ok(_) => {
                    new_name.set(String::new());
                    new_color.set("#d97706".to_string());
                    refresh.update(|n| *n += 1);
                    push_toast(ToastLevel::Success, format!("Tag \"{name}\" created."));
                }
                Err(e) => error_msg.set(Some(format!("{e}"))),
            }
        });
    };

    let on_save_edit = move |_: web_sys::MouseEvent| {
        let Some(id) = editing_id.get_untracked() else {
            return;
        };
        let name = edit_name.get_untracked().trim().to_string();
        let color = edit_color.get_untracked();
        error_msg.set(None);

        wasm_bindgen_futures::spawn_local(async move {
            let req = UpdateTagRequest {
                name: if name.is_empty() { None } else { Some(name.clone()) },
                color: Some(color),
            };
            match crate::api::update_tag(id, &req).await {
                Ok(_) => {
                    editing_id.set(None);
                    refresh.update(|n| *n += 1);
                    push_toast(ToastLevel::Success, "Tag updated.");
                }
                Err(e) => error_msg.set(Some(format!("{e}"))),
            }
        });
    };

    let on_keydown_create = move |ev: web_sys::KeyboardEvent| {
        if ev.key() == "Enter" {
            ev.prevent_default();
            do_create();
        }
    };

    view! {
        <div class="flex flex-col h-full">
            <div class="px-6 py-4 border-b border-stone-200 dark:border-stone-800">
                <h1 class="text-lg font-semibold text-stone-900 dark:text-stone-100">"Tags"</h1>
            </div>

            // Create form
            <div class="px-6 py-3 border-b border-stone-200 dark:border-stone-800">
                <div class="flex items-center gap-2">
                    <input
                        type="text"
                        class="flex-1 px-3 py-1.5 text-sm rounded-lg border border-stone-300 dark:border-stone-600
                            bg-transparent text-stone-900 dark:text-stone-100 focus:outline-none
                            focus:ring-1 focus:ring-amber-500"
                        placeholder="New tag name..."
                        prop:value=move || new_name.get()
                        on:input=move |ev| new_name.set(event_target_value(&ev))
                        on:keydown=on_keydown_create
                    />
                    <input
                        type="color"
                        class="w-8 h-8 rounded cursor-pointer border-0"
                        prop:value=move || new_color.get()
                        on:input=move |ev| new_color.set(event_target_value(&ev))
                    />
                    <button
                        class="px-3 py-1.5 bg-amber-600 hover:bg-amber-700 text-white text-sm font-medium
                            rounded-lg transition-colors"
                        on:click=move |_| do_create()
                    >
                        "Create"
                    </button>
                </div>
                {move || error_msg.get().map(|msg| view! {
                    <div class="mt-2 text-xs text-red-500">{msg}</div>
                })}
            </div>

            // Tag list
            <div class="flex-1 overflow-auto">
                <Suspense fallback=|| view! {
                    // Skeleton rows
                    <div class="divide-y divide-stone-100 dark:divide-stone-800">
                        {(0..4).map(|_| view! {
                            <div class="flex items-center gap-3 px-6 py-3">
                                <div class="w-4 h-4 rounded-full bg-stone-200 dark:bg-stone-700 animate-pulse flex-shrink-0" />
                                <div class="h-3.5 rounded bg-stone-200 dark:bg-stone-700 animate-pulse w-28" />
                            </div>
                        }).collect::<Vec<_>>()}
                    </div>
                }>
                    {move || {
                        tags.get().map(|result| {
                            match result {
                                Ok(tag_list) if tag_list.is_empty() => {
                                    view! {
                                        <div class="flex flex-col items-center justify-center gap-3 py-16">
                                            <span
                                                class="material-symbols-outlined text-stone-300 dark:text-stone-700"
                                                style="font-size: 48px;"
                                            >
                                                "label_off"
                                            </span>
                                            <p class="text-sm text-stone-400 dark:text-stone-600">
                                                "No tags yet. Create one above."
                                            </p>
                                        </div>
                                    }.into_any()
                                }
                                Ok(tag_list) => {
                                    view! {
                                        <div class="divide-y divide-stone-100 dark:divide-stone-800">
                                            {tag_list.into_iter().map(|tag| {
                                                let tag_id = tag.id;
                                                let name = tag.name.clone();
                                                let color = tag.color.clone();
                                                view! {
                                                    <div class="flex items-center justify-between px-6 py-3 hover:bg-stone-50 dark:hover:bg-stone-900/50 group">
                                                        {move || {
                                                            if editing_id.get() == Some(tag_id) {
                                                                // Edit mode
                                                                view! {
                                                                    <div class="flex items-center gap-2 flex-1">
                                                                        <input
                                                                            type="color"
                                                                            class="w-6 h-6 rounded cursor-pointer border-0"
                                                                            prop:value=move || edit_color.get()
                                                                            on:input=move |ev| edit_color.set(event_target_value(&ev))
                                                                        />
                                                                        <input
                                                                            type="text"
                                                                            class="flex-1 px-2 py-1 text-sm rounded border border-stone-300
                                                                                dark:border-stone-600 bg-transparent text-stone-900
                                                                                dark:text-stone-100 focus:outline-none focus:ring-1
                                                                                focus:ring-amber-500"
                                                                            prop:value=move || edit_name.get()
                                                                            on:input=move |ev| edit_name.set(event_target_value(&ev))
                                                                        />
                                                                        <button
                                                                            class="px-2 py-1 text-xs bg-amber-600 text-white rounded"
                                                                            on:click=on_save_edit
                                                                        >
                                                                            "Save"
                                                                        </button>
                                                                        <button
                                                                            class="px-2 py-1 text-xs text-stone-500 hover:text-stone-700"
                                                                            on:click=move |_| editing_id.set(None)
                                                                        >
                                                                            "Cancel"
                                                                        </button>
                                                                    </div>
                                                                }.into_any()
                                                            } else {
                                                                // Display mode
                                                                let display_name = name.clone();
                                                                let display_color = color.clone();
                                                                let edit_name_val = name.clone();
                                                                let edit_color_val = color.clone();
                                                                let del_name = name.clone();
                                                                view! {
                                                                    <div class="flex items-center gap-3 flex-1">
                                                                        <span
                                                                            class="w-4 h-4 rounded-full flex-shrink-0"
                                                                            style:background-color=display_color
                                                                        />
                                                                        <span class="text-sm text-stone-900 dark:text-stone-100">
                                                                            {display_name}
                                                                        </span>
                                                                    </div>
                                                                    <div class="flex items-center gap-2 opacity-0 group-hover:opacity-100 transition-opacity">
                                                                        <button
                                                                            class="text-xs text-stone-400 hover:text-stone-600 dark:hover:text-stone-300"
                                                                            on:click=move |_| {
                                                                                editing_id.set(Some(tag_id));
                                                                                edit_name.set(edit_name_val.clone());
                                                                                edit_color.set(edit_color_val.clone());
                                                                            }
                                                                        >
                                                                            "Edit"
                                                                        </button>
                                                                        <button
                                                                            class="text-xs text-red-400 hover:text-red-600"
                                                                            on:click=move |_| {
                                                                                delete_confirm_id.set(Some(tag_id));
                                                                                delete_confirm_name.set(del_name.clone());
                                                                            }
                                                                        >
                                                                            "Delete"
                                                                        </button>
                                                                    </div>
                                                                }.into_any()
                                                            }
                                                        }}
                                                    </div>
                                                }
                                            }).collect::<Vec<_>>()}
                                        </div>
                                    }.into_any()
                                }
                                Err(e) => view! {
                                    <div class="p-6 text-red-500 text-sm">{format!("Error: {e}")}</div>
                                }.into_any(),
                            }
                        })
                    }}
                </Suspense>
            </div>
        </div>

        // Delete confirmation modal — outside the list so it overlays everything
        <DeleteConfirmModal
            show=Signal::derive(move || delete_confirm_id.get().is_some())
            item_name=Signal::derive(move || delete_confirm_name.get())
            on_confirm=Callback::new(move |_| {
                let Some(id) = delete_confirm_id.get_untracked() else { return; };
                let name = delete_confirm_name.get_untracked();
                delete_confirm_id.set(None);
                wasm_bindgen_futures::spawn_local(async move {
                    match crate::api::delete_tag(id).await {
                        Ok(_) => {
                            push_toast(ToastLevel::Success, format!("Tag \"{name}\" deleted."));
                            refresh.update(|n| *n += 1);
                        }
                        Err(e) => push_toast(ToastLevel::Error, format!("Delete failed: {e}")),
                    }
                });
            })
            on_cancel=Callback::new(move |_| delete_confirm_id.set(None))
        />
    }
}
