//! Node version history panel — shows previous body snapshots and lets editors
//! restore to any of them.
use common::id::NodeId;
use leptos::prelude::*;

use crate::api;
use crate::components::toast::{ToastLevel, push_toast};

/// Format a UTC timestamp as a concise local string via JS Intl.
fn format_ts(ts: &chrono::DateTime<chrono::Utc>) -> String {
    let iso = ts.to_rfc3339();
    let js = format!(
        "new Intl.DateTimeFormat(undefined, {{year:'numeric',month:'short',day:'numeric',\
         hour:'2-digit',minute:'2-digit'}}).format(new Date('{iso}'))"
    );
    js_sys::eval(&js)
        .ok()
        .and_then(|v| v.as_string())
        .unwrap_or_else(|| iso[..16].replace('T', " "))
}

#[component]
pub fn VersionPanel(
    node_id: NodeId,
    /// Only editors/owners may restore; viewers see the list read-only.
    is_editor: bool,
    /// Called after a successful restore so the parent can refresh the node body.
    on_restore: Callback<()>,
) -> impl IntoView {
    let open = RwSignal::new(false);
    let restoring: RwSignal<Option<uuid::Uuid>> = RwSignal::new(None);
    let refresh = RwSignal::new(0u32);

    let versions = LocalResource::new(move || async move {
        let _ = refresh.get(); // re-fetch after restore
        if !open.get() {
            return Ok(vec![]);
        }
        api::fetch_versions(node_id, Some(20)).await
    });

    let do_restore = move |version_id: uuid::Uuid| {
        restoring.set(Some(version_id));
        let on_restore = on_restore;
        spawn_local(async move {
            match api::restore_version(node_id, version_id).await {
                Ok(_) => {
                    push_toast("Body restored to selected version.".to_string(), ToastLevel::Success);
                    refresh.update(|n| *n += 1);
                    on_restore.run(());
                }
                Err(e) => {
                    push_toast(format!("Restore failed: {e}"), ToastLevel::Error);
                }
            }
            restoring.set(None);
        });
    };

    view! {
        <div class="mt-4 border-t border-stone-200 dark:border-stone-700 pt-6">
        <div class="border border-stone-200 dark:border-stone-700 rounded-lg overflow-hidden">
            // ── Header / toggle ────────────────────────────────────────────
            <button
                class="w-full flex items-center justify-between px-4 py-3 bg-stone-50 dark:bg-stone-800 hover:bg-stone-100 dark:hover:bg-stone-750 text-sm font-medium text-stone-700 dark:text-stone-300 transition-colors"
                on:click=move |_| open.update(|v| *v = !*v)
            >
                <span class="flex items-center gap-2">
                    <span class="material-symbols-outlined text-base">"history_edu"</span>
                    "Version History"
                </span>
                <span class="material-symbols-outlined text-base">
                    {move || if open.get() { "expand_less" } else { "expand_more" }}
                </span>
            </button>

            // ── Body ───────────────────────────────────────────────────────
            {move || open.get().then(|| view! {
                <div class="px-4 py-3">
                    <Suspense fallback=|| view! {
                        <p class="text-xs text-stone-400 dark:text-stone-500 animate-pulse py-2">"Loading versions…"</p>
                    }>
                        {move || versions.get().map(|result| match result {
                            Err(e) => view! {
                                <p class="text-xs text-red-500">{format!("Error: {e}")}</p>
                            }.into_any(),
                            Ok(list) if list.is_empty() => view! {
                                <p class="text-xs text-stone-400 dark:text-stone-500 py-2">
                                    "No saved versions yet. Versions are created automatically each time you save."
                                </p>
                            }.into_any(),
                            Ok(list) => view! {
                                <ol class="relative border-l border-stone-200 dark:border-stone-700 ml-2 space-y-4 py-1">
                                    {list.into_iter().enumerate().map(|(i, ver)| {
                                        let vid = ver.id.0;
                                        let ts = format_ts(&ver.created_at);
                                        let is_latest = i == 0;
                                        let busy = move || restoring.get() == Some(vid);
                                        view! {
                                            <li class="ml-4">
                                                // Timeline dot
                                                <span class=move || format!(
                                                    "absolute -left-2.5 h-5 w-5 rounded-full border-2 border-white dark:border-stone-900 {}",
                                                    if is_latest { "bg-amber-500" } else { "bg-stone-400 dark:bg-stone-600" }
                                                )></span>
                                                <div class="flex items-center justify-between gap-2">
                                                    <div class="text-xs leading-relaxed">
                                                        <span class=move || format!(
                                                            "font-medium {}",
                                                            if is_latest { "text-amber-600 dark:text-amber-400" } else { "text-stone-700 dark:text-stone-300" }
                                                        )>
                                                            {if is_latest { "Current version" } else { "Previous version" }}
                                                        </span>
                                                        <br />
                                                        <span class="text-stone-400 dark:text-stone-500 text-[11px]">{ts}</span>
                                                    </div>
                                                    {(!is_latest && is_editor).then(move || view! {
                                                        <button
                                                            class="text-xs px-2 py-1 rounded border border-stone-300 dark:border-stone-600 text-stone-600 dark:text-stone-400 hover:border-amber-500 hover:text-amber-600 dark:hover:text-amber-400 transition-colors disabled:opacity-40"
                                                            disabled=busy
                                                            on:click=move |_| do_restore(vid)
                                                        >
                                                            {move || if busy() { "Restoring…" } else { "Restore" }}
                                                        </button>
                                                    })}
                                                </div>
                                            </li>
                                        }
                                    }).collect::<Vec<_>>()}
                                </ol>
                            }.into_any(),
                        })}
                    </Suspense>
                </div>
            })}
        </div>
        </div>
    }
}
