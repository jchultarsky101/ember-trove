use common::{id::{NodeId, TemplateId}, tag::Tag};
use leptos::{ev, prelude::*};

use crate::{
    auth::provide_auth_state,
    components::{
        dark_mode_toggle::Theme, layout::Layout,
        modals::shortcuts::ShortcutsModal,
        public_share_view::PublicShareView,
        toast::{push_toast, ToastLevel, ToastState},
    },
};

// ── localStorage helpers ───────────────────────────────────────────────────

fn local_storage() -> Option<web_sys::Storage> {
    web_sys::window()
        .and_then(|w| w.local_storage().ok())
        .flatten()
}

pub fn storage_get(key: &str) -> Option<String> {
    local_storage().and_then(|s| s.get_item(key).ok()).flatten()
}

pub fn storage_set(key: &str, value: &str) {
    if let Some(s) = local_storage() {
        let _ = s.set_item(key, value);
    }
}

// ── Global task refresh signal ─────────────────────────────────────────────
// Shared between TaskPanel and MyDayView so toggling a task in either view
// causes the other to re-fetch. Wrapped in a newtype to avoid collision with
// the nodes-list `refresh: RwSignal<u32>` already in context.

#[derive(Clone, Copy)]
pub struct TaskRefresh(pub RwSignal<u32>);

// ── Quick-capture modal visibility ─────────────────────────────────────────
// Lifted to App root so the global `n` keyboard shortcut and the FAB both
// drive the same signal. Newtype avoids collision with other RwSignal<bool>.

#[derive(Clone, Copy)]
pub struct ShowCapture(pub RwSignal<bool>);

// ── Current view ───────────────────────────────────────────────────────────

#[allow(dead_code)]
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum View {
    NodeList,
    NodeDetail(NodeId),
    NodeCreate,
    NodeEdit(NodeId),
    TagManager,
    Graph,
    Search,
    Admin,
    ProjectDashboard,
    MyDay,
    Notes,
    Backup,
    Templates,
    BulkPermissions,
}

/// Pre-fill data passed from TemplatesView to NodeEditor when "Use" is clicked.
#[derive(Clone, Debug, PartialEq)]
pub struct TemplatePrefill {
    pub node_type: String,
    pub body: String,
    pub template_id: TemplateId,
}

// ── App root ───────────────────────────────────────────────────────────────

#[component]
pub fn App() -> impl IntoView {
    // Public share links (`/share/<uuid>`) render a standalone read-only view
    // without authentication or the main layout.
    let pathname = web_sys::window()
        .and_then(|w| w.location().pathname().ok())
        .unwrap_or_default();
    if let Some(token_str) = pathname.strip_prefix("/share/")
        && let Ok(token) = token_str.parse::<uuid::Uuid>()
    {
        return view! { <PublicShareView token=token /> }.into_any();
    }

    let auth_state = provide_auth_state();

    // Persist theme in localStorage
    let initial_theme = storage_get("theme")
        .map(|s| {
            if s == "dark" {
                Theme::Dark
            } else {
                Theme::Light
            }
        })
        .unwrap_or(Theme::Light);

    let theme = RwSignal::new(initial_theme);
    provide_context(theme);

    // Apply / remove the `dark` class on <html> whenever theme changes
    Effect::new(move |_| {
        let is_dark = theme.get() == Theme::Dark;
        if let Some(doc) = web_sys::window().and_then(|w| w.document())
            && let Some(html) = doc.document_element()
        {
            if is_dark {
                let _ = html.class_list().add_1("dark");
                storage_set("theme", "dark");
            } else {
                let _ = html.class_list().remove_1("dark");
                storage_set("theme", "light");
            }
        }
    });

    let current_view = RwSignal::new(View::NodeList);
    provide_context(current_view);

    // Refresh trigger — bump to re-fetch nodes list.
    let refresh = RwSignal::new(0u32);
    provide_context(refresh);

    // Tag filter — set to Some(tag) to filter NodeList/SearchView by that tag.
    let tag_filter: RwSignal<Option<Tag>> = RwSignal::new(None);
    provide_context(tag_filter);

    // Node-type filter — set by sidebar type links, read by NodeList.
    let node_type_filter: RwSignal<Option<String>> = RwSignal::new(None);
    provide_context(node_type_filter);

    // Shared search query — written by SearchBar, read by SearchView.
    let search_query: RwSignal<String> = RwSignal::new(String::new());
    provide_context(search_query);

    // Global task refresh — shared by TaskPanel and MyDayView.
    let task_refresh = TaskRefresh(RwSignal::new(0u32));
    provide_context(task_refresh);

    // Quick-capture modal — shared between `n` shortcut and the FAB button.
    let show_capture = ShowCapture(RwSignal::new(false));
    provide_context(show_capture);

    // Toast notification state.
    let toast_state = ToastState::new();
    provide_context(toast_state);

    // Template prefill — set by TemplatesView "Use", consumed by NodeEditor on create.
    let template_prefill: RwSignal<Option<TemplatePrefill>> = RwSignal::new(None);
    provide_context(template_prefill);

    // Current node pin state — written by NodeView when a node loads so the
    // global `p` shortcut can toggle it without an extra API round-trip.
    let current_node_pinned: RwSignal<bool> = RwSignal::new(false);
    provide_context(current_node_pinned);

    // Keyboard shortcuts help modal visibility.
    let show_shortcuts = RwSignal::new(false);

    // ── Global keyboard shortcuts ───────────────────────────────────────────
    // Suppressed when the user is typing in an input, textarea, select, or
    // any contenteditable element.
    //
    // Shortcuts:
    //   n   → Quick-capture modal (same as FAB)
    //   g   → Graph view
    //   /   → Search (also prevents the browser's built-in page-find)
    //   Esc → Back to node list (from detail / edit / create); close modal
    //   ?   → Toggle keyboard shortcuts help
    let handle = window_event_listener(ev::keydown, move |ev: web_sys::KeyboardEvent| {
        // Ignore if a modifier key is held (Ctrl+n, Cmd+/, etc.).
        if ev.ctrl_key() || ev.meta_key() || ev.alt_key() {
            return;
        }

        // Ignore when focus is inside an editable element.
        let is_editable = web_sys::window()
            .and_then(|w| w.document())
            .and_then(|d| d.active_element())
            .map(|el| {
                let tag = el.tag_name().to_uppercase();
                if matches!(tag.as_str(), "INPUT" | "TEXTAREA" | "SELECT" | "BUTTON") {
                    return true;
                }
                // contenteditable="true" or contenteditable="" (empty = true per spec)
                el.get_attribute("contenteditable")
                    .map(|v| v != "false")
                    .unwrap_or(false)
            })
            .unwrap_or(false);

        if is_editable {
            return;
        }

        match ev.key().as_str() {
            "?" => show_shortcuts.update(|v| *v = !*v),
            "n" => show_capture.0.set(true),
            "g" => current_view.set(View::Graph),
            "/" => {
                ev.prevent_default();
                current_view.set(View::Search);
            }
            "p" => {
                // Toggle pin on the currently open node.
                if let View::NodeDetail(node_id) = current_view.get_untracked() {
                    let new_pinned = !current_node_pinned.get_untracked();
                    current_node_pinned.set(new_pinned);
                    wasm_bindgen_futures::spawn_local(async move {
                        match crate::api::set_node_pinned(node_id, new_pinned).await {
                            Ok(_) => {
                                refresh.update(|n| *n += 1);
                                let msg = if new_pinned { "Node pinned." } else { "Node unpinned." };
                                push_toast(ToastLevel::Success, msg);
                            }
                            Err(e) => {
                                current_node_pinned.set(!new_pinned); // revert
                                push_toast(ToastLevel::Error, format!("Pin failed: {e}"));
                            }
                        }
                    });
                }
            }
            "Escape" => {
                if show_shortcuts.get_untracked() {
                    show_shortcuts.set(false);
                } else if matches!(
                    current_view.get_untracked(),
                    View::NodeDetail(_) | View::NodeEdit(_) | View::NodeCreate
                ) {
                    current_view.set(View::NodeList);
                }
            }
            _ => {}
        }
    });
    on_cleanup(move || handle.remove());

    view! {
        <Layout auth_state=auth_state />
        <ShortcutsModal
            show=show_shortcuts.read_only()
            on_close=Callback::new(move |_| show_shortcuts.set(false))
        />
    }
    .into_any()
}
