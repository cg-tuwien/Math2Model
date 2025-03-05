mod application;
mod config;

use application::run;
use futures::{
    executor::{LocalPool, LocalSpawner},
    task::LocalSpawnExt,
};
use std::cell::RefCell;

fn main() -> anyhow::Result<()> {
    env_logger::init();
    let executor = SingleThreadedExecutor::new();
    any_spawner::Executor::init_local_custom_executor(executor)
        .expect("Futures executor failed to init");
    let result = run();
    // local_pool.borrow_mut().replace(LocalPool::new());
    any_spawner::Executor::poll_local();
    result
}

/// Wasm is single threaded,
/// and we want to be as close to the actual environment as possible    
/// And we want to manually dispose of the local pool,
/// instead of relying on any_spawner's eventual disposing.
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
