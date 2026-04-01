use chrono::NaiveDate;
use common::{
    id::NodeId,
    task::{CreateTaskRequest, Task, TaskPriority, TaskStatus, UpdateTaskRequest},
};
use leptos::prelude::*;

use crate::app::TaskRefresh;

fn priority_icon(p: &TaskPriority) -> &'static str {
    match p {
        TaskPriority::High => "keyboard_double_arrow_up",
        TaskPriority::Medium => "drag_handle",
        TaskPriority::Low => "keyboard_double_arrow_down",
    }
}

fn priority_color(p: &TaskPriority) -> &'static str {
    match p {
        TaskPriority::High => "color: #dc2626;",
        TaskPriority::Medium => "color: #d97706;",
        TaskPriority::Low => "color: #6b7280;",
    }
}

fn priority_label(p: &TaskPriority) -> &'static str {
    match p {
        TaskPriority::High => "High",
        TaskPriority::Medium => "Medium",
        TaskPriority::Low => "Low",
    }
}

fn status_done(s: &TaskStatus) -> bool {
    matches!(s, TaskStatus::Done | TaskStatus::Cancelled)
}

fn status_label(s: &TaskStatus) -> &'static str {
    match s {
        TaskStatus::Open => "Open",
        TaskStatus::InProgress => "In Progress",
        TaskStatus::Done => "Done",
        TaskStatus::Cancelled => "Cancelled",
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

fn parse_status(s: &str) -> TaskStatus {
    match s {
        "in_progress" => TaskStatus::InProgress,
        "done" => TaskStatus::Done,
        "cancelled" => TaskStatus::Cancelled,
        _ => TaskStatus::Open,
    }
}

fn parse_priority(s: &str) -> TaskPriority {
    match s {
        "high" => TaskPriority::High,
        "low" => TaskPriority::Low,
        _ => TaskPriority::Medium,
    }
}

// ── TaskPanel ─────────────────────────────────────────────────────────────────

#[component]
pub fn TaskPanel(node_id: NodeId) -> impl IntoView {
    let task_refresh = use_context::<TaskRefresh>()
        .expect("TaskRefresh context must be provided")
        .0;

    let tasks_resource = LocalResource::new(move || {
        let _ = task_refresh.get();
        async move { crate::api::fetch_tasks(node_id).await }
    });

    // Filter: hide done/cancelled tasks by default; toggled by the user.
    let show_completed = RwSignal::new(false);

    // New task form state
    let new_title = RwSignal::new(String::new());
    let new_priority = RwSignal::new("medium".to_string());
    let new_due = RwSignal::new(String::new());
    let adding = RwSignal::new(false);
    let show_form = RwSignal::new(false);
    let add_error = RwSignal::new(Option::<String>::None);

    let do_add = move || {
        let title = new_title.get_untracked().trim().to_string();
        if title.is_empty() {
            add_error.set(Some("Title is required.".to_string()));
            return;
        }
        let priority = parse_priority(&new_priority.get_untracked());
        let due_date = new_due
            .get_untracked()
            .trim()
            .parse::<NaiveDate>()
            .ok();
        adding.set(true);
        add_error.set(None);
        wasm_bindgen_futures::spawn_local(async move {
            let req = CreateTaskRequest {
                title,
                status: None,
                priority: Some(priority),
                focus_date: None,
                due_date,
            };
            match crate::api::create_task(node_id, &req).await {
                Ok(_) => {
                    new_title.set(String::new());
                    new_priority.set("medium".to_string());
                    new_due.set(String::new());
                    show_form.set(false);
                    task_refresh.update(|n| *n += 1);
                }
                Err(e) => {
                    add_error.set(Some(format!("{e}")));
                }
            }
            adding.set(false);
        });
    };
    let on_add = move |_| do_add();

    view! {
        <div class="mt-8 border-t border-stone-200 dark:border-stone-700 pt-6">
            // Section header
            <div class="flex items-center justify-between mb-4">
                <h3 class="text-sm font-semibold text-stone-500 dark:text-stone-400 uppercase tracking-wider flex items-center gap-2">
                    <span class="material-symbols-outlined text-base">{"check_box"}</span>
                    "Tasks"
                </h3>
                <button
                    class="flex items-center gap-1 text-xs text-amber-600 dark:text-amber-400
                        hover:text-amber-700 dark:hover:text-amber-300 transition-colors"
                    on:click=move |_| show_form.update(|v| *v = !*v)
                >
                    <span class="material-symbols-outlined text-sm">
                        {move || if show_form.get() { "remove" } else { "add" }}
                    </span>
                    {move || if show_form.get() { "Cancel" } else { "Add task" }}
                </button>
            </div>

            // Add-task form
            {move || show_form.get().then(|| view! {
                <div class="mb-4 p-3 rounded-lg bg-stone-50 dark:bg-stone-800/50
                    border border-stone-200 dark:border-stone-700 space-y-2">
                    <input
                        type="text"
                        placeholder="Task title…"
                        class="w-full bg-transparent text-sm text-stone-900 dark:text-stone-100
                            focus:outline-none placeholder-stone-400"
                        prop:value=move || new_title.get()
                        on:input=move |ev| new_title.set(event_target_value(&ev))
                        on:keydown=move |ev: leptos::ev::KeyboardEvent| {
                            if ev.key() == "Enter" { do_add(); }
                        }
                    />
                    <div class="flex items-center gap-2">
                        <select
                            class="text-xs bg-stone-100 dark:bg-stone-700 text-stone-700 dark:text-stone-300
                                rounded px-2 py-1 focus:outline-none"
                            prop:value=move || new_priority.get()
                            on:change=move |ev| new_priority.set(event_target_value(&ev))
                        >
                            <option value="high">"High"</option>
                            <option value="medium" selected>"Medium"</option>
                            <option value="low">"Low"</option>
                        </select>
                        <input
                            type="date"
                            class="text-xs bg-stone-100 dark:bg-stone-700 text-stone-700 dark:text-stone-300
                                rounded px-2 py-1 focus:outline-none"
                            title="Optional due date"
                            prop:value=move || new_due.get()
                            on:input=move |ev| new_due.set(event_target_value(&ev))
                        />
                        <span class="flex-1"/>
                        <button
                            class="text-xs bg-amber-500 hover:bg-amber-600 text-white
                                rounded px-3 py-1 transition-colors disabled:opacity-50"
                            on:click=on_add
                            disabled=move || adding.get()
                        >
                            {move || if adding.get() { "Adding…" } else { "Add" }}
                        </button>
                    </div>
                    {move || add_error.get().map(|msg| view! {
                        <p class="text-xs text-red-500">{msg}</p>
                    })}
                </div>
            })}

            // Task list (open tasks always shown; completed revealed on demand)
            {move || {
                let all_tasks = tasks_resource.get()
                    .and_then(|r| r.ok())
                    .unwrap_or_default();

                if all_tasks.is_empty() {
                    return view! {
                        <p class="text-sm text-stone-400 dark:text-stone-500 italic">
                            "No tasks yet."
                        </p>
                    }.into_any();
                }

                let (open_tasks, done_tasks): (Vec<_>, Vec<_>) =
                    all_tasks.into_iter().partition(|t| !status_done(&t.status));

                let done_count = done_tasks.len();
                let showing = show_completed.get();

                view! {
                    // Open tasks
                    {open_tasks.into_iter().map(|task| view! {
                        <TaskRow task=task task_refresh=task_refresh />
                    }).collect_view()}

                    // Completed tasks (revealed when toggled)
                    {showing.then(|| done_tasks.into_iter().map(|task| view! {
                        <TaskRow task=task task_refresh=task_refresh />
                    }).collect_view())}

                    // Disclosure row — shown only when completed tasks exist
                    {(done_count > 0).then(|| {
                        let label = if showing {
                            "Hide completed".to_string()
                        } else {
                            format!("{done_count} completed · show")
                        };
                        let icon = if showing { "expand_less" } else { "expand_more" };
                        view! {
                            <button
                                class="mt-2 flex items-center gap-1 text-xs
                                       text-stone-400 dark:text-stone-500
                                       hover:text-stone-600 dark:hover:text-stone-300
                                       transition-colors cursor-pointer"
                                on:click=move |_| show_completed.update(|v| *v = !*v)
                            >
                                <span class="material-symbols-outlined"
                                      style="font-size: 14px;">{icon}</span>
                                {label}
                            </button>
                        }
                    })}
                }.into_any()
            }}
        </div>
    }
}

// ── TaskRow ───────────────────────────────────────────────────────────────────

#[component]
fn TaskRow(task: Task, task_refresh: RwSignal<u32>) -> impl IntoView {
    let task_id = task.id;
    let is_done = status_done(&task.status);
    let status_val = status_value(&task.status).to_string();
    let priority = task.priority.clone();
    let priority_str = match priority {
        TaskPriority::High   => "high",
        TaskPriority::Medium => "medium",
        TaskPriority::Low    => "low",
    };
    let p_icon = priority_icon(&task.priority);
    let p_color = priority_color(&task.priority);
    let p_label = priority_label(&task.priority);
    let due = task.due_date;
    let focus = task.focus_date;

    let today = chrono::Utc::now().date_naive();
    let overdue = due.map(|d| !is_done && d < today).unwrap_or(false);
    let in_my_day = RwSignal::new(focus == Some(today));

    let status_sig = RwSignal::new(status_val.clone());

    // Inline editing — title, priority, due date.
    let editing_title  = RwSignal::new(false);
    let edit_title     = RwSignal::new(task.title.clone());
    let orig_title     = RwSignal::new(task.title.clone());
    let edit_priority  = RwSignal::new(priority_str.to_string());
    let edit_due       = RwSignal::new(
        due.map(|d| d.format("%Y-%m-%d").to_string())
            .unwrap_or_default(),
    );

    // Save all editable fields at once.
    let do_save_edit = move || {
        let new_title = edit_title.get_untracked().trim().to_string();
        if new_title.is_empty() { return; }
        editing_title.set(false);
        let new_priority = Some(parse_priority(&edit_priority.get_untracked()));
        let new_due: Option<Option<NaiveDate>> = Some(
            edit_due.get_untracked().trim().parse::<NaiveDate>().ok()
        );
        let req = UpdateTaskRequest {
            title: Some(new_title),
            status: None,
            priority: new_priority,
            focus_date: None,
            due_date: new_due,
        };
        wasm_bindgen_futures::spawn_local(async move {
            let _ = crate::api::update_task(task_id, &req).await;
            task_refresh.update(|n| *n += 1);
        });
    };

    // Toggle done ↔ open on checkbox click
    let on_toggle = move |_| {
        let current = status_sig.get_untracked();
        let next = if current == "done" { "open" } else { "done" };
        let next_status = parse_status(next);
        let req = UpdateTaskRequest {
            title: None,
            status: Some(next_status),
            priority: None,
            focus_date: None,
            due_date: None,
        };
        status_sig.set(next.to_string());
        wasm_bindgen_futures::spawn_local(async move {
            let _ = crate::api::update_task(task_id, &req).await;
            task_refresh.update(|n| *n += 1);
        });
    };

    // Toggle My Day
    let on_toggle_focus = move |_| {
        let currently = in_my_day.get_untracked();
        let new_focus = if currently { None } else { Some(today) };
        in_my_day.set(!currently); // optimistic UI update
        let req = UpdateTaskRequest {
            title: None,
            status: None,
            priority: None,
            focus_date: Some(new_focus),
            due_date: None,
        };
        wasm_bindgen_futures::spawn_local(async move {
            let _ = crate::api::update_task(task_id, &req).await;
            task_refresh.update(|n| *n += 1);
        });
    };

    // Delete
    let on_delete = move |_| {
        wasm_bindgen_futures::spawn_local(async move {
            let _ = crate::api::delete_task(task_id).await;
            task_refresh.update(|n| *n += 1);
        });
    };

    let title_display = task.title.clone();

    view! {
        <div class="group flex items-start gap-2 py-2 border-b border-stone-100 dark:border-stone-800 last:border-0">
            // Checkbox
            <button
                class="mt-0.5 flex-shrink-0 w-4 h-4 rounded border border-stone-400 dark:border-stone-500
                    flex items-center justify-center transition-colors
                    hover:border-amber-500"
                style=move || if status_done(&parse_status(&status_sig.get())) {
                    "background: #d97706; border-color: #d97706;"
                } else { "" }
                on:click=on_toggle
                title="Toggle done"
            >
                {move || status_done(&parse_status(&status_sig.get())).then(|| view! {
                    <span class="material-symbols-outlined text-white"
                        style="font-size: 12px;">{"check"}</span>
                })}
            </button>

            // Title + meta
            <div class="flex-1 min-w-0">
                // Title — display or inline edit form (title + priority + due date)
                {move || if editing_title.get() {
                    view! {
                        <div class="space-y-1.5 pb-1">
                            <input
                                type="text"
                                class="w-full bg-stone-100 dark:bg-stone-800 text-sm
                                    text-stone-900 dark:text-stone-100 rounded px-1.5 py-0.5
                                    focus:outline-none focus:ring-1 focus:ring-amber-500"
                                prop:value=move || edit_title.get()
                                on:input=move |ev| edit_title.set(event_target_value(&ev))
                                on:keydown=move |ev: leptos::ev::KeyboardEvent| {
                                    match ev.key().as_str() {
                                        "Enter" => do_save_edit(),
                                        "Escape" => {
                                            editing_title.set(false);
                                            edit_title.set(orig_title.get_untracked());
                                        }
                                        _ => {}
                                    }
                                }
                            />
                            <div class="flex items-center gap-2">
                                <select
                                    class="text-xs bg-stone-100 dark:bg-stone-700
                                        text-stone-700 dark:text-stone-300
                                        rounded px-2 py-0.5 focus:outline-none"
                                    prop:value=move || edit_priority.get()
                                    on:change=move |ev| edit_priority.set(event_target_value(&ev))
                                >
                                    <option value="high">"High"</option>
                                    <option value="medium">"Medium"</option>
                                    <option value="low">"Low"</option>
                                </select>
                                <input
                                    type="date"
                                    class="text-xs bg-stone-100 dark:bg-stone-700
                                        text-stone-700 dark:text-stone-300
                                        rounded px-2 py-0.5 focus:outline-none"
                                    title="Due date (optional)"
                                    prop:value=move || edit_due.get()
                                    on:input=move |ev| edit_due.set(event_target_value(&ev))
                                />
                                <span class="flex-1"/>
                                <button
                                    class="text-xs bg-amber-500 hover:bg-amber-600 text-white
                                        rounded px-2 py-0.5 transition-colors"
                                    on:click=move |_| do_save_edit()
                                >
                                    "Save"
                                </button>
                                <button
                                    class="text-xs text-stone-400 hover:text-stone-600
                                        dark:hover:text-stone-300 transition-colors"
                                    on:click=move |_| {
                                        editing_title.set(false);
                                        edit_title.set(orig_title.get_untracked());
                                    }
                                >
                                    "Cancel"
                                </button>
                            </div>
                        </div>
                    }.into_any()
                } else {
                    let td = title_display.clone();
                    view! {
                        <p
                            class="text-sm text-stone-800 dark:text-stone-200 leading-snug"
                            style=move || if status_done(&parse_status(&status_sig.get())) {
                                "text-decoration: line-through; opacity: 0.5;"
                            } else { "" }
                        >
                            {td}
                        </p>
                    }.into_any()
                }}
                <div class="flex items-center gap-2 mt-0.5">
                    // Priority badge
                    <span class="flex items-center gap-0.5 text-xs" style=p_color
                        title=p_label>
                        <span class="material-symbols-outlined" style="font-size:12px;">{p_icon}</span>
                        {p_label}
                    </span>
                    // Due date
                    {due.map(|d| {
                        let due_style = if overdue {
                            "color: #dc2626; font-weight: 600;"
                        } else {
                            "color: #6b7280;"
                        };
                        view! {
                            <span class="flex items-center gap-0.5 text-xs" style=due_style>
                                <span class="material-symbols-outlined" style="font-size:12px;">{"event"}</span>
                                {d.format("%b %-d").to_string()}
                                {overdue.then_some(" · overdue")}
                            </span>
                        }
                    })}
                    // Status badge (if not open/done)
                    {(!matches!(parse_status(&status_val), TaskStatus::Open | TaskStatus::Done)).then(|| {
                        let lbl = status_label(&parse_status(&status_val));
                        view! {
                            <span class="text-xs text-stone-400 dark:text-stone-500">{lbl}</span>
                        }
                    })}
                </div>
            </div>

            // Actions — always visible (group-hover:opacity is @media(hover:hover) only, invisible on touch)
            <div class="flex items-center gap-1 flex-shrink-0">
                // Edit task
                <button
                    class="p-1 rounded text-stone-400 hover:text-amber-500 transition-colors"
                    title="Edit task"
                    on:click=move |_| editing_title.set(true)
                >
                    <span class="material-symbols-outlined" style="font-size:16px;">{"edit"}</span>
                </button>
                // My Day toggle
                <button
                    class="p-1 rounded transition-colors"
                    style=move || if in_my_day.get() { "color: #d97706;" } else { "color: #a8a29e;" }
                    title=move || if in_my_day.get() { "Remove from My Day" } else { "Add to My Day" }
                    on:click=on_toggle_focus
                >
                    <span class="material-symbols-outlined" style="font-size:16px;">{"wb_sunny"}</span>
                </button>
                // Delete
                <button
                    class="p-1 rounded text-stone-400 hover:text-red-500 transition-colors"
                    title="Delete task"
                    on:click=on_delete
                >
                    <span class="material-symbols-outlined" style="font-size:16px;">{"delete"}</span>
                </button>
            </div>
        </div>
    }
}
