/// Attachment panel — list, upload, and delete files attached to a node.
use common::id::NodeId;
use leptos::{html::Input, prelude::*};
use wasm_bindgen::JsCast;

use crate::api;

#[component]
pub fn AttachmentPanel(node_id: NodeId) -> impl IntoView {
    let refresh = RwSignal::new(0u32);
    let uploading = RwSignal::new(false);
    let error_msg = RwSignal::new(Option::<String>::None);
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
                    if let Some(el) = file_input_ref.get_untracked() {
                        el.set_value("");
                    }
                }
                Err(e) => error_msg.set(Some(format!("Upload failed: {e}"))),
            }
            uploading.set(false);
        });
    };

    view! {
        <div class="mt-8 border-t border-gray-200 dark:border-gray-700 pt-6">
            <div class="mb-4">
                <h2 class="text-sm font-semibold text-gray-700 dark:text-gray-300 mb-3">
                    "Attachments"
                </h2>
                // Upload row
                <div class="flex items-center gap-2">
                    <input
                        type="file"
                        node_ref=file_input_ref
                        class="flex-1 text-xs text-gray-700 dark:text-gray-300
                            file:mr-2 file:py-1 file:px-2 file:rounded file:border-0
                            file:text-xs file:bg-blue-50 file:text-blue-700
                            dark:file:bg-blue-900/30 dark:file:text-blue-300
                            hover:file:bg-blue-100 dark:hover:file:bg-blue-900/50"
                    />
                    <button
                        class="p-1.5 rounded-lg text-gray-400 hover:text-gray-600
                            dark:hover:text-gray-300 hover:bg-gray-100
                            dark:hover:bg-gray-800 transition-colors disabled:opacity-30"
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
                    <div class="mt-1 text-xs text-red-500">{msg}</div>
                })}
            </div>

            <Suspense fallback=|| view! {
                <div class="text-xs text-gray-400">"Loading attachments..."</div>
            }>
                {move || {
                    attachments.get().map(|result| {
                        match result {
                            Ok(list) if list.is_empty() => view! {
                                <div class="flex flex-col items-center gap-2 py-6">
                                    <span
                                        class="material-symbols-outlined text-gray-300 dark:text-gray-700"
                                        style="font-size: 32px;"
                                    >
                                        "attach_file"
                                    </span>
                                    <p class="text-xs text-gray-400 dark:text-gray-600">
                                        "No attachments yet."
                                    </p>
                                </div>
                            }.into_any(),
                            Ok(list) => view! {
                                <div class="space-y-1">
                                    {list.into_iter().map(|att| {
                                        let att_id = att.id;
                                        let filename = att.filename.clone();
                                        let content_type = att.content_type.clone();
                                        let size_kb = att.size_bytes / 1024;
                                        let download_url = api::attachment_download_url(att_id);
                                        view! {
                                            <div class="flex items-center justify-between py-1.5 px-2 rounded
                                                hover:bg-gray-50 dark:hover:bg-gray-800/50 group">
                                                <a
                                                    href=download_url
                                                    target="_blank"
                                                    class="flex items-center gap-2 text-xs text-blue-600 dark:text-blue-400
                                                        hover:underline min-w-0"
                                                >
                                                    <span class="material-symbols-outlined text-[14px] shrink-0">
                                                        "attach_file"
                                                    </span>
                                                    <span class="truncate max-w-[200px]">{filename}</span>
                                                    <span class="text-gray-400 dark:text-gray-500 shrink-0">
                                                        {format!("({content_type}, {size_kb} KB)")}
                                                    </span>
                                                </a>
                                                <button
                                                    class="opacity-0 group-hover:opacity-100 text-red-400 hover:text-red-600
                                                        text-xs transition-opacity shrink-0"
                                                    on:click=move |_| {
                                                        wasm_bindgen_futures::spawn_local(async move {
                                                            let _ = api::delete_attachment(att_id).await;
                                                            refresh.update(|n| *n += 1);
                                                        });
                                                    }
                                                >
                                                    "\u{00d7}"
                                                </button>
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
        </div>
    }
}
