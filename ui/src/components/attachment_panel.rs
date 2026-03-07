/// Upload / list / download attachments for a node.
///
/// Phase 1 stub — Phase 6 wires the S3 + attachments API.
use leptos::prelude::*;

#[component]
pub fn AttachmentPanel() -> impl IntoView {
    view! {
        <div class="px-4 py-3">
            <p class="text-xs text-gray-400 dark:text-gray-600">"No attachments"</p>
        </div>
    }
}
