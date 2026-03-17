/// Permission panel — list, grant, and revoke per-node access.
use common::id::NodeId;
use common::permission::{GrantPermissionRequest, PermissionRole};
use leptos::prelude::*;

use crate::api;

#[component]
pub fn PermissionPanel(node_id: NodeId) -> impl IntoView {
    let refresh = RwSignal::new(0u32);
    let show_add = RwSignal::new(false);
    let subject_input = RwSignal::new(String::new());
    let role_input = RwSignal::new("viewer".to_string());
    let error_msg = RwSignal::new(Option::<String>::None);
    let saving = RwSignal::new(false);

    let permissions = LocalResource::new(move || {
        let _ = refresh.get();
        let node_id = node_id;
        async move { api::list_permissions(node_id).await }
    });

    let on_grant = move |_| {
        let subject = subject_input.get_untracked().trim().to_string();
        if subject.is_empty() {
            error_msg.set(Some("Subject ID is required.".to_string()));
            return;
        }
        let role = match role_input.get_untracked().as_str() {
            "owner" => PermissionRole::Owner,
            "editor" => PermissionRole::Editor,
            _ => PermissionRole::Viewer,
        };
        error_msg.set(None);
        saving.set(true);
        let req = GrantPermissionRequest {
            subject_id: subject,
            role,
        };
        wasm_bindgen_futures::spawn_local(async move {
            match api::grant_permission(node_id, &req).await {
                Ok(_) => {
                    show_add.set(false);
                    subject_input.set(String::new());
                    role_input.set("viewer".to_string());
                    refresh.update(|n| *n += 1);
                }
                Err(e) => error_msg.set(Some(format!("Grant failed: {e}"))),
            }
            saving.set(false);
        });
    };

    view! {
        <div class="mt-8 border-t border-gray-200 dark:border-gray-700 pt-6">
            <div class="flex items-center justify-between mb-4">
                <h2 class="text-sm font-semibold text-gray-700 dark:text-gray-300">"Sharing"</h2>
                <button
                    class="p-1.5 rounded-lg text-gray-400 hover:text-gray-600
                        dark:hover:text-gray-300 hover:bg-gray-100
                        dark:hover:bg-gray-800 transition-colors"
                    on:click=move |_| show_add.update(|v| *v = !*v)
                    title=move || if show_add.get() { "Cancel" } else { "Add permission" }
                >
                    <span class="material-symbols-outlined" style="font-size: 16px;">
                        {move || if show_add.get() { "close" } else { "person_add" }}
                    </span>
                </button>
            </div>

            // Add permission form
            {move || show_add.get().then(|| view! {
                <div class="mb-4 p-3 bg-gray-50 dark:bg-gray-900 rounded-lg space-y-2">
                    <div class="flex gap-2">
                        <input
                            type="text"
                            class="flex-1 px-2 py-1 text-xs rounded border border-gray-300 dark:border-gray-600
                                bg-transparent text-gray-900 dark:text-gray-100 focus:outline-none
                                focus:ring-1 focus:ring-blue-500"
                            placeholder="User subject ID (from OIDC sub)…"
                            prop:value=move || subject_input.get()
                            on:input=move |ev| subject_input.set(event_target_value(&ev))
                        />
                        <select
                            class="px-2 py-1 text-xs rounded border border-gray-300 dark:border-gray-600
                                bg-gray-50 dark:bg-gray-800 text-gray-700 dark:text-gray-300
                                focus:outline-none"
                            prop:value=move || role_input.get()
                            on:change=move |ev| role_input.set(event_target_value(&ev))
                        >
                            <option value="viewer">"Viewer"</option>
                            <option value="editor">"Editor"</option>
                            <option value="owner">"Owner"</option>
                        </select>
                    </div>
                    <div class="flex items-center gap-2">
                        <button
                            class="p-1.5 rounded-lg text-gray-400 hover:text-gray-600
                                dark:hover:text-gray-300 hover:bg-gray-100
                                dark:hover:bg-gray-800 transition-colors disabled:opacity-30"
                            on:click=on_grant
                            disabled=move || saving.get()
                            title=move || if saving.get() { "Saving…" } else { "Grant" }
                        >
                            <span class="material-symbols-outlined" style="font-size: 16px;">
                                {move || if saving.get() { "hourglass_empty" } else { "check" }}
                            </span>
                        </button>
                    </div>
                    {move || error_msg.get().map(|msg| view! {
                        <div class="text-xs text-red-500">{msg}</div>
                    })}
                </div>
            })}

            // Permission list
            <Suspense fallback=|| view! {
                <div class="text-xs text-gray-400">"Loading..."</div>
            }>
                {move || {
                    permissions.get().map(|result| {
                        match result {
                            Ok(list) if list.is_empty() => view! {
                                <div class="flex flex-col items-center gap-2 py-6">
                                    <span
                                        class="material-symbols-outlined text-gray-300 dark:text-gray-700"
                                        style="font-size: 32px;"
                                    >
                                        "lock"
                                    </span>
                                    <p class="text-xs text-gray-400 dark:text-gray-600">
                                        "Only you have access."
                                    </p>
                                </div>
                            }.into_any(),
                            Ok(list) => view! {
                                <div class="space-y-1">
                                    {list.into_iter().map(|perm| {
                                        let perm_id = perm.id;
                                        let subject = perm.subject_id.clone();
                                        let role_label = match perm.role {
                                            PermissionRole::Owner => "owner",
                                            PermissionRole::Editor => "editor",
                                            PermissionRole::Viewer => "viewer",
                                        };
                                        let role_color = match perm.role {
                                            PermissionRole::Owner =>
                                                "bg-purple-100 text-purple-700 dark:bg-purple-900/40 dark:text-purple-300",
                                            PermissionRole::Editor =>
                                                "bg-blue-100 text-blue-700 dark:bg-blue-900/40 dark:text-blue-300",
                                            PermissionRole::Viewer =>
                                                "bg-gray-100 text-gray-600 dark:bg-gray-800 dark:text-gray-400",
                                        };
                                        view! {
                                            <div class="flex items-center justify-between py-1.5 px-2 rounded
                                                hover:bg-gray-50 dark:hover:bg-gray-800/50 group">
                                                <div class="flex items-center gap-2 min-w-0">
                                                    <span class="material-symbols-outlined text-gray-400 dark:text-gray-600 text-[16px] shrink-0">
                                                        "person"
                                                    </span>
                                                    <span class="text-xs text-gray-700 dark:text-gray-300 font-mono truncate max-w-[180px]">
                                                        {subject}
                                                    </span>
                                                    <span class={format!("px-1.5 py-0.5 text-[10px] rounded-full font-medium shrink-0 {role_color}")}>
                                                        {role_label}
                                                    </span>
                                                </div>
                                                <button
                                                    class="opacity-0 group-hover:opacity-100 text-red-400 hover:text-red-600
                                                        text-xs transition-opacity shrink-0"
                                                    on:click=move |_| {
                                                        wasm_bindgen_futures::spawn_local(async move {
                                                            let _ = api::revoke_permission(node_id, perm_id).await;
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
