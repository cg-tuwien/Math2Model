mod application;

use application::EngineSettings;
use tracing_subscriber::fmt::{format::Pretty, time::UtcTime};
use tracing_subscriber::prelude::*;
use tracing_web::{performance_layer, MakeWebConsoleWriter};
use wasm_bindgen::prelude::*;
use web_sys::{js_sys, HtmlCanvasElement};

#[wasm_bindgen(start)]
pub fn run() {
    console_error_panic_hook::set_once();
    // https://crates.io/crates/tracing-web
    let fmt_layer = tracing_subscriber::fmt::layer()
        .with_ansi(false)
        .with_timer(UtcTime::rfc_3339())
        .with_writer(MakeWebConsoleWriter::new());
    let perf_layer = performance_layer().with_details_from_fields(Pretty::default());

    tracing_subscriber::registry()
        .with(fmt_layer)
        .with(perf_layer)
        .init();
}

#[wasm_bindgen]
pub fn init_engine(
    canvas: HtmlCanvasElement,
    on_compile_error: js_sys::Function,
) -> Result<(), JsValue> {
    wasm_bindgen_futures::spawn_local(async move {
        match application::run(EngineSettings {
            canvas,
            on_compile_error,
        })
        .await
        {
            Ok(_) => (),
            Err(e) => {
                tracing::error!("Error running application: {:?}", e);
            }
        }
    });
    Ok(())
}
