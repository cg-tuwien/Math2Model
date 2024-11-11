mod application;
pub mod wasm_abi;

use log::Level;
use wasm_bindgen::prelude::*;

#[wasm_bindgen(start)]
pub fn run() {
    console_error_panic_hook::set_once();
    console_log::init_with_level(Level::Info).unwrap();
    any_spawner::Executor::init_wasm_bindgen()
        .expect("Futures executor for reactive graph failed to init");
}
