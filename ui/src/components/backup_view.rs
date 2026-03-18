//! Backup & Restore management UI.
//!
//! Accessible via the sidebar Backup link (admin only).
//! Shows a list of backup jobs with create / download / delete / restore actions.
//! Restore is a two-step wizard: preview → confirm.

use common::backup::{BackupJob, BackupPreview};
use leptos::prelude::*;
use uuid::Uuid;

use crate::{
    api,
    components::toast::{ToastLevel, push_toast},
};

// ── Component ─────────────────────────────────────────────────────────────────

#[component]
pub fn BackupView() -> impl IntoView {
    // Refresh counter — bump to re-fetch list.
    let refresh = RwSignal::new(0u32);
    let creating = RwSignal::new(false);

    // Restore wizard state.
    let restore_preview_job: RwSignal<Option<Uuid>> = RwSignal::new(None);
    let restore_preview: RwSignal<Option<BackupPreview>> = RwSignal::new(None);
    let previewing = RwSignal::new(false);
    let restoring = RwSignal::new(false);

    // Fetch backup list.
    let jobs = LocalResource::new(move || {
        let _ = refresh.get();
        async move { api::list_backups().await }
    });

    // Create backup handler.
    let on_create = move |_| {
        creating.set(true);
        wasm_bindgen_futures::spawn_local(async move {
            match api::create_backup_api().await {
                Ok(_) => {
                    push_toast(ToastLevel::Success, "Backup created successfully.");
                    refresh.update(|n| *n += 1);
                }
                Err(e) => push_toast(ToastLevel::Error, format!("Backup failed: {e}")),
            }
            creating.set(false);
        });
    };

    view! {
        <div class="p-6 max-w-4xl mx-auto">
            // ── Page header ───────────────────────────────────────────────────
            <div class="flex items-center justify-between mb-6">
                <div>
                    <h1 class="text-xl font-semibold text-stone-900 dark:text-stone-100">
                        "Backup & Restore"
                    </h1>
                    <p class="text-sm text-stone-500 dark:text-stone-400 mt-0.5">
                        "Create, manage and restore full data snapshots."
                    </p>
                </div>
                <button
                    class="flex items-center gap-2 px-3 py-2 rounded-lg bg-amber-600
                        text-white text-sm font-medium hover:bg-amber-700 disabled:opacity-40
                        transition-colors"
                    on:click=on_create
                    disabled=move || creating.get()
                >
                    <span class="material-symbols-outlined" style="font-size: 16px;">
                        "backup"
                    </span>
                    {move || if creating.get() { "Creating…" } else { "Create Backup" }}
                </button>
            </div>

            // ── Restore preview wizard ────────────────────────────────────────
            {move || restore_preview_job.get().map(|job_id| {
                view! {
                    <div class="mb-6 p-5 rounded-xl border border-amber-300 dark:border-amber-700
                        bg-amber-50 dark:bg-amber-950/30 space-y-3">
                        <h2 class="text-sm font-semibold text-amber-800 dark:text-amber-300">
                            "Restore Preview"
                        </h2>
                        {move || restore_preview.get().map(|preview| {
                            let counts = preview.entity_counts.clone();
                            view! {
                                <div class="space-y-2">
                                    <p class="text-sm text-stone-700 dark:text-stone-300">
                                        {format!(
                                            "This backup contains {} node(s), {} edge(s), \
                                             {} tag(s), {} note(s), {} task(s), {} attachment(s).",
                                            counts.nodes, counts.edges, counts.tags,
                                            counts.notes, counts.tasks, counts.attachments
                                        )}
                                    </p>
                                    {preview.warnings.into_iter().map(|w| view! {
                                        <div class="flex items-start gap-2 text-sm
                                            text-amber-700 dark:text-amber-400">
                                            <span class="material-symbols-outlined flex-shrink-0"
                                                style="font-size: 16px; margin-top: 1px;">
                                                "warning"
                                            </span>
                                            {w}
                                        </div>
                                    }).collect_view()}
                                    <div class="flex gap-2 pt-1">
                                        <button
                                            class="px-4 py-2 rounded-lg bg-red-600 text-white
                                                text-sm font-medium hover:bg-red-700
                                                disabled:opacity-40 transition-colors"
                                            disabled=move || restoring.get()
                                            on:click=move |_| {
                                                restoring.set(true);
                                                wasm_bindgen_futures::spawn_local(async move {
                                                    match api::restore_backup(job_id).await {
                                                        Ok(_) => {
                                                            push_toast(
                                                                ToastLevel::Success,
                                                                "Restore completed successfully.",
                                                            );
                                                            restore_preview_job.set(None);
                                                            restore_preview.set(None);
                                                            refresh.update(|n| *n += 1);
                                                        }
                                                        Err(e) => push_toast(
                                                            ToastLevel::Error,
                                                            format!("Restore failed: {e}"),
                                                        ),
                                                    }
                                                    restoring.set(false);
                                                });
                                            }
                                        >
                                            {move || if restoring.get() {
                                                "Restoring…"
                                            } else {
                                                "Confirm Restore"
                                            }}
                                        </button>
                                        <button
                                            class="px-4 py-2 rounded-lg text-sm text-stone-500
                                                hover:text-stone-700 dark:hover:text-stone-300
                                                hover:bg-stone-100 dark:hover:bg-stone-800
                                                transition-colors"
                                            on:click=move |_| {
                                                restore_preview_job.set(None);
                                                restore_preview.set(None);
                                            }
                                        >
                                            "Cancel"
                                        </button>
                                    </div>
                                </div>
                            }
                        })}
                        {move || previewing.get().then(|| view! {
                            <p class="text-sm text-stone-400 dark:text-stone-500">
                                "Loading preview…"
                            </p>
                        })}
                    </div>
                }
            })}

            // ── Backup list ───────────────────────────────────────────────────
            <Suspense fallback=|| view! {
                <div class="space-y-2">
                    {(0..3).map(|_| view! {
                        <div class="h-14 rounded-lg bg-stone-100 dark:bg-stone-800 animate-pulse" />
                    }).collect::<Vec<_>>()}
                </div>
            }>
                {move || {
                    jobs.get().map(|result| match result {
                        Err(e) => view! {
                            <div class="rounded-lg border border-red-200 dark:border-red-900
                                bg-red-50 dark:bg-red-950/30 px-4 py-3 text-sm
                                text-red-600 dark:text-red-400">
                                {format!("Error loading backups: {e}")}
                            </div>
                        }.into_any(),
                        Ok(list) if list.is_empty() => view! {
                            <div class="flex flex-col items-center gap-3 py-16 text-center">
                                <span class="material-symbols-outlined
                                    text-stone-300 dark:text-stone-700"
                                    style="font-size: 48px;">
                                    "backup"
                                </span>
                                <p class="text-sm text-stone-400 dark:text-stone-600">
                                    "No backups yet. Click \"Create Backup\" to get started."
                                </p>
                            </div>
                        }.into_any(),
                        Ok(list) => view! {
                            <div class="bg-white dark:bg-stone-900 rounded-xl
                                border border-stone-200 dark:border-stone-700
                                shadow-sm overflow-hidden">
                                // Table header
                                <div class="grid grid-cols-[1fr_auto_auto] gap-4 px-4 py-2.5
                                    border-b border-stone-200 dark:border-stone-800
                                    text-xs font-medium text-stone-500 dark:text-stone-400
                                    uppercase tracking-wide">
                                    <span>"Backup"</span>
                                    <span>"Size"</span>
                                    <span>"Actions"</span>
                                </div>
                                {list.into_iter().map(|job| {
                                    view! {
                                        <BackupRow
                                            job=job
                                            refresh=refresh
                                            restore_preview_job=restore_preview_job
                                            restore_preview=restore_preview
                                            previewing=previewing
                                        />
                                    }
                                }).collect::<Vec<_>>()}
                            </div>
                        }.into_any(),
                    })
                }}
            </Suspense>
        </div>
    }
}

// ── BackupRow ─────────────────────────────────────────────────────────────────

#[component]
fn BackupRow(
    job: BackupJob,
    refresh: RwSignal<u32>,
    restore_preview_job: RwSignal<Option<Uuid>>,
    restore_preview: RwSignal<Option<BackupPreview>>,
    previewing: RwSignal<bool>,
) -> impl IntoView {
    let job_id = job.id;
    let ts = job.created_at.format("%b %-d, %Y %H:%M UTC").to_string();
    let size_kb = job.size_bytes / 1024;
    let summary = format!(
        "{} nodes · {} edges · {} tags · {} notes · {} tasks · {} attachments",
        job.node_count,
        job.edge_count,
        job.tag_count,
        job.note_count,
        job.task_count,
        job.attachment_count
    );

    let confirm_delete = RwSignal::new(false);
    let deleting = RwSignal::new(false);

    view! {
        <div class="border-b border-stone-100 dark:border-stone-800 last:border-0
            px-4 py-3 flex items-start gap-4 group hover:bg-stone-50 dark:hover:bg-stone-800/40">
            // Left: info
            <div class="flex-1 min-w-0">
                <p class="text-sm font-medium text-stone-800 dark:text-stone-200">{ts}</p>
                <p class="text-xs text-stone-400 dark:text-stone-500 mt-0.5 truncate">{summary}</p>
            </div>

            // Size
            <span class="text-xs text-stone-400 dark:text-stone-500 whitespace-nowrap mt-0.5">
                {format!("{size_kb} KB")}
            </span>

            // Actions
            <div class="flex items-center gap-1 flex-shrink-0">
                // Download
                <button
                    class="p-1.5 rounded-lg text-stone-400 hover:text-amber-600 dark:hover:text-amber-400
                        hover:bg-amber-50 dark:hover:bg-amber-900/20 transition-colors"
                    title="Download backup"
                    on:click=move |_| {
                        wasm_bindgen_futures::spawn_local(async move {
                            match api::download_backup_url(job_id).await {
                                Ok(url) => {
                                    if let Some(win) = web_sys::window() {
                                        let _ = win.open_with_url_and_target(&url, "_blank");
                                    }
                                }
                                Err(e) => push_toast(
                                    ToastLevel::Error,
                                    format!("Download failed: {e}"),
                                ),
                            }
                        });
                    }
                >
                    <span class="material-symbols-outlined" style="font-size: 16px;">
                        "download"
                    </span>
                </button>

                // Restore (preview)
                <button
                    class="p-1.5 rounded-lg text-stone-400 hover:text-amber-600 dark:hover:text-amber-400
                        hover:bg-amber-50 dark:hover:bg-amber-900/20 transition-colors"
                    title="Restore this backup"
                    disabled=move || previewing.get()
                    on:click=move |_| {
                        previewing.set(true);
                        restore_preview.set(None);
                        restore_preview_job.set(Some(job_id));
                        wasm_bindgen_futures::spawn_local(async move {
                            match api::preview_backup_restore(job_id).await {
                                Ok(preview) => restore_preview.set(Some(preview)),
                                Err(e) => {
                                    push_toast(
                                        ToastLevel::Error,
                                        format!("Preview failed: {e}"),
                                    );
                                    restore_preview_job.set(None);
                                }
                            }
                            previewing.set(false);
                        });
                    }
                >
                    <span class="material-symbols-outlined" style="font-size: 16px;">
                        "restore"
                    </span>
                </button>

                // Delete (two-click confirm)
                {move || {
                    let jid = job_id;
                    if confirm_delete.get() {
                        view! {
                            <div class="flex items-center gap-1">
                                <span class="text-xs text-red-500 whitespace-nowrap">"Delete?"</span>
                                <button
                                    class="p-1.5 rounded-lg text-white bg-red-500
                                        hover:bg-red-600 transition-colors disabled:opacity-40"
                                    disabled=move || deleting.get()
                                    on:click=move |_| {
                                        deleting.set(true);
                                        wasm_bindgen_futures::spawn_local(async move {
                                            match api::delete_backup(jid).await {
                                                Ok(_) => {
                                                    push_toast(
                                                        ToastLevel::Success,
                                                        "Backup deleted.",
                                                    );
                                                    refresh.update(|n| *n += 1);
                                                }
                                                Err(e) => {
                                                    push_toast(
                                                        ToastLevel::Error,
                                                        format!("Delete failed: {e}"),
                                                    );
                                                    deleting.set(false);
                                                    confirm_delete.set(false);
                                                }
                                            }
                                        });
                                    }
                                >
                                    <span class="material-symbols-outlined"
                                        style="font-size: 14px;">"check"</span>
                                </button>
                                <button
                                    class="p-1.5 rounded-lg text-stone-400
                                        hover:text-stone-600 dark:hover:text-stone-300
                                        hover:bg-stone-100 dark:hover:bg-stone-800
                                        transition-colors"
                                    on:click=move |_| confirm_delete.set(false)
                                >
                                    <span class="material-symbols-outlined"
                                        style="font-size: 14px;">"close"</span>
                                </button>
                            </div>
                        }.into_any()
                    } else {
                        view! {
                            <button
                                class="p-1.5 rounded-lg text-stone-300 dark:text-stone-700
                                    hover:text-red-500 dark:hover:text-red-400
                                    hover:bg-red-50 dark:hover:bg-red-900/20 transition-colors"
                                title="Delete backup"
                                on:click=move |_| confirm_delete.set(true)
                            >
                                <span class="material-symbols-outlined" style="font-size: 16px;">
                                    "delete"
                                </span>
                            </button>
                        }.into_any()
                    }
                }}
            </div>
        </div>
    }
}
