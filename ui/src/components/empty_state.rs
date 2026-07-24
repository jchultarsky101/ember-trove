use leptos::prelude::*;

/// Canonical empty-state block (design phase 2).
///
/// One shape for "there is nothing here yet": 48 px muted icon over a
/// `text-sm` line, with an optional hint underneath. Views previously
/// improvised (search used a smaller `text-4xl` icon, paddings varied
/// p-12/py-16). My Day's *zone* empties intentionally stay compact
/// italic one-liners — a zone is a region inside a page, not a page.
#[component]
pub fn EmptyState(
    /// Material Symbols icon name (rendered 48 px, muted).
    icon: &'static str,
    /// Primary line, e.g. "No nodes yet".
    message: &'static str,
    /// Optional secondary hint, e.g. how to create the first item.
    #[prop(optional)]
    hint: Option<&'static str>,
) -> impl IntoView {
    view! {
        <div class="flex flex-col items-center justify-center py-12 text-center">
            <span
                class="material-symbols-outlined text-stone-300 dark:text-stone-700"
                style="font-size:48px;"
            >
                {icon}
            </span>
            <p class="mt-3 text-sm text-stone-400 dark:text-stone-500">{message}</p>
            {hint
                .map(|h| {
                    view! { <p class="mt-1 text-xs text-stone-400 dark:text-stone-600">{h}</p> }
                })}
        </div>
    }
}
