use chrono::NaiveDate;
use common::task::{MyDayTask, Task, TaskPriority, TaskStatus, UpdateTaskRequest};
use leptos::prelude::*;

use crate::app::{TaskRefresh, View};

// ── Status helpers ────────────────────────────────────────────────────────────

fn status_done(s: &TaskStatus) -> bool {
    matches!(s, TaskStatus::Done | TaskStatus::Cancelled)
}

fn parse_status(s: &str) -> TaskStatus {
    match s {
        "in_progress" => TaskStatus::InProgress,
        "done"        => TaskStatus::Done,
        "cancelled"   => TaskStatus::Cancelled,
        _             => TaskStatus::Open,
    }
}

fn status_value(s: &TaskStatus) -> &'static str {
    match s {
        TaskStatus::Open       => "open",
        TaskStatus::InProgress => "in_progress",
        TaskStatus::Done       => "done",
        TaskStatus::Cancelled  => "cancelled",
    }
}

// ── Priority helpers ──────────────────────────────────────────────────────────

fn priority_weight(p: &TaskPriority) -> u8 {
    match p {
        TaskPriority::High   => 0,
        TaskPriority::Medium => 1,
        TaskPriority::Low    => 2,
    }
}

fn priority_value(p: &TaskPriority) -> &'static str {
    match p {
        TaskPriority::High   => "high",
        TaskPriority::Medium => "medium",
        TaskPriority::Low    => "low",
    }
}

fn parse_priority(s: &str) -> TaskPriority {
    match s {
        "high" => TaskPriority::High,
        "low"  => TaskPriority::Low,
        _      => TaskPriority::Medium,
    }
}

// ── Sort: incomplete first (high→med→low, due asc), done last ─────────────────

fn sort_tasks(tasks: &mut [Task]) {
    tasks.sort_by(|a, b| {
        let a_done = status_done(&a.status);
        let b_done = status_done(&b.status);
        match (a_done, b_done) {
            (true,  false) => std::cmp::Ordering::Greater,
            (false, true)  => std::cmp::Ordering::Less,
            _ => {
                let pw = priority_weight(&a.priority).cmp(&priority_weight(&b.priority));
                if pw != std::cmp::Ordering::Equal { return pw; }
                match (a.due_date, b.due_date) {
                    (Some(ad), Some(bd)) => ad.cmp(&bd),
                    (Some(_),  None)     => std::cmp::Ordering::Less,
                    (None,     Some(_))  => std::cmp::Ordering::Greater,
                    (None,     None)     => std::cmp::Ordering::Equal,
                }
            }
        }
    });
}

// ── MyDayView ─────────────────────────────────────────────────────────────────

#[component]
pub fn MyDayView() -> impl IntoView {
    let current_view = use_context::<RwSignal<View>>().expect("View signal must be provided");
    let refresh = use_context::<TaskRefresh>()
        .expect("TaskRefresh context must be provided")
        .0;

    let today      = chrono::Utc::now().date_naive();
    let date_label = today.format("%A, %B %-d").to_string();

    let tasks_resource = LocalResource::new(move || {
        let _ = refresh.get();
        async move { crate::api::fetch_my_day().await }
    });

    view! {
        <div class="flex flex-col h-full">

            // ── Header ────────────────────────────────────────────────────────
            <div class="px-6 py-4 border-b border-stone-200 dark:border-stone-800">
                <div class="flex items-center gap-3 mb-2">
                    <span class="material-symbols-outlined text-amber-500" style="font-size: 22px;">
                        "wb_sunny"
                    </span>
                    <div class="flex-1 min-w-0">
                        <div class="flex items-baseline justify-between gap-2">
                            <h1 class="text-lg font-semibold text-stone-900 dark:text-stone-100">
                                "My Day"
                            </h1>
                            // X / Y done counter
                            {move || {
                                let tasks = tasks_resource.get()
                                    .and_then(|r| r.ok())
                                    .unwrap_or_default();
                                let total = tasks.len();
                                if total == 0 { return None; }
                                let done = tasks.iter()
                                    .filter(|t| status_done(&t.task.status))
                                    .count();
                                Some(view! {
                                    <span class="text-xs text-stone-400 dark:text-stone-500 flex-shrink-0">
                                        {format!("{done} / {total} done")}
                                    </span>
                                })
                            }}
                        </div>
                        <p class="text-xs text-stone-400 dark:text-stone-500">{date_label}</p>
                    </div>
                </div>

                // ── Progress bar ──────────────────────────────────────────────
                {move || {
                    let tasks = tasks_resource.get()
                        .and_then(|r| r.ok())
                        .unwrap_or_default();
                    let total = tasks.len();
                    if total == 0 { return None; }
                    let done = tasks.iter()
                        .filter(|t| status_done(&t.task.status))
                        .count();
                    let pct = (done * 100) / total;
                    Some(view! {
                        <div class="h-1 bg-stone-100 dark:bg-stone-700 rounded-full overflow-hidden">
                            <div
                                class="h-full bg-amber-400 rounded-full transition-all duration-500"
                                style=format!("width: {pct}%")
                            />
                        </div>
                    })
                }}
            </div>

            // ── Content ───────────────────────────────────────────────────────
            <div class="flex-1 overflow-auto p-6 flex flex-col">
                <Suspense fallback=move || view! {
                    <p class="text-sm text-stone-400">"Loading…"</p>
                }>
                    {move || {
                        let raw_tasks = tasks_resource.get()
                            .and_then(|r| r.ok())
                            .unwrap_or_default();

                        if raw_tasks.is_empty() {
                            return view! {
                                <div class="flex-1 flex flex-col items-center justify-center gap-3 py-16">
                                    <span
                                        class="material-symbols-outlined text-stone-300 dark:text-stone-700"
                                        style="font-size: 48px;"
                                    >
                                        "wb_sunny"
                                    </span>
                                    <p class="text-stone-400 dark:text-stone-500 text-sm text-center">
                                        "No tasks for today."
                                    </p>
                                    <p class="text-stone-400 dark:text-stone-500 text-sm text-center max-w-xs">
                                        "Open a project and click the ☀ icon on a task to add it here."
                                    </p>
                                </div>
                            }.into_any();
                        }

                        // Group by node_id, then sort within each group
                        let mut grouped: Vec<(String, String, Vec<Task>)> = vec![];
                        for MyDayTask { task, node_title } in raw_tasks {
                            let node_id_str = task.node_id.to_string();
                            if let Some(g) = grouped.iter_mut().find(|(id, _, _)| id == &node_id_str) {
                                g.2.push(task);
                            } else {
                                grouped.push((node_id_str, node_title, vec![task]));
                            }
                        }
                        for (_, _, tasks) in grouped.iter_mut() {
                            sort_tasks(tasks);
                        }

                        view! {
                            <div class="space-y-8 max-w-2xl mx-auto w-full">
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

// ── MyDayGroup ────────────────────────────────────────────────────────────────

#[component]
fn MyDayGroup(
    node_id: common::id::NodeId,
    node_title: String,
    tasks: Vec<Task>,
    refresh: RwSignal<u32>,
    on_navigate: impl Fn() + Copy + 'static,
) -> impl IntoView {
    let _ = node_id;
    let show_done = RwSignal::new(false);

    // Split sorted tasks into active and done
    let (active, done): (Vec<Task>, Vec<Task>) =
        tasks.into_iter().partition(|t| !status_done(&t.status));
    let done_count  = done.len();
    let done_stored = StoredValue::new(done);

    view! {
        <div>
            // Clickable project header
            <button
                class="flex items-center gap-2 mb-2 text-xs font-semibold
                    text-stone-500 dark:text-stone-400 uppercase tracking-wider
                    hover:text-amber-600 dark:hover:text-amber-400 transition-colors"
                on:click=move |_| on_navigate()
            >
                <span class="material-symbols-outlined" style="font-size: 14px;">"rocket_launch"</span>
                {node_title}
                <span class="material-symbols-outlined" style="font-size: 12px;">"open_in_new"</span>
            </button>

            <div class="space-y-0.5 pl-2 border-l-2 border-stone-200 dark:border-stone-700">
                // Active tasks
                {active.into_iter().map(|task| {
                    view! { <MyDayTaskRow task=task refresh=refresh /> }
                }).collect_view()}

                // Completed tasks — collapsed by default
                {(done_count > 0).then(|| view! {
                    <div class="mt-1">
                        <button
                            class="flex items-center gap-1 text-xs text-stone-400
                                dark:text-stone-500 hover:text-stone-600 dark:hover:text-stone-300
                                transition-colors py-1 px-3"
                            on:click=move |_| show_done.update(|v| *v = !*v)
                        >
                            <span class="material-symbols-outlined" style="font-size: 13px;">
                                {move || if show_done.get() { "expand_more" } else { "chevron_right" }}
                            </span>
                            {move || if show_done.get() {
                                format!("Hide {done_count} completed")
                            } else {
                                format!("{done_count} completed")
                            }}
                        </button>
                        {move || show_done.get().then(|| {
                            done_stored.get_value().into_iter().map(|task| {
                                view! { <MyDayTaskRow task=task refresh=refresh /> }
                            }).collect_view()
                        })}
                    </div>
                })}
            </div>
        </div>
    }
}

// ── MyDayTaskRow ──────────────────────────────────────────────────────────────

#[component]
fn MyDayTaskRow(task: Task, refresh: RwSignal<u32>) -> impl IntoView {
    let task_id  = task.id;
    let node_id  = task.node_id;
    let today    = chrono::Utc::now().date_naive();

    let status_val   = RwSignal::new(status_value(&task.status).to_string());
    let priority_val = RwSignal::new(priority_value(&task.priority).to_string());

    let current_view = use_context::<RwSignal<View>>().expect("View signal must be provided");

    // Edit form state
    let editing       = RwSignal::new(false);
    let edit_title    = RwSignal::new(task.title.clone());
    let orig_title    = RwSignal::new(task.title.clone());
    let edit_due      = RwSignal::new(
        task.due_date
            .map(|d| d.format("%Y-%m-%d").to_string())
            .unwrap_or_default(),
    );
    let edit_priority = RwSignal::new(priority_value(&task.priority).to_string());

    let do_save = move || {
        let new_title = edit_title.get_untracked().trim().to_string();
        if new_title.is_empty() { return; }
        let new_priority = parse_priority(&edit_priority.get_untracked());
        editing.set(false);
        orig_title.set(new_title.clone());
        priority_val.set(priority_value(&new_priority).to_string());
        let new_due: Option<Option<NaiveDate>> =
            Some(edit_due.get_untracked().trim().parse::<NaiveDate>().ok());
        let req = UpdateTaskRequest {
            title:      Some(new_title),
            status:     None,
            priority:   Some(new_priority),
            focus_date: None,
            due_date:   new_due,
        };
        wasm_bindgen_futures::spawn_local(async move {
            let _ = crate::api::update_task(task_id, &req).await;
            refresh.update(|n| *n += 1);
        });
    };

    let on_toggle = move |_| {
        let current = status_val.get_untracked();
        let next    = if current == "done" { "open" } else { "done" };
        let req = UpdateTaskRequest {
            title: None, status: Some(parse_status(next)),
            priority: None, focus_date: None, due_date: None,
        };
        status_val.set(next.to_string());
        wasm_bindgen_futures::spawn_local(async move {
            let _ = crate::api::update_task(task_id, &req).await;
            refresh.update(|n| *n += 1);
        });
    };

    let on_remove = move |_| {
        let req = UpdateTaskRequest {
            title: None, status: None, priority: None,
            focus_date: Some(None), due_date: None,
        };
        wasm_bindgen_futures::spawn_local(async move {
            let _ = crate::api::update_task(task_id, &req).await;
            refresh.update(|n| *n += 1);
        });
    };

    let on_delete = move |_| {
        wasm_bindgen_futures::spawn_local(async move {
            let _ = crate::api::delete_task(task_id).await;
            refresh.update(|n| *n += 1);
        });
    };

    let overdue      = task.due_date
        .map(|d| !status_done(&task.status) && d < today)
        .unwrap_or(false);
    let due          = task.due_date;
    let carried_from = task.focus_date.filter(|&d| d < today);

    view! {
        <div class="group flex items-start gap-2 py-2 px-3 rounded-lg
            hover:bg-stone-50 dark:hover:bg-stone-800/50 transition-colors">

            // Checkbox
            <button
                class="flex-shrink-0 mt-0.5 w-5 h-5 rounded border-2 border-stone-300
                    dark:border-stone-600 flex items-center justify-center
                    hover:border-amber-500 transition-colors cursor-pointer"
                style=move || if status_done(&parse_status(&status_val.get())) {
                    "background:#d97706;border-color:#d97706;"
                } else { "" }
                on:click=on_toggle
                title="Toggle done"
            >
                {move || status_done(&parse_status(&status_val.get())).then(|| view! {
                    <span class="material-symbols-outlined text-white" style="font-size:13px;">"check"</span>
                })}
            </button>

            // Priority dot — High=red, Medium=amber, Low=hidden
            {move || match parse_priority(&priority_val.get()) {
                TaskPriority::High => Some(view! {
                    <span
                        class="flex-shrink-0 mt-1.5"
                        style="color:#ef4444;font-size:8px;line-height:1;"
                        title="High priority"
                    >{"●"}</span>
                }),
                TaskPriority::Medium => Some(view! {
                    <span
                        class="flex-shrink-0 mt-1.5"
                        style="color:#f59e0b;font-size:8px;line-height:1;"
                        title="Medium priority"
                    >{"●"}</span>
                }),
                TaskPriority::Low => None,
            }}

            // Body
            <div class="flex-1 min-w-0">
                {move || if editing.get() {
                    // ── Edit form ─────────────────────────────────────────────
                    view! {
                        <div class="space-y-2 pb-1">
                            // Title input
                            <input
                                type="text"
                                class="w-full bg-stone-100 dark:bg-stone-800 text-sm
                                    text-stone-900 dark:text-stone-100 rounded px-2 py-1
                                    focus:outline-none focus:ring-1 focus:ring-amber-500"
                                prop:value=move || edit_title.get()
                                on:input=move |ev| edit_title.set(event_target_value(&ev))
                                on:keydown=move |ev: leptos::ev::KeyboardEvent| {
                                    match ev.key().as_str() {
                                        "Enter"  => do_save(),
                                        "Escape" => {
                                            editing.set(false);
                                            edit_title.set(orig_title.get_untracked());
                                        }
                                        _ => {}
                                    }
                                }
                            />
                            // Priority selector
                            <div class="flex items-center gap-1.5">
                                <span class="text-xs text-stone-400 dark:text-stone-500">"Priority"</span>
                                {["low", "medium", "high"].iter().map(|&p| {
                                    let (label, sel_style) = match p {
                                        "high"   => ("High",   "color:#ef4444;border-color:#ef4444;"),
                                        "medium" => ("Medium", "color:#f59e0b;border-color:#f59e0b;"),
                                        _        => ("Low",    "color:#9ca3af;border-color:#9ca3af;"),
                                    };
                                    view! {
                                        <button
                                            class="text-xs px-2 py-0.5 rounded border transition-colors cursor-pointer"
                                            style=move || if edit_priority.get() == p {
                                                format!("{sel_style}font-weight:600;")
                                            } else {
                                                "color:#9ca3af;border-color:#d1d5db;".to_string()
                                            }
                                            on:click=move |_| edit_priority.set(p.to_string())
                                        >
                                            {label}
                                        </button>
                                    }
                                }).collect_view()}
                            </div>
                            // Due date + Save/Cancel
                            <div class="flex items-center gap-2">
                                <input
                                    type="date"
                                    class="text-xs bg-stone-100 dark:bg-stone-700
                                        text-stone-700 dark:text-stone-300
                                        rounded px-2 py-0.5 focus:outline-none
                                        focus:ring-1 focus:ring-amber-500"
                                    title="Due date (optional)"
                                    prop:value=move || edit_due.get()
                                    on:input=move |ev| edit_due.set(event_target_value(&ev))
                                />
                                <span class="flex-1"/>
                                <button
                                    class="text-xs bg-amber-500 hover:bg-amber-600 text-white
                                        rounded px-2.5 py-0.5 transition-colors cursor-pointer"
                                    on:click=move |_| do_save()
                                >
                                    "Save"
                                </button>
                                <button
                                    class="text-xs text-stone-400 hover:text-stone-700
                                        dark:hover:text-stone-200 transition-colors cursor-pointer"
                                    on:click=move |_| {
                                        editing.set(false);
                                        edit_title.set(orig_title.get_untracked());
                                    }
                                >
                                    "Cancel"
                                </button>
                            </div>
                        </div>
                    }.into_any()
                } else {
                    // ── Display row ───────────────────────────────────────────
                    view! {
                        <div class="flex items-center gap-2">
                            // Title — click navigates to the parent node
                            <span
                                class="flex-1 min-w-0 text-sm text-stone-800 dark:text-stone-200
                                    cursor-pointer hover:text-amber-600 dark:hover:text-amber-400
                                    transition-colors truncate"
                                style=move || if status_done(&parse_status(&status_val.get())) {
                                    "text-decoration:line-through;opacity:0.45;"
                                } else { "" }
                                on:click=move |_| current_view.set(View::NodeDetail(node_id))
                                title="Open parent node"
                            >
                                {move || orig_title.get()}
                            </span>

                            // Carried-over badge
                            {carried_from.map(|d| view! {
                                <span
                                    class="flex items-center gap-0.5 text-stone-400
                                        dark:text-stone-500 flex-shrink-0"
                                    style="font-size:11px;"
                                    title=format!("Carried over from {}", d.format("%b %-d"))
                                >
                                    <span class="material-symbols-outlined" style="font-size:12px;">"history"</span>
                                    {d.format("%b %-d").to_string()}
                                </span>
                            })}

                            // Due date
                            {due.map(|d| {
                                let style = if overdue {
                                    "color:#dc2626;font-size:11px;font-weight:600;"
                                } else {
                                    "color:#9ca3af;font-size:11px;"
                                };
                                let label = if overdue {
                                    format!("⚠ {}", d.format("%b %-d"))
                                } else {
                                    d.format("%b %-d").to_string()
                                };
                                view! {
                                    <span style=style class="flex-shrink-0">{label}</span>
                                }
                            })}

                            // Actions — muted until hover
                            <div class="flex items-center gap-0.5 flex-shrink-0">
                                <button
                                    class="p-1 rounded text-stone-300 dark:text-stone-600
                                        hover:text-stone-600 dark:hover:text-stone-300
                                        transition-colors cursor-pointer"
                                    title="Edit task"
                                    on:click=move |_| editing.set(true)
                                >
                                    <span class="material-symbols-outlined" style="font-size:15px;">"edit"</span>
                                </button>
                                <button
                                    class="p-1 rounded text-amber-300 dark:text-amber-700
                                        hover:text-amber-500 transition-colors cursor-pointer"
                                    title="Remove from My Day"
                                    on:click=on_remove
                                >
                                    <span class="material-symbols-outlined" style="font-size:15px;">"wb_sunny"</span>
                                </button>
                                <button
                                    class="p-1 rounded text-stone-300 dark:text-stone-600
                                        hover:text-red-500 transition-colors cursor-pointer"
                                    title="Delete task"
                                    on:click=on_delete
                                >
                                    <span class="material-symbols-outlined" style="font-size:15px;">"delete"</span>
                                </button>
                            </div>
                        </div>
                    }.into_any()
                }}
            </div>
        </div>
    }
}
