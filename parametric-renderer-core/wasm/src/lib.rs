mod application;
pub mod wasm_abi;

use tracing_subscriber::fmt::{format::JsonFields, time::UtcTime};
use tracing_subscriber::prelude::*;
use tracing_web::{performance_layer, MakeWebConsoleWriter};
use wasm_bindgen::prelude::*;

#[wasm_bindgen(start)]
pub fn run() {
    console_error_panic_hook::set_once();
    // https://crates.io/crates/tracing-web
    let fmt_layer = tracing_subscriber::fmt::layer()
        .with_ansi(false)
        .with_timer(UtcTime::rfc_3339())
        .with_writer(MakeWebConsoleWriter::new());
    let perf_layer = performance_layer().with_details_from_fields(JsonFields::default());

    tracing_subscriber::registry()
        .with(GlobalFilter)
        .with(fmt_layer)
        .with(perf_layer)
        .init();
}

struct GlobalFilter;
impl<S: tracing::Subscriber> tracing_subscriber::layer::Layer<S> for GlobalFilter {
    fn register_callsite(
        &self,
        metadata: &'static tracing::Metadata<'static>,
    ) -> tracing::subscriber::Interest {
        if metadata.target() == "pinky_swear" {
            tracing::subscriber::Interest::never()
        } else {
            tracing::subscriber::Interest::always()
        }
    }
}
