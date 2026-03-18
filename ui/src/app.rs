use common::{id::NodeId, tag::Tag};
use leptos::prelude::*;

use crate::{
    auth::provide_auth_state,
    components::{dark_mode_toggle::Theme, layout::Layout, toast::ToastState},
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
}

// ── App root ───────────────────────────────────────────────────────────────

#[component]
pub fn App() -> impl IntoView {
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

    // Shared search query — written by SearchBar, read by SearchView.
    let search_query: RwSignal<String> = RwSignal::new(String::new());
    provide_context(search_query);

    // Toast notification state.
    let toast_state = ToastState::new();
    provide_context(toast_state);

    view! {
        <Layout auth_state=auth_state />
    }
}
