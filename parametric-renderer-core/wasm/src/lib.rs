mod application;
pub mod wasm_abi;

use log::Level;
use renderer_core::local_executor::LocalExecutor;
use wasm_bindgen::prelude::*;
#[wasm_bindgen(start)]
pub fn run() {
    console_error_panic_hook::set_once();
    console_log::init_with_level(Level::Info).unwrap();
    any_spawner::Executor::init_local_custom_executor(SingleThreadedExecutor::new())
        .expect("Futures executor for reactive graph failed to init");
}

struct SingleThreadedExecutor {
    inner: LocalExecutor,
}

impl SingleThreadedExecutor {
    fn new() -> Self {
        Self {
            inner: LocalExecutor::new(),
        }
    }
}

impl any_spawner::CustomExecutor for SingleThreadedExecutor {
    fn spawn(&self, fut: any_spawner::PinnedFuture<()>) {
        wasm_bindgen_futures::spawn_local(fut);
    }

    fn spawn_local(&self, fut: any_spawner::PinnedLocalFuture<()>) {
        self.inner.spawn_local(fut);
    }

    fn poll_local(&self) {
        self.inner.poll_local();
    }
}
