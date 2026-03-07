/// Split-pane Markdown editor (textarea + live preview).
///
/// Phase 1 stub — Phase 3 wires save / cancel / API calls.
use leptos::prelude::*;
use pulldown_cmark::{html, Options, Parser};

#[allow(dead_code)]
fn render_markdown(source: &str) -> String {
    let opts = Options::ENABLE_STRIKETHROUGH
        | Options::ENABLE_TABLES
        | Options::ENABLE_TASKLISTS;
    let parser = Parser::new_ext(source, opts);
    let mut html_out = String::new();
    html::push_html(&mut html_out, parser);
    ammonia::clean(&html_out)
}

#[component]
pub fn NodeEditor() -> impl IntoView {
    let content = RwSignal::new(String::new());

    let preview_html = move || render_markdown(&content.get());

    view! {
        <div class="flex h-full divide-x divide-gray-200 dark:divide-gray-700">
            // Edit pane
            <div class="flex-1 flex flex-col">
                <textarea
                    class="flex-1 p-4 font-mono text-sm resize-none bg-transparent
                        text-gray-900 dark:text-gray-100 focus:outline-none"
                    placeholder="Write in Markdown…"
                    prop:value=move || content.get()
                    on:input=move |ev| content.set(event_target_value(&ev))
                />
            </div>
            // Preview pane
            <div class="flex-1 overflow-auto p-6">
                <div
                    class="prose max-w-none"
                    inner_html=preview_html
                />
            </div>
        </div>
    }
}
