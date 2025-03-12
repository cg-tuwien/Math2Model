mod application;
mod config;

use application::run;
use env_logger::Env;
use futures::executor::LocalPool;
use renderer_core::local_executor::LocalExecutor;
use std::sync::Arc;

fn main() -> anyhow::Result<()> {
    env_logger::Builder::from_env(Env::default().default_filter_or("info"))
        .filter_module("wgpu_hal::vulkan::instance", log::LevelFilter::Warn)
        .filter_module("naga::back::spv::writer", log::LevelFilter::Warn)
        .init();
    let executor = ArcSingleThreadedExecutor(Arc::new(LocalExecutor::new()));
    any_spawner::Executor::init_local_custom_executor(executor.clone())
        .expect("Futures executor failed to init");
    let result = run();
    executor.yeet(); // And manually drop the scheduled tasks
    result
}

/// Wasm is single threaded,
/// and we want to be as close to the actual environment as possible
#[derive(Clone)]
struct ArcSingleThreadedExecutor(Arc<LocalExecutor>);
impl any_spawner::CustomExecutor for ArcSingleThreadedExecutor {
    fn spawn(&self, fut: any_spawner::PinnedFuture<()>) {
        self.0.spawn(fut);
    }

    fn spawn_local(&self, fut: any_spawner::PinnedLocalFuture<()>) {
        self.0.spawn_local(fut);
    }

    fn poll_local(&self) {
        self.0.poll_local();
    }
}

impl ArcSingleThreadedExecutor {
    fn yeet(&self) {
        *self.0.local_pool.borrow_mut() = LocalPool::new();
    }
}
