use common::task::{ProjectDashboardEntry, TaskSummary};
use leptos::prelude::*;

use crate::app::TaskRefresh;
use crate::components::format_helpers::format_relative_short;
use crate::components::node_meta::{status_color, status_icon, status_label};
use crate::components::task_common::priority_color_hex;
use crate::markdown::render_markdown_plain;
use leptos_router::hooks::use_navigate;

#[component]
pub fn ProjectDashboard() -> impl IntoView {
    let navigate = StoredValue::new(use_navigate());

    let task_refresh = use_context::<TaskRefresh>()
        .expect("TaskRefresh context must be provided")
        .0;

    let entries = LocalResource::new(move || {
        let _ = task_refresh.get();
        async move { crate::api::fetch_project_dashboard().await }
    });

    view! {
        <div class="flex flex-col h-full">
            // Header
            <div class="flex items-center gap-3 px-6 py-4 border-b border-stone-200 dark:border-stone-800">
                <span class="material-symbols-outlined text-amber-500" style="font-size: 22px;">
                    {"rocket_launch"}
                </span>
                <h1 class="text-lg font-semibold text-stone-900 dark:text-stone-100">
                    "Project Dashboard"
                </h1>
            </div>

            // Content
            <div class="flex-1 overflow-auto p-6 flex flex-col">
                <Suspense fallback=move || view! {
                    <crate::components::skeleton::SkeletonCards cards=3 />
                }>
                    {move || {
                        let data = entries.get().and_then(|r| r.ok()).unwrap_or_default();
                        if data.is_empty() {
                            return view! {
                                <div class="flex-1 flex flex-col items-center justify-center gap-3">
                                    <span class="material-symbols-outlined text-stone-300 dark:text-stone-700"
                                        style="font-size: 48px;">{"rocket_launch"}</span>
                                    <p class="text-stone-400 dark:text-stone-500 text-sm">
                                        "No projects yet. Create a Project node to get started."
                                    </p>
                                </div>
                            }.into_any();
                        }

                        view! {
                            <div class="space-y-4">
                                {data.into_iter().map(|entry| {
                                    let node_id = entry.node_id;
                                    let nav = navigate;
                                    view! {
                                        <ProjectCard
                                            entry=entry
                                            on_navigate=move || nav.get_value()(&format!("/nodes/{node_id}"), Default::default())
                                        />
                                    }
                                }).collect_view()}
                            </div>
                        }.into_any()
                    }}
                </Suspense>
            </div>
        </div>
    }
}

// ── Project Card ───────────────────────��─────────────────────────────────────

#[component]
fn ProjectCard(
    entry: ProjectDashboardEntry,
    on_navigate: impl Fn() + 'static,
) -> impl IntoView {
    let counts = &entry.task_counts;
    let total = counts.open + counts.in_progress + counts.done + counts.cancelled;
    let done_pct = if total > 0 {
        (counts.done * 100 / total) as f32
    } else {
        0.0
    };

    let s_icon = status_icon(&entry.node_status);
    let s_label = status_label(&entry.node_status);
    let s_color = status_color(&entry.node_status);

    let status_html = entry.status_section.as_deref().map(render_markdown_plain);
    let open_tasks = entry.open_tasks;
    let has_more = entry.has_more_tasks;
    let open_count = counts.open + counts.in_progress;
    let updated_label = format_relative_short(&entry.last_activity_at);

    view! {
        <div
            class="rounded-lg border border-stone-200 dark:border-stone-800
                bg-stone-50 dark:bg-stone-800/40
                hover:border-amber-300 dark:hover:border-amber-700
                transition-colors cursor-pointer group overflow-hidden"
            on:click=move |_| on_navigate()
        >
            // ── Top bar: title + status + counts + progress ──────────────
            // Mobile: title row stacks above a wrapping meta row so the title
            // is never crushed by the badges/progress on narrow viewports.
            // Desktop (sm+): single horizontal row as before.
            <div class="flex flex-col sm:flex-row sm:items-center gap-2 sm:gap-4 px-4 py-3">
                // Title
                <div class="flex items-center gap-2 min-w-0 sm:flex-1">
                    <span class="material-symbols-outlined text-amber-500 flex-shrink-0"
                        style="font-size: 16px;">{"rocket_launch"}</span>
                    <span class="font-medium text-stone-800 dark:text-stone-200 truncate
                        group-hover:text-amber-700 dark:group-hover:text-amber-400 transition-colors">
                        {entry.title}
                    </span>
                </div>

                // Meta row: status + activity + counts + progress.
                // On mobile this wraps under the title; on sm+ it sits inline.
                <div class="flex items-center flex-wrap gap-x-3 gap-y-2 sm:gap-4 sm:flex-nowrap">
                    // Node status badge
                    <div class="flex items-center gap-1 text-sm flex-shrink-0" style=s_color>
                        <span class="material-symbols-outlined" style="font-size: 15px;">{s_icon}</span>
                        <span class="text-xs">{s_label}</span>
                    </div>

                    // Last-activity label
                    <div class="hidden sm:flex items-center gap-1 text-xs
                        text-stone-500 dark:text-stone-400 flex-shrink-0"
                        title="Most recent activity across the project and its tasks">
                        <span class="material-symbols-outlined" style="font-size: 14px;">{"history"}</span>
                        <span>{updated_label}</span>
                    </div>

                    // Count badges
                    <div class="flex items-center gap-1 flex-shrink-0">
                        <CountBadge count=counts.open
                            color="bg-stone-200 dark:bg-stone-700 text-stone-700 dark:text-stone-300" />
                        <CountBadge count=counts.in_progress
                            color="bg-amber-100 dark:bg-amber-900/30 text-amber-700 dark:text-amber-400" />
                        <CountBadge count=counts.done
                            color="bg-green-100 dark:bg-green-900/30 text-green-700 dark:text-green-400" />
                        <CountBadge count=counts.cancelled
                            color="bg-stone-100 dark:bg-stone-800 text-stone-400 dark:text-stone-500" />
                    </div>

                    // Progress
                    <div class="flex items-center gap-2 w-28 flex-shrink-0">
                        {if total == 0 {
                            view! {
                                <span class="text-xs text-stone-400 dark:text-stone-500 italic">"No tasks"</span>
                            }.into_any()
                        } else {
                            view! {
                                <div class="flex items-center gap-2 w-full">
                                    <div class="flex-1 h-1.5 bg-stone-200 dark:bg-stone-700 rounded-full overflow-hidden">
                                        <div
                                            class="h-full bg-green-500 rounded-full transition-all"
                                            style=format!("width: {done_pct:.0}%")
                                        />
                                    </div>
                                    <span class="text-xs text-stone-500 dark:text-stone-400 flex-shrink-0">
                                        {format!("{done_pct:.0}%")}
                                    </span>
                                </div>
                            }.into_any()
                        }}
                    </div>
                </div>
            </div>

            // ── Expandable detail section ────────────────────────────────
            {(status_html.is_some() || !open_tasks.is_empty()).then(|| {
                view! {
                    <div class="border-t border-stone-200 dark:border-stone-700/60 px-4 py-3
                        grid grid-cols-1 md:grid-cols-2 gap-4">

                        // Status section (left column)
                        {status_html.map(|html| view! {
                            <div>
                                <div class="text-xs font-semibold uppercase tracking-wider
                                    text-stone-400 dark:text-stone-500 mb-1.5">
                                    "Status"
                                </div>
                                <div
                                    class="prose prose-sm dark:prose-invert max-w-none
                                        text-stone-600 dark:text-stone-300
                                        [&_ul]:list-disc [&_ul]:pl-4 [&_ol]:list-decimal [&_ol]:pl-4
                                        [&_li]:my-0.5 [&_p]:my-1"
                                    inner_html=html
                                />
                            </div>
                        })}

                        // Open tasks (right column)
                        {(!open_tasks.is_empty()).then(|| view! {
                            <div>
                                <div class="text-xs font-semibold uppercase tracking-wider
                                    text-stone-400 dark:text-stone-500 mb-1.5">
                                    {format!("Open Tasks ({})", open_count)}
                                </div>
                                <div class="space-y-1">
                                    {open_tasks.into_iter().map(|t| view! {
                                        <TaskChip task=t />
                                    }).collect_view()}
                                    {has_more.then(|| view! {
                                        <div class="text-xs text-stone-400 dark:text-stone-500 italic pl-5 pt-0.5">
                                            "\u{2026}and more"
                                        </div>
                                    })}
                                </div>
                            </div>
                        })}
                    </div>
                }
            })}
        </div>
    }
}

// ── Task chip (compact read-only task row) ──────��────────────────────────────

#[component]
fn TaskChip(task: TaskSummary) -> impl IntoView {
    let color = priority_color_hex(&task.priority);
    let due_label = task
        .due_date
        .map(|d| d.format("%b %-d").to_string())
        .unwrap_or_default();
    let has_due = task.due_date.is_some();
    let in_progress = matches!(task.status, common::task::TaskStatus::InProgress);

    view! {
        <div class="flex items-center gap-1.5 text-xs py-0.5 group/task">
            // Priority dot
            <span
                class="w-2 h-2 rounded-full flex-shrink-0"
                style=format!("background-color: {color};")
            />
            // Title
            <span class="truncate text-stone-700 dark:text-stone-300 flex-1 min-w-0">
                {task.title}
            </span>
            // In-progress badge
            {in_progress.then(|| view! {
                <span class="text-[10px] px-1.5 py-0.5 rounded
                    bg-amber-100 dark:bg-amber-900/30
                    text-amber-700 dark:text-amber-400 flex-shrink-0">
                    "In Progress"
                </span>
            })}
            // Due date
            {has_due.then(|| view! {
                <span class="text-stone-400 dark:text-stone-500 flex-shrink-0 whitespace-nowrap">
                    {due_label}
                </span>
            })}
        </div>
    }
}

// ── Count badge ───────��─────────────────────────────���────────────────────────

#[component]
fn CountBadge(count: u32, color: &'static str) -> impl IntoView {
    view! {
        <span class=format!("inline-flex items-center justify-center w-7 h-5 rounded text-xs font-semibold {color}")>
            {count.to_string()}
        </span>
    }
}
