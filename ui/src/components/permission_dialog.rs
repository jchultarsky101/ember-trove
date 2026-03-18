/// Permission panel — list, grant, and revoke per-node access.
///
/// When the "Add permission" form is opened, the panel attempts to fetch the
/// list of Keycloak users from `/admin/users` so the operator can pick by name
/// instead of pasting a raw UUID.  If the current user is not an admin (403)
/// the panel falls back gracefully to a plain text input.
use common::admin::AdminUser;
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

    // User picker state — loaded once when the form is first opened.
    let admin_users: RwSignal<Option<Vec<AdminUser>>> = RwSignal::new(None);
    let admin_api_available = RwSignal::new(true);

    let permissions = LocalResource::new(move || {
        let _ = refresh.get();
        let node_id = node_id;
        async move { api::list_permissions(node_id).await }
    });

    // Fetch admin users the first time the "Add permission" form is opened.
    let on_toggle_add = move |_| {
        let opening = !show_add.get_untracked();
        show_add.update(|v| *v = !*v);
        if opening && admin_users.get_untracked().is_none() {
            wasm_bindgen_futures::spawn_local(async move {
                match api::list_admin_users().await {
                    Ok(users) => {
                        admin_users.set(Some(users));
                        admin_api_available.set(true);
                    }
                    Err(crate::error::UiError::Api { status: 403, .. }) => {
                        // Not an admin — fall back to raw text input.
                        admin_users.set(Some(vec![]));
                        admin_api_available.set(false);
                    }
                    Err(_) => {
                        // Network error or other — fall back to raw text input.
                        admin_users.set(Some(vec![]));
                        admin_api_available.set(false);
                    }
                }
            });
        }
    };

    let on_grant = move |_| {
        let subject = subject_input.get_untracked().trim().to_string();
        if subject.is_empty() {
            error_msg.set(Some("Please select or enter a user.".to_string()));
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
                        "Sharing"
                    </h2>
                </button>
                {move || open.get().then(|| view! {
                    <button
                        class="p-1.5 rounded-lg text-stone-400 hover:text-stone-600
                            dark:hover:text-stone-300 hover:bg-stone-100
                            dark:hover:bg-stone-800 transition-colors"
                        on:click=on_toggle_add
                        title=move || if show_add.get() { "Cancel" } else { "Add permission" }
                    >
                        <span class="material-symbols-outlined" style="font-size: 16px;">
                            {move || if show_add.get() { "close" } else { "person_add" }}
                        </span>
                    </button>
                })}
            </div>

            {move || open.get().then(|| view! {
            <div class="mt-4">
            // Add permission form
            {move || show_add.get().then(|| view! {
                <div class="mb-4 p-3 bg-stone-50 dark:bg-stone-900 rounded-lg space-y-2">
                    <div class="flex gap-2">
                        // User picker: dropdown if admin API available, text input as fallback.
                        {move || {
                            let users_opt = admin_users.get();
                            match users_opt {
                                // Still loading
                                None => view! {
                                    <div class="flex-1 flex items-center px-2 py-1 text-xs text-stone-400">
                                        <span class="material-symbols-outlined mr-1"
                                            style="font-size: 14px;">"hourglass_empty"</span>
                                        "Loading users\u{2026}"
                                    </div>
                                }.into_any(),
                                // Loaded with users + admin API available: show select
                                Some(users) if admin_api_available.get() && !users.is_empty() => {
                                    view! {
                                        <select
                                            class="flex-1 px-2 py-1 text-xs rounded border border-stone-300 dark:border-stone-600
                                                bg-stone-50 dark:bg-stone-800 text-stone-700 dark:text-stone-300
                                                focus:outline-none focus:ring-1 focus:ring-amber-500"
                                            prop:value=move || subject_input.get()
                                            on:change=move |ev| subject_input.set(event_target_value(&ev))
                                        >
                                            <option value="">"\u{2014} Select a user \u{2014}"</option>
                                            {users.into_iter().map(|u| {
                                                let id = u.id.clone();
                                                let label = match (&u.first_name, &u.last_name, &u.email) {
                                                    (Some(f), Some(l), Some(e))
                                                        if !f.is_empty() || !l.is_empty() =>
                                                        format!("{} {} <{}>", f, l, e).trim().to_string(),
                                                    (_, _, Some(e)) =>
                                                        format!("{} <{}>", u.username, e),
                                                    _ => u.username.clone(),
                                                };
                                                view! {
                                                    <option value={id}>{label}</option>
                                                }
                                            }).collect::<Vec<_>>()}
                                        </select>
                                    }.into_any()
                                },
                                // Not admin or empty user list: raw text input fallback
                                _ => view! {
                                    <input
                                        type="text"
                                        class="flex-1 px-2 py-1 text-xs rounded border border-stone-300 dark:border-stone-600
                                            bg-transparent text-stone-900 dark:text-stone-100 focus:outline-none
                                            focus:ring-1 focus:ring-amber-500"
                                        placeholder="User subject ID (OIDC sub)\u{2026}"
                                        prop:value=move || subject_input.get()
                                        on:input=move |ev| subject_input.set(event_target_value(&ev))
                                    />
                                }.into_any(),
                            }
                        }}
                        <select
                            class="px-2 py-1 text-xs rounded border border-stone-300 dark:border-stone-600
                                bg-stone-50 dark:bg-stone-800 text-stone-700 dark:text-stone-300
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
                            class="p-1.5 rounded-lg text-stone-400 hover:text-stone-600
                                dark:hover:text-stone-300 hover:bg-stone-100
                                dark:hover:bg-stone-800 transition-colors disabled:opacity-30"
                            on:click=on_grant
                            disabled=move || saving.get()
                            title=move || if saving.get() { "Saving\u{2026}" } else { "Grant" }
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
                <div class="text-xs text-stone-400">"Loading..."</div>
            }>
                {move || {
                    permissions.get().map(|result| {
                        match result {
                            Ok(list) if list.is_empty() => view! {
                                <div class="flex flex-col items-center gap-2 py-6">
                                    <span
                                        class="material-symbols-outlined text-stone-300 dark:text-stone-700"
                                        style="font-size: 32px;"
                                    >
                                        "lock"
                                    </span>
                                    <p class="text-xs text-stone-400 dark:text-stone-600">
                                        "Only you have access."
                                    </p>
                                </div>
                            }.into_any(),
                            Ok(list) => view! {
                                <div class="space-y-1">
                                    {list.into_iter().map(|perm| {
                                        let perm_id = perm.id;
                                        let subject = perm.subject_id.clone();

                                        // Resolve display name from cached users list.
                                        let display_subject = {
                                            let sub = subject.clone();
                                            move || {
                                                if let Some(users) = admin_users.get()
                                                    && let Some(u) = users.iter().find(|u| u.id == sub) {
                                                        let name = u.display_name();
                                                        let email = u.email.as_deref().unwrap_or("");
                                                        if email.is_empty() {
                                                            return name;
                                                        }
                                                        return format!("{name} <{email}>");
                                                }
                                                sub.clone()
                                            }
                                        };

                                        let role_label = match perm.role {
                                            PermissionRole::Owner => "owner",
                                            PermissionRole::Editor => "editor",
                                            PermissionRole::Viewer => "viewer",
                                        };
                                        let role_color = match perm.role {
                                            PermissionRole::Owner =>
                                                "bg-purple-100 text-purple-700 dark:bg-purple-900/40 dark:text-purple-300",
                                            PermissionRole::Editor =>
                                                "bg-amber-100 text-amber-700 dark:bg-amber-900/40 dark:text-amber-300",
                                            PermissionRole::Viewer =>
                                                "bg-stone-100 text-stone-600 dark:bg-stone-800 dark:text-stone-400",
                                        };
                                        view! {
                                            <div class="flex items-center justify-between py-1.5 px-2 rounded
                                                hover:bg-stone-50 dark:hover:bg-stone-800/50 group">
                                                <div class="flex items-center gap-2 min-w-0">
                                                    <span class="material-symbols-outlined text-stone-400 dark:text-stone-600 text-[16px] shrink-0">
                                                        "person"
                                                    </span>
                                                    <span class="text-xs text-stone-700 dark:text-stone-300 truncate max-w-[180px]"
                                                        title={subject.clone()}>
                                                        {display_subject}
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
            </div>  // close mt-4
            })}    // close open.then
        </div>
    }
}
