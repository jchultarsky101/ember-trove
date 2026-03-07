use leptos::prelude::*;

use crate::auth::AuthState;

#[component]
pub fn Sidebar(auth_state: AuthState) -> impl IntoView {
    let _auth = auth_state;

    view! {
        <nav class="flex-1 overflow-y-auto px-3 py-4 space-y-1">
            <SidebarLink icon="article"    label="Articles"   />
            <SidebarLink icon="work"       label="Projects"   />
            <SidebarLink icon="category"   label="Areas"      />
            <SidebarLink icon="inventory"  label="Resources"  />
            <SidebarLink icon="menu_book"  label="References" />
            <div class="border-t border-gray-200 dark:border-gray-700 my-3" />
            <SidebarLink icon="share"      label="Graph"      />
            <SidebarLink icon="search"     label="Search"     />
            <SidebarLink icon="sell"       label="Tags"       />
        </nav>
    }
}

#[component]
fn SidebarLink(icon: &'static str, label: &'static str) -> impl IntoView {
    view! {
        <button class="flex items-center gap-3 w-full px-3 py-2 text-sm font-medium rounded-lg
            text-gray-700 dark:text-gray-300 hover:bg-gray-100 dark:hover:bg-gray-800
            transition-colors">
            <span class="material-symbols-outlined text-gray-500 dark:text-gray-400">{icon}</span>
            {label}
        </button>
    }
}
