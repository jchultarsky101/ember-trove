use common::task::{MyDayTask, Task, TaskStatus, UpdateTaskRequest};
use leptos::prelude::*;

use crate::app::{TaskRefresh, View};

fn status_done(s: &TaskStatus) -> bool {
    matches!(s, TaskStatus::Done | TaskStatus::Cancelled)
}

fn parse_status(s: &str) -> TaskStatus {
    match s {
        "in_progress" => TaskStatus::InProgress,
        "done" => TaskStatus::Done,
        "cancelled" => TaskStatus::Cancelled,
        _ => TaskStatus::Open,
    }
}

fn status_value(s: &TaskStatus) -> &'static str {
    match s {
        TaskStatus::Open => "open",
        TaskStatus::InProgress => "in_progress",
        TaskStatus::Done => "done",
        TaskStatus::Cancelled => "cancelled",
    }
}

#[component]
pub fn MyDayView() -> impl IntoView {
    let current_view = use_context::<RwSignal<View>>().expect("View signal must be provided");
    let refresh = use_context::<TaskRefresh>()
        .expect("TaskRefresh context must be provided")
        .0;

    let today = chrono::Utc::now().date_naive();
    let date_label = today.format("%A, %B %-d").to_string();

    let tasks_resource = LocalResource::new(move || {
        let _ = refresh.get();
        async move { crate::api::fetch_my_day().await }
    });

    view! {
        <div class="flex flex-col h-full">
            // Header
            <div class="flex items-center gap-3 px-6 py-4 border-b border-stone-200 dark:border-stone-800">
                <span class="material-symbols-outlined text-amber-500" style="font-size: 22px;">
                    {"wb_sunny"}
                </span>
                <div>
                    <h1 class="text-lg font-semibold text-stone-900 dark:text-stone-100">
                        "My Day"
                    </h1>
                    <p class="text-xs text-stone-400 dark:text-stone-500">{date_label}</p>
                </div>
            </div>

            // Content
            <div class="flex-1 overflow-auto p-6 flex flex-col">
                <Suspense fallback=move || view! {
                    <p class="text-sm text-stone-400">"Loading…"</p>
                }>
                    {move || {
                        let tasks = tasks_resource.get().and_then(|r| r.ok()).unwrap_or_default();

                        if tasks.is_empty() {
                            return view! {
                                <div class="flex-1 flex flex-col items-center justify-center gap-3">
                                    <span class="material-symbols-outlined text-stone-300 dark:text-stone-700"
                                        style="font-size: 48px;">{"wb_sunny"}</span>
                                    <p class="text-stone-400 dark:text-stone-500 text-sm text-center">
                                        "No tasks for today."
                                    </p>
                                    <p class="text-stone-400 dark:text-stone-500 text-sm text-center">
                                        "Open a project and click the ☀ icon on a task to add it here."
                                    </p>
                                </div>
                            }.into_any();
                        }

                        // Group by node_id, carrying the node_title from the first item
                        let mut grouped: Vec<(String, String, Vec<Task>)> = vec![];
                        for MyDayTask { task, node_title } in tasks {
                            let node_id_str = task.node_id.to_string();
                            if let Some(g) = grouped.iter_mut().find(|(id, _, _)| id == &node_id_str) {
                                g.2.push(task);
                            } else {
                                grouped.push((node_id_str, node_title, vec![task]));
                            }
                        }

                        view! {
                            <div class="space-y-6 max-w-2xl mx-auto w-full">
                                {grouped.into_iter().map(|(_, node_title, tasks)| {
                                    let node_id = tasks[0].node_id;
                                    view! {
                                        <MyDayGroup
                                            node_id=node_id
                                            node_title=node_title
                                            tasks=tasks
                                            refresh=refresh
                                            on_navigate=move || current_view.set(View::NodeDetail(node_id))
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

#[component]
fn MyDayGroup(
    node_id: common::id::NodeId,
    node_title: String,
    tasks: Vec<Task>,
    refresh: RwSignal<u32>,
    on_navigate: impl Fn() + Copy + 'static,
) -> impl IntoView {
    let _ = node_id;
    view! {
        <div>
            // Clickable project header — shows actual project title
            <button
                class="flex items-center gap-2 mb-2 text-xs font-semibold text-stone-500 dark:text-stone-400
                    uppercase tracking-wider hover:text-amber-600 dark:hover:text-amber-400 transition-colors"
                on:click=move |_| on_navigate()
            >
                <span class="material-symbols-outlined" style="font-size: 14px;">{"rocket_launch"}</span>
                {node_title}
                <span class="material-symbols-outlined" style="font-size: 12px;">{"open_in_new"}</span>
            </button>
            <div class="space-y-1 pl-2 border-l-2 border-stone-200 dark:border-stone-700">
                {tasks.into_iter().map(|task| {
                    view! { <MyDayTaskRow task=task refresh=refresh />}
                }).collect_view()}
            </div>
        </div>
    }
}

#[component]
fn MyDayTaskRow(task: Task, refresh: RwSignal<u32>) -> impl IntoView {
    let task_id = task.id;
    let title = task.title.clone();
    let status_val = RwSignal::new(status_value(&task.status).to_string());
    let today = chrono::Utc::now().date_naive();

    let on_toggle = move |_| {
        let current = status_val.get_untracked();
        let next = if current == "done" { "open" } else { "done" };
        let next_status = parse_status(next);
        let req = UpdateTaskRequest {
            title: None,
            status: Some(next_status),
            priority: None,
            focus_date: None,
            due_date: None,
        };
        status_val.set(next.to_string());
        wasm_bindgen_futures::spawn_local(async move {
            let _ = crate::api::update_task(task_id, &req).await;
            refresh.update(|n| *n += 1);
        });
    };

    // Remove from My Day
    let on_remove = move |_| {
        let req = UpdateTaskRequest {
            title: None,
            status: None,
            priority: None,
            focus_date: Some(None), // clear focus_date
            due_date: None,
        };
        wasm_bindgen_futures::spawn_local(async move {
            let _ = crate::api::update_task(task_id, &req).await;
            refresh.update(|n| *n += 1);
        });
    };

    let overdue = task.due_date.map(|d| !status_done(&task.status) && d < today).unwrap_or(false);
    let due = task.due_date;

    view! {
        <div class="group flex items-center gap-3 py-2.5 px-3 rounded-lg
            hover:bg-stone-50 dark:hover:bg-stone-800/50 transition-colors">
            // Checkbox
            <button
                class="flex-shrink-0 w-5 h-5 rounded border-2 border-stone-300 dark:border-stone-600
                    flex items-center justify-center hover:border-amber-500 transition-colors"
                style=move || if status_done(&parse_status(&status_val.get())) {
                    "background: #d97706; border-color: #d97706;"
                } else { "" }
                on:click=on_toggle
                title="Toggle done"
            >
                {move || status_done(&parse_status(&status_val.get())).then(|| view! {
                    <span class="material-symbols-outlined text-white" style="font-size: 13px;">{"check"}</span>
                })}
            </button>

            // Title
            <span
                class="flex-1 text-sm text-stone-800 dark:text-stone-200"
                style=move || if status_done(&parse_status(&status_val.get())) {
                    "text-decoration: line-through; opacity: 0.5;"
                } else { "" }
            >
                {title}
            </span>

            // Due date
            {due.map(|d| {
                let style = if overdue {
                    "color: #dc2626; font-size: 11px;"
                } else {
                    "color: #9ca3af; font-size: 11px;"
                };
                view! {
                    <span style=style>{d.format("%b %-d").to_string()}</span>
                }
            })}

            // Remove from My Day (hover)
            <button
                class="opacity-0 group-hover:opacity-100 text-stone-300 hover:text-amber-500
                    transition-all flex-shrink-0"
                title="Remove from My Day"
                on:click=on_remove
            >
                <span class="material-symbols-outlined" style="font-size: 16px;">{"wb_sunny"}</span>
            </button>
        </div>
    }
}
