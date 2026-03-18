use common::id::NodeId;
use leptos::prelude::*;

use crate::app::View;
use crate::components::node_meta::{status_color, status_icon, status_label};

#[component]
pub fn ProjectDashboard() -> impl IntoView {
    let current_view = use_context::<RwSignal<View>>().expect("View signal must be provided");

    let entries = LocalResource::new(|| async move {
        crate::api::fetch_project_dashboard().await
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
                    <p class="text-sm text-stone-400">"Loading projects…"</p>
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
                            <div class="space-y-3">
                                // Column headers
                                <div class="grid grid-cols-12 gap-4 px-4 pb-2 border-b border-stone-200 dark:border-stone-700
                                    text-xs font-medium text-stone-500 dark:text-stone-400 uppercase tracking-wider">
                                    <div class="col-span-4">"Project"</div>
                                    <div class="col-span-2">"Status"</div>
                                    <div class="col-span-1 text-center">"Open"</div>
                                    <div class="col-span-1 text-center">"In Progress"</div>
                                    <div class="col-span-1 text-center">"Done"</div>
                                    <div class="col-span-1 text-center">"Cancelled"</div>
                                    <div class="col-span-2 text-center">"Progress"</div>
                                </div>
                                {data.into_iter().map(|entry| {
                                    let node_id = entry.node_id;
                                    let title = entry.title.clone();
                                    let ns = entry.node_status.clone();
                                    let counts = entry.task_counts;
                                    let total = counts.open + counts.in_progress + counts.done + counts.cancelled;
                                    let done_pct = if total > 0 {
                                        (counts.done * 100 / total) as f32
                                    } else { 0.0 };
                                    let s_icon = status_icon(&ns);
                                    let s_label = status_label(&ns);
                                    let s_color = status_color(&ns);
                                    view! {
                                        <ProjectRow
                                            node_id=node_id
                                            title=title
                                            node_status=ns
                                            status_icon=s_icon
                                            status_label=s_label
                                            status_color=s_color
                                            open=counts.open
                                            in_progress=counts.in_progress
                                            done=counts.done
                                            cancelled=counts.cancelled
                                            total=total
                                            done_pct=done_pct
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

#[allow(clippy::too_many_arguments)]
#[component]
fn ProjectRow(
    node_id: NodeId,
    title: String,
    #[allow(unused_variables)]
    node_status: String,
    status_icon: &'static str,
    status_label: &'static str,
    status_color: &'static str,
    open: u32,
    in_progress: u32,
    done: u32,
    cancelled: u32,
    total: u32,
    done_pct: f32,
    on_navigate: impl Fn() + 'static,
) -> impl IntoView {
    let _ = node_id;
    view! {
        <div
            class="grid grid-cols-12 gap-4 items-center px-4 py-3 rounded-lg
                bg-stone-50 dark:bg-stone-800/40
                hover:bg-amber-50/50 dark:hover:bg-stone-700/40
                border border-stone-100 dark:border-stone-800
                cursor-pointer transition-colors group"
            on:click=move |_| on_navigate()
        >
            // Project title
            <div class="col-span-4 flex items-center gap-2 min-w-0">
                <span class="material-symbols-outlined text-amber-500 flex-shrink-0"
                    style="font-size: 16px;">{"rocket_launch"}</span>
                <span class="font-medium text-stone-800 dark:text-stone-200 truncate
                    group-hover:text-amber-700 dark:group-hover:text-amber-400 transition-colors">
                    {title}
                </span>
            </div>

            // Node status
            <div class="col-span-2 flex items-center gap-1 text-sm" style=status_color>
                <span class="material-symbols-outlined" style="font-size: 15px;">{status_icon}</span>
                <span class="text-xs">{status_label}</span>
            </div>

            // Task counts
            <div class="col-span-1 text-center">
                <CountBadge count=open color="bg-stone-200 dark:bg-stone-700 text-stone-700 dark:text-stone-300" />
            </div>
            <div class="col-span-1 text-center">
                <CountBadge count=in_progress color="bg-amber-100 dark:bg-amber-900/30 text-amber-700 dark:text-amber-400" />
            </div>
            <div class="col-span-1 text-center">
                <CountBadge count=done color="bg-green-100 dark:bg-green-900/30 text-green-700 dark:text-green-400" />
            </div>
            <div class="col-span-1 text-center">
                <CountBadge count=cancelled color="bg-stone-100 dark:bg-stone-800 text-stone-400 dark:text-stone-500" />
            </div>

            // Progress bar
            <div class="col-span-2">
                {if total == 0 {
                    view! {
                        <span class="text-xs text-stone-400 dark:text-stone-500 italic">"No tasks"</span>
                    }.into_any()
                } else {
                    view! {
                        <div class="flex items-center gap-2">
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
    }
}

#[component]
fn CountBadge(count: u32, color: &'static str) -> impl IntoView {
    view! {
        <span class=format!("inline-flex items-center justify-center w-7 h-5 rounded text-xs font-semibold {color}")>
            {count.to_string()}
        </span>
    }
}
