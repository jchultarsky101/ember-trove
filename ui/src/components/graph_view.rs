/// Visual knowledge graph — force-directed SVG node-link diagram.
///
/// Fetches all nodes and edges, runs a Fruchterman-Reingold layout in WASM,
/// and renders an interactive SVG with pan (drag) and zoom (wheel).
use std::collections::HashMap;

use leptos::prelude::*;
use uuid::Uuid;
use wasm_bindgen_futures::spawn_local;
use web_sys::{MouseEvent, WheelEvent};

use common::{edge::Edge, id::NodeId, node::Node};

use crate::{
    api::{fetch_all_edges, fetch_nodes},
    app::View,
};

// SVG coordinate-space canvas size
const W: f64 = 1000.0;
const H: f64 = 700.0;
const MARGIN: f64 = 80.0;

/// Fruchterman-Reingold spring layout (200 iterations, O(n²) repulsion).
fn force_layout(node_ids: &[Uuid], edge_pairs: &[(Uuid, Uuid)]) -> Vec<(Uuid, f64, f64)> {
    let n = node_ids.len();
    if n == 0 {
        return vec![];
    }
    if n == 1 {
        return vec![(node_ids[0], W / 2.0, H / 2.0)];
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
        .map(|(i, id)| (*id, px[i], py[i]))
        .collect()
}

#[component]
pub fn GraphView() -> impl IntoView {
    let current_view = use_context::<RwSignal<View>>().expect("View signal must be provided");

    let loading = RwSignal::new(true);
    let error_msg = RwSignal::new(Option::<String>::None);
    let nodes_sig: RwSignal<Vec<Node>> = RwSignal::new(vec![]);
    let edges_sig: RwSignal<Vec<Edge>> = RwSignal::new(vec![]);
    let positions: RwSignal<Vec<(Uuid, f64, f64)>> = RwSignal::new(vec![]);

    // Pan/zoom state
    let pan_x = RwSignal::new(0.0_f64);
    let pan_y = RwSignal::new(0.0_f64);
    let zoom = RwSignal::new(1.0_f64);
    let panning = RwSignal::new(false);
    let last_mx = RwSignal::new(0.0_f64);
    let last_my = RwSignal::new(0.0_f64);

    // Fetch nodes + edges once on mount, then compute layout
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
                    let layout = force_layout(&node_ids, &edge_pairs);
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
                let pos_map: HashMap<Uuid, (f64, f64)> = positions
                    .get()
                    .into_iter()
                    .map(|(id, x, y)| (id, (x, y)))
                    .collect();

                // Build static edge SVG elements
                let edge_svgs: Vec<_> = edges
                    .iter()
                    .filter_map(|edge| {
                        let (x1, y1) = pos_map.get(&edge.source_id.0).copied()?;
                        let (x2, y2) = pos_map.get(&edge.target_id.0).copied()?;
                        let mid_x = (x1 + x2) / 2.0;
                        let mid_y = (y1 + y2) / 2.0;
                        let lbl = edge.label.clone();
                        Some(
                            view! {
                                <g>
                                    <line
                                        x1=format!("{x1:.1}")
                                        y1=format!("{y1:.1}")
                                        x2=format!("{x2:.1}")
                                        y2=format!("{y2:.1}")
                                        stroke="#94a3b8"
                                        attr:stroke-width="1.5"
                                        attr:stroke-opacity="0.6"
                                    />
                                    {lbl.map(|l| {
                                        view! {
                                            <text
                                                x=format!("{mid_x:.1}")
                                                y=format!("{mid_y:.1}")
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
                            .into_any(),
                        )
                    })
                    .collect();

                // Build static node SVG elements
                let node_svgs: Vec<_> = nodes
                    .iter()
                    .map(|node| {
                        let (nx, ny) = pos_map
                            .get(&node.id.0)
                            .copied()
                            .unwrap_or((W / 2.0, H / 2.0));
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
                                style="cursor: pointer;"
                                on:click=move |ev: MouseEvent| {
                                    ev.stop_propagation();
                                    current_view.set(View::NodeDetail(node_id));
                                }
                                on:mousedown=|ev: MouseEvent| ev.stop_propagation()
                            >
                                <circle
                                    cx=format!("{nx:.1}")
                                    cy=format!("{ny:.1}")
                                    r="28"
                                    fill="#f59e0b"
                                    attr:fill-opacity="0.15"
                                    stroke="#f59e0b"
                                    attr:stroke-width="2"
                                />
                                <text
                                    x=format!("{nx:.1}")
                                    y=format!("{:.1}", ny + 5.0)
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

                view! {
                    <svg
                        class="w-full h-full"
                        style=move || {
                            if panning.get() { "cursor: grabbing;" } else { "cursor: grab;" }
                        }
                        on:mousedown=move |ev: MouseEvent| {
                            panning.set(true);
                            last_mx.set(ev.client_x() as f64);
                            last_my.set(ev.client_y() as f64);
                        }
                        on:mousemove=move |ev: MouseEvent| {
                            if panning.get_untracked() {
                                let mx = ev.client_x() as f64;
                                let my = ev.client_y() as f64;
                                pan_x.update(|p| *p += mx - last_mx.get_untracked());
                                pan_y.update(|p| *p += my - last_my.get_untracked());
                                last_mx.set(mx);
                                last_my.set(my);
                            }
                        }
                        on:mouseup=move |_: MouseEvent| {
                            panning.set(false);
                        }
                        on:mouseleave=move |_: MouseEvent| {
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
