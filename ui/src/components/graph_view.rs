/// Visual knowledge graph — force-directed SVG node-link diagram.
///
/// Fetches all nodes and edges, runs a Fruchterman-Reingold layout in WASM,
/// then overlays saved positions from the API.  Nodes are draggable; positions
/// are persisted to the DB on mouse-up.  The canvas supports pan (drag on
/// background) and zoom (wheel).
///
/// Node shapes convey type: circle=Article, diamond=Project, rounded-rect=Area,
/// hexagon=Resource, triangle=Reference.  Each type has a distinct fill colour.
///
/// Edge lines convey type via colour + dash pattern; text labels are omitted.
/// Hovering an edge shows a pop-up card with the relationship type, direction,
/// and optional custom label.
///
/// SVG presentation attributes are set via `style` because Leptos 0.8 writes
/// `attr:` prefixes literally for unknown SVG elements.
use std::collections::HashMap;

use leptos::prelude::*;
use uuid::Uuid;
use wasm_bindgen_futures::spawn_local;
use web_sys::{MouseEvent, WheelEvent};

use common::{
    edge::{Edge, EdgeType},
    id::NodeId,
    node::{Node, NodeType},
};

use crate::{
    api::{fetch_all_edges, fetch_nodes, fetch_positions, save_position},
    app::View,
    components::node_meta::{status_color_hex, status_label, type_label},
};

const W: f64 = 1000.0;
const H: f64 = 700.0;
const MARGIN: f64 = 80.0;
/// Effective radius used for edge start/end offset (conservative for all shapes).
const NODE_R: f64 = 22.0;
const ARROW_OFFSET: f64 = NODE_R + 4.0;
/// Y offset of the title text below the node centre.
const TEXT_Y_OFFSET: f64 = 33.0;

// ── Hover card data ──────────────────────────────────────────────────────────

/// Data for the node summary card shown on hover.
#[derive(Clone)]
struct NodeHoverInfo {
    title: String,
    node_type: String,
    status: String,
    body_preview: Option<String>,
    node_id: Uuid,
}

#[derive(Clone)]
struct EdgeHover {
    type_label: &'static str,
    custom_label: Option<String>,
    src_title: String,
    tgt_title: String,
    src_id: Uuid,
    tgt_id: Uuid,
}

// ── Node-type helpers ────────────────────────────────────────────────────────

fn node_fill(nt: &NodeType) -> &'static str {
    match nt {
        NodeType::Article   => "#d97706",
        NodeType::Project   => "#2563eb",
        NodeType::Area      => "#16a34a",
        NodeType::Resource  => "#9333ea",
        NodeType::Reference => "#dc2626",
    }
}

fn node_stroke_color(nt: &NodeType) -> &'static str {
    match nt {
        NodeType::Article   => "#92400e",
        NodeType::Project   => "#1e40af",
        NodeType::Area      => "#166534",
        NodeType::Resource  => "#6b21a8",
        NodeType::Reference => "#991b1b",
    }
}

/// Diamond (rotated square) points for Project nodes.
fn diamond_points(cx: f64, cy: f64) -> String {
    let r = 22.0_f64;
    format!(
        "{:.1},{:.1} {:.1},{:.1} {:.1},{:.1} {:.1},{:.1}",
        cx,
        cy - r,
        cx + r,
        cy,
        cx,
        cy + r,
        cx - r,
        cy,
    )
}

/// Regular-hexagon points for Resource nodes.
fn hexagon_points(cx: f64, cy: f64) -> String {
    let r = 20.0_f64;
    (0..6)
        .map(|i| {
            let a = std::f64::consts::PI / 3.0 * i as f64 - std::f64::consts::PI / 6.0;
            format!("{:.1},{:.1}", cx + r * a.cos(), cy + r * a.sin())
        })
        .collect::<Vec<_>>()
        .join(" ")
}

/// Upward-pointing equilateral triangle for Reference nodes.
fn triangle_points(cx: f64, cy: f64) -> String {
    let r = 22.0_f64;
    let hx = r * 0.866_f64; // sqrt(3)/2
    format!(
        "{:.1},{:.1} {:.1},{:.1} {:.1},{:.1}",
        cx,
        cy - r,
        cx + hx,
        cy + r * 0.5,
        cx - hx,
        cy + r * 0.5,
    )
}

// ── Edge-type helpers ────────────────────────────────────────────────────────

fn edge_color(et: &EdgeType) -> &'static str {
    match et {
        EdgeType::References  => "#d97706",
        EdgeType::Contains    => "#22c55e",
        EdgeType::RelatedTo   => "#a855f7",
        EdgeType::DependsOn   => "#f97316",
        EdgeType::DerivedFrom => "#ec4899",
        EdgeType::WikiLink    => "#60a5fa",
    }
}

/// CSS stroke-dasharray; "none" → solid line.
fn edge_dash(et: &EdgeType) -> &'static str {
    match et {
        EdgeType::References  => "none",
        EdgeType::Contains    => "none",
        EdgeType::RelatedTo   => "8,4",
        EdgeType::DependsOn   => "3,4",
        EdgeType::DerivedFrom => "8,3,2,3",
        EdgeType::WikiLink    => "4,2",
    }
}

fn edge_stroke_width(et: &EdgeType) -> f64 {
    match et {
        EdgeType::Contains => 2.5,
        EdgeType::WikiLink => 1.0,
        _                  => 1.5,
    }
}

fn edge_label(et: &EdgeType) -> &'static str {
    match et {
        EdgeType::References  => "References",
        EdgeType::Contains    => "Contains",
        EdgeType::RelatedTo   => "Related to",
        EdgeType::DependsOn   => "Depends on",
        EdgeType::DerivedFrom => "Derived from",
        EdgeType::WikiLink    => "Wiki link",
    }
}

fn edge_marker_id(et: &EdgeType) -> &'static str {
    match et {
        EdgeType::References  => "arrow-references",
        EdgeType::Contains    => "arrow-contains",
        EdgeType::RelatedTo   => "arrow-related-to",
        EdgeType::DependsOn   => "arrow-depends-on",
        EdgeType::DerivedFrom => "arrow-derived-from",
        EdgeType::WikiLink    => "arrow-wiki-link",
    }
}

// ── SVG marker injection ─────────────────────────────────────────────────────

fn inject_svg_markers() {
    let Some(win) = web_sys::window() else { return };
    let Some(doc) = win.document() else { return };
    let Some(svg) = doc.get_element_by_id("graph-svg") else { return };
    if svg.query_selector("marker").ok().flatten().is_some() {
        return;
    }

    let ns = "http://www.w3.org/2000/svg";
    let Ok(defs) = doc.create_element_ns(Some(ns), "defs") else { return };

    const ARROWS: &[(&str, &str)] = &[
        ("arrow-references",   "#d97706"),
        ("arrow-contains",     "#22c55e"),
        ("arrow-related-to",   "#a855f7"),
        ("arrow-depends-on",   "#f97316"),
        ("arrow-derived-from", "#ec4899"),
        ("arrow-wiki-link",    "#60a5fa"),
    ];

    for (id, color) in ARROWS {
        let Ok(marker) = doc.create_element_ns(Some(ns), "marker") else { continue };
        let _ = marker.set_attribute("id", id);
        let _ = marker.set_attribute("markerWidth", "8");
        let _ = marker.set_attribute("markerHeight", "6");
        let _ = marker.set_attribute("refX", "6");
        let _ = marker.set_attribute("refY", "3");
        let _ = marker.set_attribute("orient", "auto");

        let Ok(path) = doc.create_element_ns(Some(ns), "path") else { continue };
        let _ = path.set_attribute("d", "M 0 0 L 6 3 L 0 6 Z");
        let _ = path.set_attribute("fill", color);
        let _ = marker.append_child(&path);
        let _ = defs.append_child(&marker);
    }

    if let Some(first) = svg.first_child() {
        let _ = svg.insert_before(&defs, Some(&first));
    } else {
        let _ = svg.append_child(&defs);
    }
}

// ── Misc helpers ─────────────────────────────────────────────────────────────

/// Truncate to at most `n` characters, appending "…" if clipped.
fn truncate(s: &str, n: usize) -> String {
    let mut chars = s.chars();
    let taken: String = chars.by_ref().take(n).collect();
    if chars.next().is_some() {
        format!("{taken}\u{2026}")
    } else {
        taken
    }
}

/// Strip Markdown syntax and return up to 100 chars as a plain-text preview.
fn node_body_preview(body: &str) -> Option<String> {
    let text: String = body
        .lines()
        .map(str::trim)
        .filter(|l| {
            !l.is_empty()
                && !l.starts_with('#')
                && !l.starts_with("```")
                && !l.starts_with("---")
        })
        .collect::<Vec<_>>()
        .join(" ");
    if text.is_empty() {
        return None;
    }
    let clean = text.replace("**", "").replace("__", "").replace('`', "");
    Some(truncate(&clean, 100))
}

/// Compute the SVG path `d` for an edge: starts at the source shape boundary,
/// ends just before the target so the arrowhead is visible.
fn compute_path(x1: f64, y1: f64, x2: f64, y2: f64) -> String {
    let dx = x2 - x1;
    let dy = y2 - y1;
    let len = (dx * dx + dy * dy).sqrt();
    if len < NODE_R + ARROW_OFFSET + 2.0 {
        return String::new();
    }
    let ux = dx / len;
    let uy = dy / len;
    format!(
        "M {:.1} {:.1} L {:.1} {:.1}",
        x1 + ux * NODE_R,
        y1 + uy * NODE_R,
        x2 - ux * ARROW_OFFSET,
        y2 - uy * ARROW_OFFSET,
    )
}

// ── Layout ───────────────────────────────────────────────────────────────────

fn force_layout(node_ids: &[Uuid], edge_pairs: &[(Uuid, Uuid)]) -> HashMap<Uuid, (f64, f64)> {
    let n = node_ids.len();
    if n == 0 {
        return HashMap::new();
    }
    if n == 1 {
        let mut m = HashMap::new();
        m.insert(node_ids[0], (W / 2.0, H / 2.0));
        return m;
    }

    let uw = W - 2.0 * MARGIN;
    let uh = H - 2.0 * MARGIN;

    let mut px: Vec<f64> = (0..n)
        .map(|_| MARGIN + js_sys::Math::random() * uw)
        .collect();
    let mut py: Vec<f64> = (0..n)
        .map(|_| MARGIN + js_sys::Math::random() * uh)
        .collect();

    let k = (uw * uh / n as f64).sqrt();

    for iter in 0..200_u32 {
        let mut disp_x = vec![0.0_f64; n];
        let mut disp_y = vec![0.0_f64; n];

        for i in 0..n {
            for j in (i + 1)..n {
                let ddx = px[i] - px[j];
                let ddy = py[i] - py[j];
                let dist = (ddx * ddx + ddy * ddy).sqrt().max(1.0);
                let force = k * k / dist;
                let fx = ddx / dist * force;
                let fy = ddy / dist * force;
                disp_x[i] += fx;
                disp_y[i] += fy;
                disp_x[j] -= fx;
                disp_y[j] -= fy;
            }
        }

        for (src, tgt) in edge_pairs {
            let si = node_ids.iter().position(|id| id == src);
            let ti = node_ids.iter().position(|id| id == tgt);
            if let (Some(si), Some(ti)) = (si, ti) {
                let ddx = px[si] - px[ti];
                let ddy = py[si] - py[ti];
                let dist = (ddx * ddx + ddy * ddy).sqrt().max(1.0);
                let force = dist * dist / k;
                let fx = ddx / dist * force;
                let fy = ddy / dist * force;
                disp_x[si] -= fx;
                disp_y[si] -= fy;
                disp_x[ti] += fx;
                disp_y[ti] += fy;
            }
        }

        let temp = 200.0_f64 * (1.0 - iter as f64 / 200.0).max(0.01);
        for i in 0..n {
            let mag = (disp_x[i] * disp_x[i] + disp_y[i] * disp_y[i])
                .sqrt()
                .max(0.001);
            let step = mag.min(temp);
            px[i] = (px[i] + disp_x[i] / mag * step).clamp(MARGIN, W - MARGIN);
            py[i] = (py[i] + disp_y[i] / mag * step).clamp(MARGIN, H - MARGIN);
        }
    }

    node_ids
        .iter()
        .enumerate()
        .map(|(i, id)| (*id, (px[i], py[i])))
        .collect()
}

// ── Component ────────────────────────────────────────────────────────────────

#[component]
pub fn GraphView() -> impl IntoView {
    let current_view = use_context::<RwSignal<View>>().expect("View signal must be provided");
    let loading = RwSignal::new(true);
    let error_msg = RwSignal::new(Option::<String>::None);
    let nodes_sig: RwSignal<Vec<Node>> = RwSignal::new(vec![]);
    let edges_sig: RwSignal<Vec<Edge>> = RwSignal::new(vec![]);
    let positions: RwSignal<HashMap<Uuid, (f64, f64)>> = RwSignal::new(HashMap::new());

    let pan_x = RwSignal::new(0.0_f64);
    let pan_y = RwSignal::new(0.0_f64);
    let zoom = RwSignal::new(1.0_f64);
    let panning = RwSignal::new(false);
    let last_mx = RwSignal::new(0.0_f64);
    let last_my = RwSignal::new(0.0_f64);

    let drag_node: RwSignal<Option<Uuid>> = RwSignal::new(None);
    let drag_offset: RwSignal<(f64, f64)> = RwSignal::new((0.0, 0.0));
    let did_drag = RwSignal::new(false);

    let edge_hover: RwSignal<Option<EdgeHover>> = RwSignal::new(None);
    let node_hover: RwSignal<Option<NodeHoverInfo>> = RwSignal::new(None);

    // ── Type-visibility filter signals ───────────────────────────────────────
    // Each signal controls whether nodes of that type are rendered.
    let show_articles:   RwSignal<bool> = RwSignal::new(true);
    let show_projects:   RwSignal<bool> = RwSignal::new(true);
    let show_areas:      RwSignal<bool> = RwSignal::new(true);
    let show_resources:  RwSignal<bool> = RwSignal::new(true);
    let show_references: RwSignal<bool> = RwSignal::new(true);

    Effect::new(move |_| {
        spawn_local(async move {
            match fetch_nodes().await {
                Err(e) => {
                    error_msg.set(Some(format!("{e}")));
                    loading.set(false);
                }
                Ok(nodes) => {
                    let edges = fetch_all_edges().await.unwrap_or_default();
                    let node_ids: Vec<Uuid> = nodes.iter().map(|n| n.id.0).collect();
                    let edge_pairs: Vec<(Uuid, Uuid)> =
                        edges.iter().map(|e| (e.source_id.0, e.target_id.0)).collect();

                    let mut layout = force_layout(&node_ids, &edge_pairs);

                    if let Ok(saved) = fetch_positions().await {
                        for pos in saved {
                            layout.insert(pos.node_id.0, (pos.x, pos.y));
                        }
                    }

                    positions.set(layout);
                    nodes_sig.set(nodes);
                    edges_sig.set(edges);
                    loading.set(false);

                    spawn_local(async {
                        gloo_timers::future::TimeoutFuture::new(50).await;
                        inject_svg_markers();
                    });
                }
            }
        });
    });

    view! {
        <div class="relative w-full h-full overflow-hidden bg-stone-50 dark:bg-stone-950 select-none">
            // ── Legend overlay ───────────────────────────────────────────────
            // ── Toolbar (top-right) ──────────────────────────────────────────
            <div class="absolute top-3 right-3 z-10 flex items-center gap-2">
                <button
                    class="px-2.5 py-1 rounded-lg text-xs font-medium
                           bg-white/80 dark:bg-stone-900/85 backdrop-blur-sm
                           border border-stone-200 dark:border-stone-700
                           text-stone-600 dark:text-stone-300
                           hover:bg-stone-50 dark:hover:bg-stone-800 cursor-pointer transition-colors"
                    title="Fit all nodes into view"
                    on:click=move |_| {
                        pan_x.set(0.0);
                        pan_y.set(0.0);
                        zoom.set(1.0);
                    }
                >
                    "Fit"
                </button>
            </div>

            // ── Legend + type filter overlay (bottom-left) ───────────────────
            <div class="absolute bottom-4 left-4 z-10
                        bg-white/80 dark:bg-stone-900/85 backdrop-blur-sm
                        border border-stone-200 dark:border-stone-700
                        rounded-xl p-3 flex flex-col gap-2 text-xs">
                // Node type toggles — click to show/hide
                <p class="font-semibold text-stone-500 dark:text-stone-400 uppercase tracking-wide
                           text-[10px] mb-0.5">"Nodes (click to filter)"</p>
                <div class="flex flex-col gap-1">
                    <LegendToggle label="Article"   color="#d97706" shape="circle"   show=show_articles />
                    <LegendToggle label="Project"   color="#2563eb" shape="diamond"  show=show_projects />
                    <LegendToggle label="Area"      color="#16a34a" shape="rect"     show=show_areas />
                    <LegendToggle label="Resource"  color="#9333ea" shape="hexagon"  show=show_resources />
                    <LegendToggle label="Reference" color="#dc2626" shape="triangle" show=show_references />
                </div>
                // Edge line styles (non-interactive)
                <p class="font-semibold text-stone-500 dark:text-stone-400 uppercase tracking-wide
                           text-[10px] mt-1 mb-0.5">"Edges"</p>
                <div class="flex flex-col gap-1">
                    <LegendEdge label="References"  color="#d97706" dash="none"    width="1.5" />
                    <LegendEdge label="Contains"    color="#22c55e" dash="none"    width="2.5" />
                    <LegendEdge label="Related to"  color="#a855f7" dash="8,4"     width="1.5" />
                    <LegendEdge label="Depends on"  color="#f97316" dash="3,4"     width="1.5" />
                    <LegendEdge label="Derived from" color="#ec4899" dash="8,3,2,3" width="1.5" />
                    <LegendEdge label="Wiki link"   color="#60a5fa" dash="4,2"     width="1.0" />
                </div>
            </div>
            {move || {
                if loading.get() {
                    return view! {
                        <div class="flex items-center justify-center h-full text-stone-400 dark:text-stone-600">
                            <span class="text-sm">"Loading graph\u{2026}"</span>
                        </div>
                    }
                    .into_any();
                }
                if let Some(err) = error_msg.get() {
                    return view! {
                        <div class="flex items-center justify-center h-full text-red-400">
                            <span class="text-sm">{err}</span>
                        </div>
                    }
                    .into_any();
                }
                let all_nodes = nodes_sig.get();
                if all_nodes.is_empty() {
                    return view! {
                        <div class="flex items-center justify-center h-full text-stone-400 dark:text-stone-600">
                            <span class="text-sm">
                                "No nodes yet. Create some notes to see the graph."
                            </span>
                        </div>
                    }
                    .into_any();
                }

                // Apply type-visibility filter.
                let nodes: Vec<Node> = all_nodes
                    .into_iter()
                    .filter(|n| match &n.node_type {
                        NodeType::Article   => show_articles.get(),
                        NodeType::Project   => show_projects.get(),
                        NodeType::Area      => show_areas.get(),
                        NodeType::Resource  => show_resources.get(),
                        NodeType::Reference => show_references.get(),
                    })
                    .collect();

                // Retain only edges where both endpoints are visible.
                let visible_ids: std::collections::HashSet<Uuid> =
                    nodes.iter().map(|n| n.id.0).collect();
                let edges: Vec<Edge> = edges_sig
                    .get()
                    .into_iter()
                    .filter(|e| {
                        visible_ids.contains(&e.source_id.0)
                            && visible_ids.contains(&e.target_id.0)
                    })
                    .collect();

                // Title lookup map for hover cards.
                let title_map: HashMap<Uuid, String> = nodes
                    .iter()
                    .map(|n| (n.id.0, n.title.clone()))
                    .collect();

                // ── Edge SVGs ────────────────────────────────────────────────
                // Each edge: transparent wide hit-area path + visible styled path.
                // No text label on the edge — visual encoding (colour + dash) carries type.
                let edge_svgs: Vec<_> = edges
                    .iter()
                    .map(|edge| {
                        let src = edge.source_id.0;
                        let tgt = edge.target_id.0;
                        let color = edge_color(&edge.edge_type);
                        let dash = edge_dash(&edge.edge_type);
                        let sw = edge_stroke_width(&edge.edge_type);
                        let marker_id = edge_marker_id(&edge.edge_type);

                        let hover_info = EdgeHover {
                            type_label: edge_label(&edge.edge_type),
                            custom_label: edge.label.clone(),
                            src_title: title_map.get(&src).cloned().unwrap_or_default(),
                            tgt_title: title_map.get(&tgt).cloned().unwrap_or_default(),
                            src_id: src,
                            tgt_id: tgt,
                        };
                        let hover_info_out = hover_info.clone();

                        let dash_part = if dash == "none" {
                            String::new()
                        } else {
                            format!(" stroke-dasharray: {dash};")
                        };
                        let path_style = format!(
                            "stroke: {color}; stroke-width: {sw}; stroke-opacity: 0.85; \
                             fill: none; marker-end: url(#{marker_id});{dash_part}"
                        );

                        let d_vis = move || {
                            let pos = positions.get();
                            let (x1, y1) = pos.get(&src).copied().unwrap_or((0.0, 0.0));
                            let (x2, y2) = pos.get(&tgt).copied().unwrap_or((0.0, 0.0));
                            compute_path(x1, y1, x2, y2)
                        };
                        let d_hit = move || {
                            let pos = positions.get();
                            let (x1, y1) = pos.get(&src).copied().unwrap_or((0.0, 0.0));
                            let (x2, y2) = pos.get(&tgt).copied().unwrap_or((0.0, 0.0));
                            compute_path(x1, y1, x2, y2)
                        };

                        view! {
                            <g
                                on:mouseover=move |_| edge_hover.set(Some(hover_info.clone()))
                                on:mouseout=move |_| edge_hover.set(None)
                            >
                                // Wide transparent hit area for easier hovering.
                                <path
                                    d=d_hit
                                    style="stroke: transparent; stroke-width: 14; fill: none;"
                                />
                                <path d=d_vis style=path_style />
                                <title>{hover_info_out.type_label}</title>
                            </g>
                        }
                        .into_any()
                    })
                    .collect();

                // ── Node SVGs ────────────────────────────────────────────────
                // Shape encodes type; full title shown below in a dark pill.
                let node_svgs: Vec<_> = nodes
                    .iter()
                    .map(|node| {
                        let id = node.id.0;
                        let node_id: NodeId = node.id;
                        let title = node.title.clone();
                        let title_text = title.clone();
                        let node_type = node.node_type.clone();
                        let fill = node_fill(&node_type);
                        let stroke_c = node_stroke_color(&node_type);

                        // Data for the hover summary card.
                        let hover_info = NodeHoverInfo {
                            title: node.title.clone(),
                            node_type: format!("{:?}", node.node_type).to_lowercase(),
                            status: format!("{:?}", node.status).to_lowercase(),
                            body_preview: node.body.as_deref().and_then(node_body_preview),
                            node_id: node.id.0,
                        };

                        // Estimate background-pill width from title char count.
                        let bg_w = (title.chars().count() as f64 * 5.4 + 10.0_f64).max(32.0_f64);

                        let shape_el: AnyView = match node_type {
                            NodeType::Article => view! {
                                <circle
                                    cx=move || {
                                        format!(
                                            "{:.1}",
                                            positions.get().get(&id).map(|p| p.0).unwrap_or(W / 2.0)
                                        )
                                    }
                                    cy=move || {
                                        format!(
                                            "{:.1}",
                                            positions.get().get(&id).map(|p| p.1).unwrap_or(H / 2.0)
                                        )
                                    }
                                    r="20"
                                    style=format!(
                                        "fill: {fill}; stroke: {stroke_c}; stroke-width: 1.5px;"
                                    )
                                />
                            }
                            .into_any(),
                            NodeType::Project => view! {
                                <polygon
                                    points=move || {
                                        let pos = positions.get();
                                        let (cx, cy) =
                                            pos.get(&id).copied().unwrap_or((W / 2.0, H / 2.0));
                                        diamond_points(cx, cy)
                                    }
                                    style=format!(
                                        "fill: {fill}; stroke: {stroke_c}; stroke-width: 1.5px;"
                                    )
                                />
                            }
                            .into_any(),
                            NodeType::Area => view! {
                                <rect
                                    x=move || {
                                        format!(
                                            "{:.1}",
                                            positions
                                                .get()
                                                .get(&id)
                                                .map(|p| p.0 - 22.0)
                                                .unwrap_or(W / 2.0 - 22.0)
                                        )
                                    }
                                    y=move || {
                                        format!(
                                            "{:.1}",
                                            positions
                                                .get()
                                                .get(&id)
                                                .map(|p| p.1 - 15.0)
                                                .unwrap_or(H / 2.0 - 15.0)
                                        )
                                    }
                                    width="44"
                                    height="30"
                                    rx="6"
                                    style=format!(
                                        "fill: {fill}; stroke: {stroke_c}; stroke-width: 1.5px;"
                                    )
                                />
                            }
                            .into_any(),
                            NodeType::Resource => view! {
                                <polygon
                                    points=move || {
                                        let pos = positions.get();
                                        let (cx, cy) =
                                            pos.get(&id).copied().unwrap_or((W / 2.0, H / 2.0));
                                        hexagon_points(cx, cy)
                                    }
                                    style=format!(
                                        "fill: {fill}; stroke: {stroke_c}; stroke-width: 1.5px;"
                                    )
                                />
                            }
                            .into_any(),
                            NodeType::Reference => view! {
                                <polygon
                                    points=move || {
                                        let pos = positions.get();
                                        let (cx, cy) =
                                            pos.get(&id).copied().unwrap_or((W / 2.0, H / 2.0));
                                        triangle_points(cx, cy)
                                    }
                                    style=format!(
                                        "fill: {fill}; stroke: {stroke_c}; stroke-width: 1.5px;"
                                    )
                                />
                            }
                            .into_any(),
                        };

                        view! {
                            <g
                                style="cursor: grab;"
                                on:click=move |ev: MouseEvent| {
                                    // Suppress the click that follows a drag.
                                    if did_drag.get_untracked() {
                                        did_drag.set(false);
                                        ev.stop_propagation();
                                    }
                                }
                                on:dblclick=move |ev: MouseEvent| {
                                    ev.stop_propagation();
                                    current_view.set(View::NodeDetail(node_id));
                                }
                                on:mouseover=move |ev: MouseEvent| {
                                    ev.stop_propagation();
                                    node_hover.set(Some(hover_info.clone()));
                                }
                                on:mouseout=move |_| node_hover.set(None)
                                on:mousedown=move |ev: MouseEvent| {
                                    ev.stop_propagation();
                                    ev.prevent_default();
                                    did_drag.set(false);
                                    let (nx, ny) = positions
                                        .with_untracked(|m| m.get(&id).copied().unwrap_or((0.0, 0.0)));
                                    let mx = (ev.client_x() as f64 - pan_x.get_untracked())
                                        / zoom.get_untracked();
                                    let my = (ev.client_y() as f64 - pan_y.get_untracked())
                                        / zoom.get_untracked();
                                    drag_offset.set((mx - nx, my - ny));
                                    drag_node.set(Some(id));
                                }
                            >
                                <title>{title}</title>
                                {shape_el}
                                // Semi-transparent pill behind title text.
                                <rect
                                    x=move || {
                                        let cx = positions
                                            .get()
                                            .get(&id)
                                            .map(|p| p.0)
                                            .unwrap_or(W / 2.0);
                                        format!("{:.1}", cx - bg_w / 2.0)
                                    }
                                    y=move || {
                                        let cy = positions
                                            .get()
                                            .get(&id)
                                            .map(|p| p.1)
                                            .unwrap_or(H / 2.0);
                                        format!("{:.1}", cy + 22.0)
                                    }
                                    width=format!("{:.1}", bg_w)
                                    height="14"
                                    rx="3"
                                    style="fill: rgba(0,0,0,0.62); pointer-events: none;"
                                />
                                // Full title — no truncation.
                                <text
                                    x=move || {
                                        format!(
                                            "{:.1}",
                                            positions.get().get(&id).map(|p| p.0).unwrap_or(W / 2.0)
                                        )
                                    }
                                    y=move || {
                                        format!(
                                            "{:.1}",
                                            positions
                                                .get()
                                                .get(&id)
                                                .map(|p| p.1 + TEXT_Y_OFFSET)
                                                .unwrap_or(H / 2.0 + TEXT_Y_OFFSET)
                                        )
                                    }
                                    style="text-anchor: middle; font-size: 9px; \
                                           font-weight: 600; fill: #ffffff; pointer-events: none;"
                                >
                                    {title_text}
                                </text>
                            </g>
                        }
                        .into_any()
                    })
                    .collect();

                let svg_cursor = move || {
                    if drag_node.get().is_some() || panning.get() {
                        "cursor: grabbing;"
                    } else {
                        "cursor: default;"
                    }
                };

                // ── Edge hover card ──────────────────────────────────────────
                // Rendered last so it sits on top of all edges and nodes.
                let hover_card = move || {
                    edge_hover.get().map(|h| {
                        let pos = positions.get();
                        let x1 = pos.get(&h.src_id).map(|p| p.0).unwrap_or(W / 2.0);
                        let y1 = pos.get(&h.src_id).map(|p| p.1).unwrap_or(H / 2.0);
                        let x2 = pos.get(&h.tgt_id).map(|p| p.0).unwrap_or(W / 2.0);
                        let y2 = pos.get(&h.tgt_id).map(|p| p.1).unwrap_or(H / 2.0);
                        let cx = (x1 + x2) / 2.0;
                        let cy = (y1 + y2) / 2.0 - 24.0;

                        let relation = h.type_label.to_string();
                        let arrow_line = format!(
                            "{} \u{2192} {}",
                            truncate(&h.src_title, 20),
                            truncate(&h.tgt_title, 20)
                        );
                        let custom_lbl = h
                            .custom_label
                            .as_deref()
                            .filter(|s| !s.is_empty())
                            .map(|s| truncate(s, 28));

                        let line_count = if custom_lbl.is_some() { 3.0_f64 } else { 2.0_f64 };
                        let card_h = line_count * 14.0 + 10.0;
                        let max_chars = [
                            relation.len(),
                            arrow_line.len(),
                            custom_lbl.as_deref().map(str::len).unwrap_or(0),
                        ]
                        .into_iter()
                        .max()
                        .unwrap_or(0);
                        let card_w = (max_chars as f64 * 5.8 + 16.0_f64).max(80.0_f64);

                        let cx_s = format!("{:.1}", cx);
                        let card_x = format!("{:.1}", cx - card_w / 2.0);
                        let card_y = format!("{:.1}", cy - 5.0);
                        let y_rel = format!("{:.1}", cy + 9.0);
                        let y_arrow = format!("{:.1}", cy + 23.0);
                        let y_cust = format!("{:.1}", cy + 37.0);

                        view! {
                            <g style="pointer-events: none;">
                                <rect
                                    x=card_x
                                    y=card_y
                                    width=format!("{:.1}", card_w)
                                    height=format!("{:.1}", card_h)
                                    rx="5"
                                    style="fill: rgba(12,12,12,0.92); \
                                           stroke: rgba(255,255,255,0.18); stroke-width: 0.5;"
                                />
                                <text
                                    x=cx_s.clone()
                                    y=y_rel
                                    style="text-anchor: middle; font-size: 10px; \
                                           font-weight: 700; fill: #fbbf24;"
                                >
                                    {relation}
                                </text>
                                <text
                                    x=cx_s.clone()
                                    y=y_arrow
                                    style="text-anchor: middle; font-size: 8.5px; fill: #d4d4d4;"
                                >
                                    {arrow_line}
                                </text>
                                {custom_lbl.map(|lbl| {
                                    view! {
                                        <text
                                            x=cx_s
                                            y=y_cust
                                            style="text-anchor: middle; font-size: 8.5px; \
                                                   font-style: italic; fill: #a3a3a3;"
                                        >
                                            {lbl}
                                        </text>
                                    }
                                })}
                            </g>
                        }
                        .into_any()
                    })
                };

                view! {
                    <svg
                        id="graph-svg"
                        class="w-full h-full"
                        style=svg_cursor
                        on:mousedown=move |ev: MouseEvent| {
                            if drag_node.get_untracked().is_none() {
                                panning.set(true);
                                last_mx.set(ev.client_x() as f64);
                                last_my.set(ev.client_y() as f64);
                            }
                        }
                        on:mousemove=move |ev: MouseEvent| {
                            if let Some(nid) = drag_node.get_untracked() {
                                ev.prevent_default();
                                did_drag.set(true);
                                let mx = (ev.client_x() as f64 - pan_x.get_untracked())
                                    / zoom.get_untracked();
                                let my = (ev.client_y() as f64 - pan_y.get_untracked())
                                    / zoom.get_untracked();
                                let (ox, oy) = drag_offset.get_untracked();
                                let new_x = (mx - ox).clamp(MARGIN, W - MARGIN);
                                let new_y = (my - oy).clamp(MARGIN, H - MARGIN);
                                positions.update(|map| {
                                    map.insert(nid, (new_x, new_y));
                                });
                            } else if panning.get_untracked() {
                                let mx = ev.client_x() as f64;
                                let my = ev.client_y() as f64;
                                pan_x.update(|p| *p += mx - last_mx.get_untracked());
                                pan_y.update(|p| *p += my - last_my.get_untracked());
                                last_mx.set(mx);
                                last_my.set(my);
                            }
                        }
                        on:mouseup=move |_ev: MouseEvent| {
                            if let Some(nid) = drag_node.get_untracked() {
                                let (x, y) = positions
                                    .with_untracked(|m| m.get(&nid).copied().unwrap_or((0.0, 0.0)));
                                spawn_local(async move {
                                    let _ = save_position(nid, x, y).await;
                                });
                                drag_node.set(None);
                            }
                            panning.set(false);
                        }
                        on:mouseleave=move |_: MouseEvent| {
                            if let Some(nid) = drag_node.get_untracked() {
                                let (x, y) = positions
                                    .with_untracked(|m| m.get(&nid).copied().unwrap_or((0.0, 0.0)));
                                spawn_local(async move {
                                    let _ = save_position(nid, x, y).await;
                                });
                                drag_node.set(None);
                            }
                            panning.set(false);
                        }
                        on:wheel=move |ev: WheelEvent| {
                            ev.prevent_default();
                            let factor = if ev.delta_y() > 0.0 { 0.9_f64 } else { 1.1_f64 };
                            zoom.update(|z| *z = (*z * factor).clamp(0.1, 8.0));
                        }
                    >
                        <g transform=move || {
                            format!(
                                "translate({:.1},{:.1}) scale({:.3})",
                                pan_x.get(),
                                pan_y.get(),
                                zoom.get(),
                            )
                        }>
                            {edge_svgs}
                            {node_svgs}
                            {hover_card}
                            // Node summary card — shown on hover, rendered last so it's on top.
                            {move || {
                                node_hover.get().map(|h| {
                                    let pos = positions.get();
                                    let (nx, ny) = pos.get(&h.node_id).copied().unwrap_or((W / 2.0, H / 2.0));

                                    let type_lbl = type_label(&h.node_type);
                                    let status_lbl = status_label(&h.status);
                                    let status_col = status_color_hex(&h.status);

                                    // Card dimensions
                                    let card_w = (h.title.chars().count() as f64 * 6.0 + 20.0)
                                        .clamp(120.0_f64, 240.0_f64);
                                    let has_preview = h.body_preview.is_some();
                                    let card_h = if has_preview { 66.0_f64 } else { 50.0_f64 };
                                    let cx = nx;
                                    let cy = ny - NODE_R - card_h - 8.0;

                                    let cx_s  = format!("{:.1}", cx);
                                    let card_x = format!("{:.1}", cx - card_w / 2.0);
                                    let card_y = format!("{:.1}", cy);
                                    let y_title  = format!("{:.1}", cy + 13.0);
                                    let y_meta   = format!("{:.1}", cy + 27.0);
                                    let y_prev   = format!("{:.1}", cy + 41.0);
                                    let meta_txt = format!("{type_lbl}  ·  {status_lbl}");

                                    let preview_text = h.body_preview.clone();

                                    view! {
                                        <g style="pointer-events: none;">
                                            // Card background
                                            <rect
                                                x=card_x
                                                y=card_y
                                                width=format!("{:.1}", card_w)
                                                height=format!("{:.1}", card_h)
                                                rx="6"
                                                style="fill: rgba(12,12,12,0.92); \
                                                       stroke: rgba(255,255,255,0.18); \
                                                       stroke-width: 0.5;"
                                            />
                                            // Title
                                            <text
                                                x=cx_s.clone()
                                                y=y_title
                                                style="text-anchor: middle; font-size: 10.5px; \
                                                       font-weight: 700; fill: #f5f5f4;"
                                            >
                                                {h.title.clone()}
                                            </text>
                                            // Type · Status meta line
                                            <text
                                                x=cx_s.clone()
                                                y=y_meta
                                                style=format!(
                                                    "text-anchor: middle; font-size: 8px; fill: {};",
                                                    status_col
                                                )
                                            >
                                                {meta_txt}
                                            </text>
                                            // Body preview (optional)
                                            {preview_text.map(|preview| view! {
                                                <text
                                                    x=cx_s
                                                    y=y_prev
                                                    style="text-anchor: middle; font-size: 7.5px; \
                                                           fill: #a8a29e;"
                                                >
                                                    {preview}
                                                </text>
                                            })}
                                            // "Double-click to open" hint
                                            <text
                                                x=format!("{:.1}", cx)
                                                y=format!("{:.1}", cy + card_h - 5.0)
                                                style="text-anchor: middle; font-size: 7px; \
                                                       fill: rgba(255,255,255,0.3);"
                                            >
                                                "double-click to open"
                                            </text>
                                        </g>
                                    }
                                    .into_any()
                                })
                            }}
                        </g>
                    </svg>
                }
                .into_any()
            }}
        </div>
    }
}

// ── Legend sub-components ────────────────────────────────────────────────────

/// Clickable legend row that toggles node-type visibility.
#[component]
fn LegendToggle(
    label: &'static str,
    color: &'static str,
    shape: &'static str,
    show: RwSignal<bool>,
) -> impl IntoView {
    let icon_view = legend_shape_icon(color, shape);
    view! {
        <button
            class=move || {
                let dimmed = if show.get() { "" } else { " opacity-40" };
                format!(
                    "flex items-center gap-1.5 cursor-pointer select-none w-full text-left\
                     hover:opacity-80 transition-opacity{dimmed}"
                )
            }
            title=move || if show.get() { format!("Hide {label}") } else { format!("Show {label}") }
            on:click=move |_| show.update(|v| *v = !*v)
        >
            {icon_view}
            <span class="text-stone-700 dark:text-stone-300">{label}</span>
            {move || (!show.get()).then(|| view! {
                <span class="ml-auto text-stone-400 dark:text-stone-500 text-[10px]">"hidden"</span>
            })}
        </button>
    }
}

/// Builds the small SVG icon used in legend rows (shared by LegendToggle and LegendShape).
fn legend_shape_icon(color: &'static str, shape: &'static str) -> AnyView {
    match shape {
        "diamond" => view! {
            <svg width="14" height="14" viewBox="0 0 14 14" style="flex-shrink:0;">
                <polygon
                    points="7,1 13,7 7,13 1,7"
                    style=format!("fill:{color}; stroke:{color}; stroke-width:0.5;")
                />
            </svg>
        }
        .into_any(),
        "rect" => view! {
            <svg width="14" height="14" viewBox="0 0 14 14" style="flex-shrink:0;">
                <rect
                    x="1" y="3" width="12" height="8" rx="2"
                    style=format!("fill:{color}; stroke:{color}; stroke-width:0.5;")
                />
            </svg>
        }
        .into_any(),
        "hexagon" => view! {
            <svg width="14" height="14" viewBox="0 0 14 14" style="flex-shrink:0;">
                <polygon
                    points="7,1 12.2,4 12.2,10 7,13 1.8,10 1.8,4"
                    style=format!("fill:{color}; stroke:{color}; stroke-width:0.5;")
                />
            </svg>
        }
        .into_any(),
        "triangle" => view! {
            <svg width="14" height="14" viewBox="0 0 14 14" style="flex-shrink:0;">
                <polygon
                    points="7,1 13,13 1,13"
                    style=format!("fill:{color}; stroke:{color}; stroke-width:0.5;")
                />
            </svg>
        }
        .into_any(),
        _ => view! {
            <svg width="14" height="14" viewBox="0 0 14 14" style="flex-shrink:0;">
                <circle
                    cx="7" cy="7" r="6"
                    style=format!("fill:{color}; stroke:{color}; stroke-width:0.5;")
                />
            </svg>
        }
        .into_any(),
    }
}

#[component]
fn LegendShape(label: &'static str, color: &'static str, shape: &'static str) -> impl IntoView {
    let icon: AnyView = match shape {
        "diamond" => view! {
            <svg width="14" height="14" viewBox="0 0 14 14" style="flex-shrink:0;">
                <polygon
                    points="7,1 13,7 7,13 1,7"
                    style=format!("fill:{color}; stroke:{color}; stroke-width:0.5;")
                />
            </svg>
        }
        .into_any(),
        "rect" => view! {
            <svg width="14" height="14" viewBox="0 0 14 14" style="flex-shrink:0;">
                <rect
                    x="1" y="3" width="12" height="8" rx="2"
                    style=format!("fill:{color}; stroke:{color}; stroke-width:0.5;")
                />
            </svg>
        }
        .into_any(),
        "hexagon" => view! {
            <svg width="14" height="14" viewBox="0 0 14 14" style="flex-shrink:0;">
                <polygon
                    points="7,1 12.2,4 12.2,10 7,13 1.8,10 1.8,4"
                    style=format!("fill:{color}; stroke:{color}; stroke-width:0.5;")
                />
            </svg>
        }
        .into_any(),
        "triangle" => view! {
            <svg width="14" height="14" viewBox="0 0 14 14" style="flex-shrink:0;">
                <polygon
                    points="7,1 13,13 1,13"
                    style=format!("fill:{color}; stroke:{color}; stroke-width:0.5;")
                />
            </svg>
        }
        .into_any(),
        _ => view! {
            // circle (Article default)
            <svg width="14" height="14" viewBox="0 0 14 14" style="flex-shrink:0;">
                <circle
                    cx="7" cy="7" r="6"
                    style=format!("fill:{color}; stroke:{color}; stroke-width:0.5;")
                />
            </svg>
        }
        .into_any(),
    };

    view! {
        <div class="flex items-center gap-1.5">
            {icon}
            <span class="text-stone-700 dark:text-stone-300">{label}</span>
        </div>
    }
}

#[component]
fn LegendEdge(
    label: &'static str,
    color: &'static str,
    dash: &'static str,
    width: &'static str,
) -> impl IntoView {
    let dash_attr = if dash == "none" {
        String::new()
    } else {
        format!(" stroke-dasharray:{dash};")
    };
    let line_style = format!("stroke:{color}; stroke-width:{width}; fill:none;{dash_attr}");

    view! {
        <div class="flex items-center gap-1.5">
            <svg width="20" height="10" viewBox="0 0 20 10" style="flex-shrink:0;">
                <line x1="1" y1="5" x2="19" y2="5" style=line_style />
            </svg>
            <span class="text-stone-700 dark:text-stone-300">{label}</span>
        </div>
    }
}
