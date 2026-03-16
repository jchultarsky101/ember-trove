/// Visual knowledge graph — force-directed SVG node-link diagram.
///
/// Fetches all nodes and edges, runs a Fruchterman-Reingold layout in WASM,
/// then overlays saved positions from the API.  Nodes are draggable; positions
/// are persisted to the DB on mouse-up.  The canvas supports pan (drag on
/// background) and zoom (wheel).
use std::collections::HashMap;

use leptos::prelude::*;
use uuid::Uuid;
use wasm_bindgen_futures::spawn_local;
use web_sys::{MouseEvent, WheelEvent};

use common::{edge::Edge, id::NodeId, node::Node};

use crate::{
    api::{fetch_all_edges, fetch_nodes, fetch_positions, save_position},
    app::View,
};

// SVG coordinate-space canvas size
const W: f64 = 1000.0;
const H: f64 = 700.0;
const MARGIN: f64 = 80.0;

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

        // Repulsion (symmetric, O(n²/2))
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

        // Attraction along edges
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

        // Apply displacement with cooling temperature
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

#[component]
pub fn GraphView() -> impl IntoView {
    let current_view = use_context::<RwSignal<View>>().expect("View signal must be provided");

    let loading = RwSignal::new(true);
    let error_msg = RwSignal::new(Option::<String>::None);
    let nodes_sig: RwSignal<Vec<Node>> = RwSignal::new(vec![]);
    let edges_sig: RwSignal<Vec<Edge>> = RwSignal::new(vec![]);
    // Reactive positions map — updated live during drag so edges follow.
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
    // Offset = (mouse_svg_x - node_x) at drag-start, so the grab point stays fixed.
    let drag_offset: RwSignal<(f64, f64)> = RwSignal::new((0.0, 0.0));

    // Fetch nodes + edges once on mount, run FR layout, then override with saved positions.
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

                    // 1. Run FR layout as the baseline.
                    let mut layout = force_layout(&node_ids, &edge_pairs);

                    // 2. Override with any saved positions from the API.
                    if let Ok(saved) = fetch_positions().await {
                        for pos in saved {
                            layout.insert(pos.node_id.0, (pos.x, pos.y));
                        }
                    }

                    positions.set(layout);
                    nodes_sig.set(nodes);
                    edges_sig.set(edges);
                    loading.set(false);
                }
            }
        });
    });

    view! {
        <div class="relative w-full h-full overflow-hidden bg-gray-50 dark:bg-gray-950 select-none">
            {move || {
                if loading.get() {
                    return view! {
                        <div class="flex items-center justify-center h-full text-gray-400 dark:text-gray-600">
                            <span class="text-sm">"Loading graph…"</span>
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
                        <div class="flex items-center justify-center h-full text-gray-400 dark:text-gray-600">
                            <span class="text-sm">
                                "No nodes yet. Create some notes to see the graph."
                            </span>
                        </div>
                    }
                    .into_any();
                }
                let edges = edges_sig.get();

                // Build node SVG elements — each with its own drag handler.
                // Node positions are read reactively inside each node's closure
                // so only the moved node re-renders, not the whole list.
                let node_svgs: Vec<_> = nodes
                    .iter()
                    .map(|node| {
                        let id = node.id.0;
                        let node_id: NodeId = node.id;
                        let title = node.title.clone();
                        let display: String = if title.chars().count() > 14 {
                            let s: String = title.chars().take(14).collect();
                            format!("{s}…")
                        } else {
                            title
                        };

                        view! {
                            <g
                                style="cursor: grab;"
                                on:click=move |ev: MouseEvent| {
                                    // Only fire click when not dragging.
                                    if drag_node.get_untracked().is_none() {
                                        ev.stop_propagation();
                                        current_view.set(View::NodeDetail(node_id));
                                    }
                                }
                                on:mousedown=move |ev: MouseEvent| {
                                    ev.stop_propagation();
                                    ev.prevent_default();
                                    let (nx, ny) = positions
                                        .with_untracked(|m| m.get(&id).copied().unwrap_or((0.0, 0.0)));
                                    // Convert viewport coords to SVG canvas coords.
                                    let mx = (ev.client_x() as f64 - pan_x.get_untracked())
                                        / zoom.get_untracked();
                                    let my = (ev.client_y() as f64 - pan_y.get_untracked())
                                        / zoom.get_untracked();
                                    drag_offset.set((mx - nx, my - ny));
                                    drag_node.set(Some(id));
                                }
                            >
                                <circle
                                    cx=move || {
                                        format!(
                                            "{:.1}",
                                            positions
                                                .get()
                                                .get(&id)
                                                .map(|p| p.0)
                                                .unwrap_or(W / 2.0),
                                        )
                                    }
                                    cy=move || {
                                        format!(
                                            "{:.1}",
                                            positions
                                                .get()
                                                .get(&id)
                                                .map(|p| p.1)
                                                .unwrap_or(H / 2.0),
                                        )
                                    }
                                    r="28"
                                    fill="#f59e0b"
                                    attr:fill-opacity="0.15"
                                    stroke="#f59e0b"
                                    attr:stroke-width="2"
                                />
                                <text
                                    x=move || {
                                        format!(
                                            "{:.1}",
                                            positions
                                                .get()
                                                .get(&id)
                                                .map(|p| p.0)
                                                .unwrap_or(W / 2.0),
                                        )
                                    }
                                    y=move || {
                                        format!(
                                            "{:.1}",
                                            positions
                                                .get()
                                                .get(&id)
                                                .map(|p| p.1 + 5.0)
                                                .unwrap_or(H / 2.0 + 5.0),
                                        )
                                    }
                                    attr:text-anchor="middle"
                                    attr:font-size="11"
                                    attr:font-weight="500"
                                    fill="#374151"
                                >
                                    {display}
                                </text>
                            </g>
                        }
                        .into_any()
                    })
                    .collect();

                // Build edge SVG elements — read positions reactively so edges
                // follow dragged nodes in real time.
                let edge_svgs: Vec<_> = edges
                    .iter()
                    .map(|edge| {
                        let src = edge.source_id.0;
                        let tgt = edge.target_id.0;
                        let lbl = edge.label.clone();
                        view! {
                            <g>
                                <line
                                    x1=move || {
                                        format!(
                                            "{:.1}",
                                            positions.get().get(&src).map(|p| p.0).unwrap_or(0.0),
                                        )
                                    }
                                    y1=move || {
                                        format!(
                                            "{:.1}",
                                            positions.get().get(&src).map(|p| p.1).unwrap_or(0.0),
                                        )
                                    }
                                    x2=move || {
                                        format!(
                                            "{:.1}",
                                            positions.get().get(&tgt).map(|p| p.0).unwrap_or(0.0),
                                        )
                                    }
                                    y2=move || {
                                        format!(
                                            "{:.1}",
                                            positions.get().get(&tgt).map(|p| p.1).unwrap_or(0.0),
                                        )
                                    }
                                    stroke="#94a3b8"
                                    attr:stroke-width="1.5"
                                    attr:stroke-opacity="0.6"
                                />
                                {lbl.map(|l| {
                                    view! {
                                        <text
                                            x=move || {
                                                format!(
                                                    "{:.1}",
                                                    {
                                                        let pos = positions.get();
                                                        let x1 = pos.get(&src).map(|p| p.0).unwrap_or(0.0);
                                                        let x2 = pos.get(&tgt).map(|p| p.0).unwrap_or(0.0);
                                                        (x1 + x2) / 2.0
                                                    },
                                                )
                                            }
                                            y=move || {
                                                format!(
                                                    "{:.1}",
                                                    {
                                                        let pos = positions.get();
                                                        let y1 = pos.get(&src).map(|p| p.1).unwrap_or(0.0);
                                                        let y2 = pos.get(&tgt).map(|p| p.1).unwrap_or(0.0);
                                                        (y1 + y2) / 2.0
                                                    },
                                                )
                                            }
                                            attr:text-anchor="middle"
                                            attr:font-size="10"
                                            fill="#94a3b8"
                                            dy="-4"
                                        >
                                            {l}
                                        </text>
                                    }
                                })}
                            </g>
                        }
                        .into_any()
                    })
                    .collect();

                view! {
                    <svg
                        class="w-full h-full"
                        style=move || {
                            if drag_node.get().is_some() || panning.get() {
                                "cursor: grabbing;"
                            } else {
                                "cursor: default;"
                            }
                        }
                        on:mousedown=move |ev: MouseEvent| {
                            // Only start panning when clicking the canvas background.
                            panning.set(true);
                            last_mx.set(ev.client_x() as f64);
                            last_my.set(ev.client_y() as f64);
                        }
                        on:mousemove=move |ev: MouseEvent| {
                            if let Some(nid) = drag_node.get_untracked() {
                                // Dragging a node.
                                ev.prevent_default();
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
                                // Panning the canvas.
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
                            // Cancel drag/pan when cursor leaves SVG.
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
                        </g>
                    </svg>
                }
                .into_any()
            }}
        </div>
    }
}
