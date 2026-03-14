use common::tag::{CreateTagRequest, UpdateTagRequest};
use leptos::prelude::*;

/// Full-page tag management: list all tags, create new, edit name/color, delete.
#[component]
pub fn TagManager() -> impl IntoView {
    let refresh = RwSignal::new(0u32);
    let new_name = RwSignal::new(String::new());
    let new_color = RwSignal::new("#3b82f6".to_string());
    let error_msg = RwSignal::new(Option::<String>::None);
    let editing_id = RwSignal::new(Option::<common::id::TagId>::None);
    let edit_name = RwSignal::new(String::new());
    let edit_color = RwSignal::new(String::new());

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
            let req = CreateTagRequest { name, color };
            match crate::api::create_tag(&req).await {
                Ok(_) => {
                    new_name.set(String::new());
                    new_color.set("#3b82f6".to_string());
                    refresh.update(|n| *n += 1);
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
                name: if name.is_empty() { None } else { Some(name) },
                color: Some(color),
            };
            match crate::api::update_tag(id, &req).await {
                Ok(_) => {
                    editing_id.set(None);
                    refresh.update(|n| *n += 1);
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
            <div class="px-6 py-4 border-b border-gray-200 dark:border-gray-800">
                <h1 class="text-lg font-semibold text-gray-900 dark:text-gray-100">"Tags"</h1>
            </div>

            // Create form
            <div class="px-6 py-3 border-b border-gray-200 dark:border-gray-800">
                <div class="flex items-center gap-2">
                    <input
                        type="text"
                        class="flex-1 px-3 py-1.5 text-sm rounded-lg border border-gray-300 dark:border-gray-600
                            bg-transparent text-gray-900 dark:text-gray-100 focus:outline-none
                            focus:ring-1 focus:ring-blue-500"
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
                        class="px-3 py-1.5 bg-blue-600 hover:bg-blue-700 text-white text-sm font-medium
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
                    <div class="p-6 text-gray-400 text-sm">"Loading tags..."</div>
                }>
                    {move || {
                        tags.get().map(|result| {
                            match result {
                                Ok(tag_list) if tag_list.is_empty() => {
                                    view! {
                                        <div class="p-6 text-gray-400 text-sm">"No tags yet. Create one above."</div>
                                    }.into_any()
                                }
                                Ok(tag_list) => {
                                    view! {
                                        <div class="divide-y divide-gray-100 dark:divide-gray-800">
                                            {tag_list.into_iter().map(|tag| {
                                                let tag_id = tag.id;
                                                let name = tag.name.clone();
                                                let color = tag.color.clone();
                                                view! {
                                                    <div class="flex items-center justify-between px-6 py-3 hover:bg-gray-50 dark:hover:bg-gray-900/50 group">
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
                                                                            class="flex-1 px-2 py-1 text-sm rounded border border-gray-300
                                                                                dark:border-gray-600 bg-transparent text-gray-900
                                                                                dark:text-gray-100 focus:outline-none focus:ring-1
                                                                                focus:ring-blue-500"
                                                                            prop:value=move || edit_name.get()
                                                                            on:input=move |ev| edit_name.set(event_target_value(&ev))
                                                                        />
                                                                        <button
                                                                            class="px-2 py-1 text-xs bg-blue-600 text-white rounded"
                                                                            on:click=on_save_edit
                                                                        >
                                                                            "Save"
                                                                        </button>
                                                                        <button
                                                                            class="px-2 py-1 text-xs text-gray-500 hover:text-gray-700"
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
                                                                view! {
                                                                    <div class="flex items-center gap-3 flex-1">
                                                                        <span
                                                                            class="w-4 h-4 rounded-full flex-shrink-0"
                                                                            style:background-color=display_color
                                                                        />
                                                                        <span class="text-sm text-gray-900 dark:text-gray-100">
                                                                            {display_name}
                                                                        </span>
                                                                    </div>
                                                                    <div class="flex items-center gap-2 opacity-0 group-hover:opacity-100 transition-opacity">
                                                                        <button
                                                                            class="text-xs text-gray-400 hover:text-gray-600 dark:hover:text-gray-300"
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
                                                                                wasm_bindgen_futures::spawn_local(async move {
                                                                                    let _ = crate::api::delete_tag(tag_id).await;
                                                                                    refresh.update(|n| *n += 1);
                                                                                });
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
    }
}
