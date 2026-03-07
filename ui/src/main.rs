mod api;
mod app;
mod auth;
mod components;
mod error;

use tracing_subscriber::{fmt, prelude::*, EnvFilter};
use tracing_web::MakeConsoleWriter;

use app::App;

fn init_tracing() {
    let filter = option_env!("RUST_LOG")
        .and_then(|s| s.parse::<EnvFilter>().ok())
        .unwrap_or_else(|| EnvFilter::new("warn"));

    let fmt_layer = fmt::layer()
        .with_ansi(false)
        .without_time()
        .with_writer(MakeConsoleWriter);

    tracing_subscriber::registry()
        .with(filter)
        .with(fmt_layer)
        .init();
}

fn main() {
    init_tracing();
    leptos::mount::mount_to_body(App);
}
