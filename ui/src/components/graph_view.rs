/// Visual knowledge graph — force-directed SVG node-link diagram.
///
/// Fetches all nodes and edges, runs a Fruchterman-Reingold layout in WASM,
/// then overlays saved positions from the API.  Nodes are draggable; positions
/// are persisted to the DB on mouse-up.  The canvas supports pan (drag on
/// background) and zoom (wheel).
///
/// SVG presentation attributes (stroke-width, fill-opacity, etc.) are set via
/// the `style` attribute because Leptos 0.8 writes `attr:` prefixes literally
/// into the DOM for SVG elements rather than calling `setAttribute`.
///
/// Arrowhead `<marker>` elements are injected imperatively via `web_sys` after
/// first render, since `<marker>` is not a known Leptos element.
use std::collections::HashMap;

use leptos::prelude::*;
use uuid::Uuid;
use wasm_bindgen_futures::spawn_local;
use web_sys::{MouseEvent, WheelEvent};

use common::{
    edge::{Edge, EdgeType},
    id::NodeId,
    node::Node,
};

use crate::{
    api::{fetch_all_edges, fetch_nodes, fetch_positions, save_position},
    app::View,
    components::dark_mode_toggle::Theme,
};

// SVG coordinate-space canvas size
const W: f64 = 1000.0;
const H: f64 = 700.0;
const MARGIN: f64 = 80.0;
const NODE_R: f64 = 28.0;
/// Arrow tip lands NODE_R + 4 px from the target centre.
const ARROW_OFFSET: f64 = NODE_R + 4.0;

// ── Edge-type helpers ──────────────────────────────────────────────────────

fn edge_color(et: &EdgeType) -> &'static str {
    match et {
        EdgeType::References  => "#d97706",
        EdgeType::Contains    => "#22c55e",
        EdgeType::RelatedTo   => "#a855f7",
        EdgeType::DependsOn   => "#f97316",
        EdgeType::DerivedFrom => "#ec4899",
        EdgeType::WikiLink    => "#fb923c",
    }
}

fn edge_label(et: &EdgeType) -> &'static str {
    match et {
        EdgeType::References  => "references",
        EdgeType::Contains    => "contains",
        EdgeType::RelatedTo   => "related to",
        EdgeType::DependsOn   => "depends on",
        EdgeType::DerivedFrom => "derived from",
        EdgeType::WikiLink    => "wiki link",
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

// ── SVG marker injection ───────────────────────────────────────────────────

/// Inject arrowhead `<marker>` elements into the graph SVG's `<defs>`.
///
/// Leptos 0.8 writes `attr:foo` prefixes literally into the DOM (via
/// `setAttribute("attr:foo", val)`) for SVG elements it does not recognise
/// (such as `<marker>`).  Creating them with `web_sys` and the correct SVG
/// namespace avoids this issue.
fn inject_svg_markers() {
    let Some(win) = web_sys::window() else { return };
    let Some(doc) = win.document() else { return };
    let Some(svg) = doc.get_element_by_id("graph-svg") else { return };
    // Guard: don't double-inject.
    if svg.query_selector("marker").ok().flatten().is_some() {
        return;
    }

    let ns = "http://www.w3.org/2000/svg";
    let Ok(defs) = doc.create_element_ns(Some(ns), "defs") else { return };

    const ARROWS: &[(&str, &str)] = &[
        ("arrow-references",  "#d97706"),
        ("arrow-contains",    "#22c55e"),
        ("arrow-related-to",  "#a855f7"),
        ("arrow-depends-on",  "#f97316"),
        ("arrow-derived-from","#ec4899"),
        ("arrow-wiki-link",   "#fb923c"),
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

    // Insert as first child so markers precede all drawing elements.
    if let Some(first) = svg.first_child() {
        let _ = svg.insert_before(&defs, Some(&first));
    } else {
        let _ = svg.append_child(&defs);
    }
}

// ── Layout ─────────────────────────────────────────────────────────────────

/// Fruchterman-Reingold spring layout (200 iterations, O(n²) repulsion).
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

// ── Component ──────────────────────────────────────────────────────────────

#[component]
pub fn GraphView() -> impl IntoView {
    let current_view = use_context::<RwSignal<View>>().expect("View signal must be provided");
    let theme = use_context::<RwSignal<Theme>>().map(|t| t.read_only());

    let loading = RwSignal::new(true);
    let error_msg = RwSignal::new(Option::<String>::None);
    let nodes_sig: RwSignal<Vec<Node>> = RwSignal::new(vec![]);
    let edges_sig: RwSignal<Vec<Edge>> = RwSignal::new(vec![]);
    let positions: RwSignal<HashMap<Uuid, (f64, f64)>> = RwSignal::new(HashMap::new());

    // Pan/zoom state
    let pan_x = RwSignal::new(0.0_f64);
    let pan_y = RwSignal::new(0.0_f64);
    let zoom = RwSignal::new(1.0_f64);
    let panning = RwSignal::new(false);
    let last_mx = RwSignal::new(0.0_f64);
    let last_my = RwSignal::new(0.0_f64);

    // Drag state
    let drag_node: RwSignal<Option<Uuid>> = RwSignal::new(None);
    let drag_offset: RwSignal<(f64, f64)> = RwSignal::new((0.0, 0.0));
    let did_drag = RwSignal::new(false);

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

                    // Defer marker injection to the next tick so Leptos has
                    // finished rendering the SVG element into the DOM.
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
                let nodes = nodes_sig.get();
                if nodes.is_empty() {
                    return view! {
                        <div class="flex items-center justify-center h-full text-stone-400 dark:text-stone-600">
                            <span class="text-sm">
                                "No nodes yet. Create some notes to see the graph."
                            </span>
                        </div>
                    }
                    .into_any();
                }
                let edges = edges_sig.get();

                // ── Edge SVGs ───────────────────────────────────────────────
                // Coloured by edge type, with directional arrowheads and a
                // tooltip showing the type label (and custom label if present).
                // SVG presentation attributes are set via `style` because
                // Leptos 0.8 writes `attr:` prefixes literally for SVG elements.
                let edge_svgs: Vec<_> = edges
                    .iter()
                    .map(|edge| {
                        let src = edge.source_id.0;
                        let tgt = edge.target_id.0;
                        let color = edge_color(&edge.edge_type);
                        let marker_id = edge_marker_id(&edge.edge_type);
                        let type_str = edge_label(&edge.edge_type);
                        let tooltip_text = match &edge.label {
                            Some(l) => format!("{type_str}: {l}"),
                            None => type_str.to_string(),
                        };
                        let mid_label = edge.label.clone()
                            .unwrap_or_else(|| type_str.to_string());
                        // Static styles — no reactive closure needed.
                        let path_style = format!(
                            "stroke: {color}; stroke-width: 1.5; stroke-opacity: 0.75; \
                             fill: none; marker-end: url(#{marker_id});"
                        );
                        let label_style = format!(
                            "font-size: 9px; font-style: italic; fill: {color}; \
                             pointer-events: none; text-anchor: middle;"
                        );

                        // Shortened path: starts at source circle boundary, ends
                        // just before target circle so the arrowhead is visible.
                        let d = move || {
                            let pos = positions.get();
                            let (x1, y1) = pos.get(&src).copied().unwrap_or((0.0, 0.0));
                            let (x2, y2) = pos.get(&tgt).copied().unwrap_or((0.0, 0.0));
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
                        };
                        let mid_x = move || {
                            let pos = positions.get();
                            let x1 = pos.get(&src).map(|p| p.0).unwrap_or(0.0);
                            let x2 = pos.get(&tgt).map(|p| p.0).unwrap_or(0.0);
                            format!("{:.1}", (x1 + x2) / 2.0)
                        };
                        let mid_y = move || {
                            let pos = positions.get();
                            let y1 = pos.get(&src).map(|p| p.1).unwrap_or(0.0);
                            let y2 = pos.get(&tgt).map(|p| p.1).unwrap_or(0.0);
                            format!("{:.1}", (y1 + y2) / 2.0 - 5.0)
                        };

                        view! {
                            <g>
                                <title>{tooltip_text}</title>
                                <path d=d style=path_style />
                                <text x=mid_x y=mid_y style=label_style>
                                    {mid_label}
                                </text>
                            </g>
                        }
                        .into_any()
                    })
                    .collect();

                // ── Node SVGs ───────────────────────────────────────────────
                // White-halo text (paint-order: stroke fill) via `style` for
                // contrast against edges; font-size reduced to 10 px.
                let node_svgs: Vec<_> = nodes
                    .iter()
                    .map(|node| {
                        let id = node.id.0;
                        let node_id: NodeId = node.id;
                        let title = node.title.clone();
                        let display: String = if title.chars().count() > 16 {
                            let s: String = title.chars().take(16).collect();
                            format!("{s}\u{2026}")
                        } else {
                            title
                        };

                        view! {
                            <g
                                style="cursor: grab;"
                                on:click=move |ev: MouseEvent| {
                                    if did_drag.get_untracked() {
                                        did_drag.set(false);
                                        ev.stop_propagation();
                                        return;
                                    }
                                    ev.stop_propagation();
                                    current_view.set(View::NodeDetail(node_id));
                                }
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
                                <circle
                                    cx=move || format!("{:.1}", positions.get().get(&id).map(|p| p.0).unwrap_or(W / 2.0))
                                    cy=move || format!("{:.1}", positions.get().get(&id).map(|p| p.1).unwrap_or(H / 2.0))
                                    r="28"
                                    style="fill: #f59e0b; fill-opacity: 0.15; stroke: #f59e0b; stroke-width: 2px;"
                                />
                                <text
                                    x=move || format!("{:.1}", positions.get().get(&id).map(|p| p.0).unwrap_or(W / 2.0))
                                    y=move || format!("{:.1}", positions.get().get(&id).map(|p| p.1 + 5.0).unwrap_or(H / 2.0 + 5.0))
                                    style=move || {
                                        let is_dark = theme
                                            .map(|t| t.get() == Theme::Dark)
                                            .unwrap_or(false);
                                        let (text_fill, halo) = if is_dark {
                                            ("#fcd34d", "#0c0a09")  // amber-300 on stone-950
                                        } else {
                                            ("#92400e", "#fafaf9")  // amber-900 on stone-50
                                        };
                                        format!(
                                            "text-anchor: middle; font-size: 10px; font-weight: 600; \
                                             fill: {text_fill}; stroke: {halo}; stroke-width: 3px; \
                                             paint-order: stroke fill; pointer-events: none;"
                                        )
                                    }
                                >
                                    {display}
                                </text>
                            </g>
                        }
                        .into_any()
                    })
                    .collect();

                // Cursor style for the SVG.
                let svg_cursor = move || {
                    if drag_node.get().is_some() || panning.get() {
                        "cursor: grabbing;"
                    } else {
                        "cursor: default;"
                    }
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
                                positions.update(|map| { map.insert(nid, (new_x, new_y)); });
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
                                spawn_local(async move { let _ = save_position(nid, x, y).await; });
                                drag_node.set(None);
                            }
                            panning.set(false);
                        }
                        on:mouseleave=move |_: MouseEvent| {
                            if let Some(nid) = drag_node.get_untracked() {
                                let (x, y) = positions
                                    .with_untracked(|m| m.get(&nid).copied().unwrap_or((0.0, 0.0)));
                                spawn_local(async move { let _ = save_position(nid, x, y).await; });
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
                        <g transform=move || format!(
                            "translate({:.1},{:.1}) scale({:.3})",
                            pan_x.get(), pan_y.get(), zoom.get(),
                        )>
                            {edge_svgs}
                            {node_svgs}
                        </g>
                    </svg>
                }
                .into_any()
            }}
        </div>
    }
}
