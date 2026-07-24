//! Pattern: page header + empty state primitives (design phase 2, 2026-07-24)
//!
//! Every top-level list view renders its header through `PageHeader` and its
//! "nothing here" state through `EmptyState` — never hand-rolled markup. The
//! primitives own the type scale (title `text-lg font-semibold`, subtitle
//! `text-xs`), the 22 px amber icon, and the `px-4 md:px-6 py-4` bar, so a
//! new view can't reintroduce the pre-phase-2 drift (Inbox once shipped
//! `text-xl`/26px while every sibling was `text-lg`/22px).
//!
//! Real code: ui/src/components/page_header.rs · empty_state.rs
//! Converted views: my_day, inbox, calendar, node_list, notes, dashboard,
//! tag_manager. NOT node_view (bespoke back-button/editable-title header) or
//! search_view (its bar is a wrapping filter toolbar).
//!
//! Radius convention (stated, phase 2): cards `rounded-lg` · popovers &
//! dropdowns `rounded-xl shadow-xl` · modals `rounded-2xl shadow-2xl` with
//! border. Row dividers inside cards: `divide-stone-100 dark:divide-stone-800`.

// ── Static title + right-aligned action cluster ──────────────────────────────
view! {
    <PageHeader icon="inbox" title="Inbox" subtitle="Capture tasks…".into_any()>
        <button /* right-aligned cluster */>"Process"</button>
    </PageHeader>
}

// ── Reactive title (node_list) ───────────────────────────────────────────────
view! {
    <PageHeader title=Signal::derive(move || match node_type_filter.get().as_deref() {
        Some("project") => "Projects",
        _ => "All Nodes",
    })>
        // …
    </PageHeader>
}

// ── Empty state (page-level; zone-level empties stay compact italic) ─────────
view! {
    <EmptyState icon="search_off" message="No results found."
        hint="Try different keywords, tags, or enable fuzzy search." />
}
