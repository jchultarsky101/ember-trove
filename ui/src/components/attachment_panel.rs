/// Attachment panel — list, upload, delete, and inline-preview files attached to a node.
use common::id::NodeId;
use leptos::{html::Input, prelude::*};
use wasm_bindgen::JsCast;

use crate::api;

/// Returns true for MIME types the browser can display inline.
fn is_previewable(content_type: &str) -> bool {
    content_type.starts_with("image/") || content_type == "application/pdf"
}

#[component]
pub fn AttachmentPanel(node_id: NodeId) -> impl IntoView {
    let refresh = RwSignal::new(0u32);
    let uploading = RwSignal::new(false);
    let error_msg = RwSignal::new(Option::<String>::None);
    let selected_filename = RwSignal::new(Option::<String>::None);
    let file_input_ref = NodeRef::<Input>::new();

    let attachments = LocalResource::new(move || {
        let _ = refresh.get();
        let node_id = node_id;
        async move { api::fetch_attachments(node_id).await }
    });

    let on_upload = move |_| {
        let input_el = match file_input_ref.get_untracked() {
            Some(el) => el,
            None => return,
        };
        let files = match input_el.files() {
            Some(f) if f.length() > 0 => f,
            _ => {
                error_msg.set(Some("Please select a file.".to_string()));
                return;
            }
        };
        let file: web_sys::File = match files.get(0) {
            Some(f) => f,
            None => {
                error_msg.set(Some("No file selected.".to_string()));
                return;
            }
        };

        let form_data = match web_sys::FormData::new() {
            Ok(fd) => fd,
            Err(_) => {
                error_msg.set(Some("Could not create form data.".to_string()));
                return;
            }
        };

        // File inherits Blob in JS; unchecked_ref is safe here.
        let blob: &web_sys::Blob = file.unchecked_ref();
        if form_data
            .append_with_blob_and_filename("file", blob, &file.name())
            .is_err()
        {
            error_msg.set(Some("Could not attach file.".to_string()));
            return;
        }

        error_msg.set(None);
        uploading.set(true);

        wasm_bindgen_futures::spawn_local(async move {
            match api::upload_attachment(node_id, form_data).await {
                Ok(_) => {
                    refresh.update(|n| *n += 1);
                    selected_filename.set(None);
                    if let Some(el) = file_input_ref.get_untracked() {
                        el.set_value("");
                    }
                }
                Err(e) => error_msg.set(Some(format!("Upload failed: {e}"))),
            }
            uploading.set(false);
        });
    };

    let on_pick = move |_| {
        if let Some(el) = file_input_ref.get_untracked() {
            el.click();
        }
    };

    let on_file_change = move |_| {
        let name = file_input_ref
            .get_untracked()
            .and_then(|el| el.files())
            .and_then(|files| files.get(0))
            .map(|f| f.name());
        selected_filename.set(name);
    };

    let open = RwSignal::new(false);

    view! {
        <div class="mt-8 border-t border-stone-200 dark:border-stone-700 pt-6">
            <div class="flex items-center justify-between">
                <button
                    class="flex items-center gap-1 text-left cursor-pointer"
                    on:click=move |_| open.update(|v| *v = !*v)
                >
                    <span
                        class="material-symbols-outlined text-stone-400 dark:text-stone-500"
                        style="font-size: 16px;"
                    >
                        {move || if open.get() { "expand_more" } else { "chevron_right" }}
                    </span>
                    <h2 class="text-sm font-semibold text-stone-700 dark:text-stone-300">
                        "Attachments"
                    </h2>
                </button>
            </div>

            {move || open.get().then(|| view! {
                <div class="mt-4">
                // Upload row — hidden native input, icon buttons
                <div class="flex items-center gap-1 mb-3">
                    <input
                        type="file"
                        node_ref=file_input_ref
                        on:change=on_file_change
                        class="hidden"
                    />
                    <button
                        class="p-1.5 rounded-lg text-stone-400 hover:text-stone-600
                            dark:hover:text-stone-300 hover:bg-stone-100
                            dark:hover:bg-stone-800 transition-colors"
                        on:click=on_pick
                        title="Choose file"
                    >
                        <span class="material-symbols-outlined" style="font-size: 16px;">
                            "folder_open"
                        </span>
                    </button>
                    <span class="flex-1 text-xs text-stone-500 dark:text-stone-400 truncate">
                        {move || selected_filename.get().unwrap_or_else(|| "No file chosen".to_string())}
                    </span>
                    <button
                        class="p-1.5 rounded-lg text-stone-400 hover:text-stone-600
                            dark:hover:text-stone-300 hover:bg-stone-100
                            dark:hover:bg-stone-800 transition-colors disabled:opacity-30"
                        on:click=on_upload
                        disabled=move || uploading.get()
                        title=move || if uploading.get() { "Uploading…" } else { "Upload" }
                    >
                        <span class="material-symbols-outlined" style="font-size: 16px;">
                            {move || if uploading.get() { "hourglass_empty" } else { "upload" }}
                        </span>
                    </button>
                </div>
                {move || error_msg.get().map(|msg| view! {
                    <div class="mt-1 mb-2 text-xs text-red-500">{msg}</div>
                })}

            <Suspense fallback=|| view! {
                <div class="text-xs text-stone-400">"Loading attachments..."</div>
            }>
                {move || {
                    attachments.get().map(|result| {
                        match result {
                            Ok(list) if list.is_empty() => view! {
                                <div class="flex flex-col items-center gap-2 py-6">
                                    <span
                                        class="material-symbols-outlined text-stone-300 dark:text-stone-700"
                                        style="font-size: 32px;"
                                    >
                                        "attach_file"
                                    </span>
                                    <p class="text-xs text-stone-400 dark:text-stone-600">
                                        "No attachments yet."
                                    </p>
                                </div>
                            }.into_any(),
                            Ok(list) => view! {
                                <div class="space-y-2">
                                    {list.into_iter().map(|att| {
                                        let att_id = att.id;
                                        let filename = att.filename.clone();
                                        let content_type = att.content_type.clone();
                                        let size_kb = att.size_bytes / 1024;
                                        let download_url = api::attachment_download_url(att_id);
                                        let previewable = is_previewable(&content_type);
                                        let is_image = content_type.starts_with("image/");
                                        let preview_open = RwSignal::new(false);
                                        let url_for_preview = download_url.clone();
                                        view! {
                                            <div class="rounded-lg border border-stone-200 dark:border-stone-700
                                                        bg-stone-50 dark:bg-stone-800/30">
                                                // Header row: icon + name + meta + actions
                                                <div class="flex items-center gap-2 px-3 py-2 group">
                                                    <span class="material-symbols-outlined text-stone-400
                                                                 dark:text-stone-500 shrink-0"
                                                          style="font-size: 16px;">
                                                        {if is_image { "image" } else if content_type == "application/pdf" { "picture_as_pdf" } else { "attach_file" }}
                                                    </span>
                                                    <span class="flex-1 text-xs text-stone-700 dark:text-stone-300
                                                                 truncate min-w-0">
                                                        {filename.clone()}
                                                    </span>
                                                    <span class="text-xs text-stone-400 dark:text-stone-500 shrink-0">
                                                        {format!("{size_kb} KB")}
                                                    </span>
                                                    // Preview toggle (only for previewable types)
                                                    {previewable.then(|| view! {
                                                        <button
                                                            class="p-0.5 text-stone-400 hover:text-amber-500
                                                                   dark:hover:text-amber-400 cursor-pointer
                                                                   transition-colors"
                                                            title=move || if preview_open.get() { "Hide preview" } else { "Show preview" }
                                                            on:click=move |_| preview_open.update(|v| *v = !*v)
                                                        >
                                                            <span class="material-symbols-outlined"
                                                                  style="font-size: 16px;">
                                                                {move || if preview_open.get() { "visibility_off" } else { "visibility" }}
                                                            </span>
                                                        </button>
                                                    })}
                                                    // Download link
                                                    <a
                                                        href=download_url
                                                        target="_blank"
                                                        class="p-0.5 text-stone-400 hover:text-amber-500
                                                               dark:hover:text-amber-400 transition-colors"
                                                        title="Download"
                                                    >
                                                        <span class="material-symbols-outlined"
                                                              style="font-size: 16px;">
                                                            "download"
                                                        </span>
                                                    </a>
                                                    // Delete button
                                                    <button
                                                        class="p-0.5 text-stone-400 hover:text-red-500
                                                               cursor-pointer transition-colors"
                                                        title="Delete attachment"
                                                        on:click=move |_| {
                                                            wasm_bindgen_futures::spawn_local(async move {
                                                                let _ = api::delete_attachment(att_id).await;
                                                                refresh.update(|n| *n += 1);
                                                            });
                                                        }
                                                    >
                                                        <span class="material-symbols-outlined"
                                                              style="font-size: 16px;">
                                                            "delete"
                                                        </span>
                                                    </button>
                                                </div>

                                                // Inline preview (toggled)
                                                {move || {
                                                    if !preview_open.get() { return None; }
                                                    Some(if is_image {
                                                        view! {
                                                            <div class="px-3 pb-3">
                                                                <img
                                                                    src=url_for_preview.clone()
                                                                    alt=filename.clone()
                                                                    class="max-w-full max-h-96 rounded-lg object-contain
                                                                           border border-stone-200 dark:border-stone-700"
                                                                />
                                                            </div>
                                                        }.into_any()
                                                    } else {
                                                        // PDF
                                                        view! {
                                                            <div class="px-3 pb-3">
                                                                <iframe
                                                                    src=url_for_preview.clone()
                                                                    class="w-full rounded-lg border border-stone-200
                                                                           dark:border-stone-700"
                                                                    style="height: 500px;"
                                                                    title=filename.clone()
                                                                />
                                                            </div>
                                                        }.into_any()
                                                    })
                                                }}
                                            </div>
                                        }
                                    }).collect::<Vec<_>>()}
                                </div>
                            }.into_any(),
                            Err(e) => view! {
                                <div class="text-xs text-red-500">{format!("Error: {e}")}</div>
                            }.into_any(),
                        }
                    })
                }}
            </Suspense>
            </div>  // close mt-4
            })}    // close open.then
        </div>
    }
}
