mod application;
mod config;

use application::run;
use futures::{
    executor::{LocalPool, LocalSpawner},
    task::LocalSpawnExt,
};
use std::cell::RefCell;
use env_logger::Env;
use std::sync::Arc;

fn main() -> anyhow::Result<()> {
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();
    let executor = ArcSingleThreadedExecutor(Arc::new(SingleThreadedExecutor::new()));
    any_spawner::Executor::init_local_custom_executor(executor.clone())
        .expect("Futures executor failed to init");
    let result = run();
    // executor.yeet(); // Weird that dropping would make it unhappy
    result
}

/// Wasm is single threaded,
/// and we want to be as close to the actual environment as possible
struct SingleThreadedExecutor {
    local_pool: RefCell<LocalPool>,
    local_spawner: LocalSpawner,
}

impl SingleThreadedExecutor {
    fn new() -> Self {
        let local_pool = LocalPool::new();
        let local_spawner = local_pool.spawner();
        Self {
            local_pool: RefCell::new(local_pool),
            local_spawner,
        }
    }
}

impl any_spawner::CustomExecutor for SingleThreadedExecutor {
    fn spawn(&self, fut: any_spawner::PinnedFuture<()>) {
        let _ = self.local_spawner.spawn_local(fut);
    }

    fn spawn_local(&self, fut: any_spawner::PinnedLocalFuture<()>) {
        let _ = self.local_spawner.spawn_local(fut);
    }

    fn poll_local(&self) {
        self.local_pool.borrow_mut().run_until_stalled();
    }
}

#[derive(Clone)]
struct ArcSingleThreadedExecutor(Arc<SingleThreadedExecutor>);
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