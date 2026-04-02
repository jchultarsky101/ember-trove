use leptos::prelude::*;

use crate::app::View;
use crate::markdown::render_markdown_plain;

/// Mirror of note_panel::PALETTE — full class strings so Tailwind's scanner picks them up.
const PALETTE: &[(&str, &str)] = &[
    ("default", "bg-stone-50 dark:bg-stone-900/50 border-stone-200 dark:border-stone-700"),
    ("amber",   "bg-amber-100 dark:bg-amber-950/60 border-amber-300 dark:border-amber-800"),
    ("rose",    "bg-rose-100 dark:bg-rose-950/60 border-rose-300 dark:border-rose-800"),
    ("lime",    "bg-lime-100 dark:bg-lime-950/60 border-lime-300 dark:border-lime-800"),
    ("sky",     "bg-sky-100 dark:bg-sky-950/60 border-sky-300 dark:border-sky-800"),
    ("violet",  "bg-violet-100 dark:bg-violet-950/60 border-violet-300 dark:border-violet-800"),
];

fn palette_card_class(color: &str) -> &'static str {
    PALETTE.iter()
        .find(|(k, _)| *k == color)
        .map(|(_, cls)| *cls)
        .unwrap_or(PALETTE[0].1)
}

#[component]
pub fn NotesView() -> impl IntoView {
    let current_view = use_context::<RwSignal<View>>().expect("View signal must be provided");

    let feed_resource = LocalResource::new(move || async move {
        crate::api::fetch_notes_feed().await
    });

    view! {
        <div class="flex flex-col h-full">
            // Header
            <div class="flex items-center gap-3 px-6 py-4 border-b border-stone-200 dark:border-stone-800">
                <span class="material-symbols-outlined text-amber-500" style="font-size: 22px;">
                    {"sticky_note_2"}
                </span>
                <h1 class="text-lg font-semibold text-stone-900 dark:text-stone-100">
                    "Notes"
                </h1>
            </div>

            // Feed
            <div class="flex-1 overflow-auto p-6 flex flex-col">
                <Suspense fallback=move || view! {
                    <p class="text-sm text-stone-400">"Loading…"</p>
                }>
                    {move || {
                        let notes = feed_resource.get()
                            .and_then(|r| r.ok())
                            .unwrap_or_default();

                        if notes.is_empty() {
                            return view! {
                                <div class="flex-1 flex flex-col items-center justify-center gap-3">
                                    <span class="material-symbols-outlined text-stone-300 dark:text-stone-700"
                                        style="font-size: 48px;">{"sticky_note_2"}</span>
                                    <p class="text-stone-400 dark:text-stone-500 text-sm text-center">
                                        "No notes yet."
                                    </p>
                                    <p class="text-stone-400 dark:text-stone-500 text-sm text-center">
                                        "Open a node and add a note to see it here."
                                    </p>
                                </div>
                            }.into_any();
                        }

                        view! {
                            <div class="space-y-4 w-full">
                                {notes.into_iter().map(|feed_note| {
                                    let node_id = feed_note.note.node_id;
                                    let node_title = feed_note.node_title.clone();
                                    let body_html = render_markdown_plain(&feed_note.note.body);
                                    let card_class = palette_card_class(&feed_note.note.color).to_string();
                                    let ts = feed_note.note.created_at
                                        .format("%b %-d, %Y %H:%M")
                                        .to_string();

                                    view! {
                                        <div class=format!("rounded-lg border px-4 py-3 {card_class}")>
                                            // Node link header
                                            <button
                                                class="flex items-center gap-1.5 mb-2 text-xs font-semibold
                                                    text-stone-400 dark:text-stone-500 uppercase tracking-wider
                                                    hover:text-amber-600 dark:hover:text-amber-400 transition-colors"
                                                on:click=move |_| current_view.set(View::NodeDetail(node_id))
                                            >
                                                <span class="material-symbols-outlined" style="font-size: 13px;">
                                                    {"description"}
                                                </span>
                                                {node_title}
                                                <span class="material-symbols-outlined" style="font-size: 12px;">
                                                    {"open_in_new"}
                                                </span>
                                            </button>
                                            // Body
                                            <div
                                                class="prose prose-sm max-w-none dark:prose-invert
                                                    prose-p:my-0.5 prose-ul:my-0.5 prose-ol:my-0.5
                                                    prose-li:my-0 prose-headings:mt-1 prose-headings:mb-0.5"
                                                inner_html=body_html
                                            />
                                            // Timestamp
                                            <p class="text-xs text-stone-400 dark:text-stone-600 mt-2">
                                                {ts}
                                            </p>
                                        </div>
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
