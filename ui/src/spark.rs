//! Spark-on-complete — the app's signature micro-interaction (design phase 3).
//!
//! Completing a task strikes a brief burst of ember particles off the
//! checkbox (styles in `input.css`, `.spark-particle`). Design decisions:
//!
//! - Particles live in a `position:fixed` wrapper appended to `<body>` at the
//!   checkbox's viewport coordinates — row markup and e2e selectors are
//!   untouched, and a mid-animation row refetch can't kill the burst.
//! - Removal is timer-based (not `animationend`) so a dropped event can never
//!   leak DOM nodes.
//! - `prefers-reduced-motion` skips the burst entirely.
//! - Fire-and-forget: every fallible step is swallowed — a missing spark must
//!   never break the status toggle it decorates.

use wasm_bindgen::JsCast;

/// Particles per burst.
const COUNT: u32 = 7;
/// Approximate spread radius in px.
const RADIUS: f64 = 18.0;
/// Golden angle in radians — deterministic but visually irregular spread
/// (no RNG dependency, and stable for tests).
const GOLDEN_ANGLE: f64 = 2.399_963;

fn reduced_motion() -> bool {
    web_sys::window()
        .and_then(|w| w.match_media("(prefers-reduced-motion: reduce)").ok())
        .flatten()
        .is_some_and(|m| m.matches())
}

/// Strike a spark burst centred on the event's current target (the checkbox
/// button). Call only on the transition *to* done — un-completing a task is
/// not a celebration.
pub fn strike_spark(ev: &web_sys::MouseEvent) {
    if reduced_motion() {
        return;
    }
    let Some(target) = ev.current_target() else {
        return;
    };
    let Ok(el) = target.dyn_into::<web_sys::Element>() else {
        return;
    };
    let rect = el.get_bounding_client_rect();
    let cx = rect.left() + rect.width() / 2.0;
    let cy = rect.top() + rect.height() / 2.0;
    let Some(doc) = web_sys::window().and_then(|w| w.document()) else {
        return;
    };
    let Some(body) = doc.body() else {
        return;
    };
    let Ok(wrap) = doc.create_element("span") else {
        return;
    };
    let _ = wrap.set_attribute("aria-hidden", "true");
    let _ = wrap.set_attribute(
        "style",
        &format!(
            "position:fixed;left:{cx}px;top:{cy}px;width:0;height:0;\
             z-index:60;pointer-events:none;"
        ),
    );
    for i in 0..COUNT {
        let ang = f64::from(i) * GOLDEN_ANGLE;
        // Vary distance a step per particle so the ring reads as a burst.
        let dist = RADIUS * (0.7 + 0.3 * f64::from(i * 37 % 10) / 10.0);
        let (dx, dy) = (ang.cos() * dist, ang.sin() * dist);
        if let Ok(p) = doc.create_element("span") {
            let _ = p.set_attribute("class", "spark-particle");
            let _ = p.set_attribute(
                "style",
                &format!("--spark-dx:{dx:.1}px;--spark-dy:{dy:.1}px;"),
            );
            let _ = wrap.append_child(&p);
        }
    }
    let _ = body.append_child(&wrap);
    wasm_bindgen_futures::spawn_local(async move {
        // Outlives the 0.5 s animation.
        gloo_timers::future::TimeoutFuture::new(600).await;
        wrap.remove();
    });
}
