use std::cell::RefCell;

use futures::{
    executor::{LocalPool, LocalSpawner},
    task::LocalSpawnExt,
};

/// Our renderer depends on being able to *force run* the effects.
pub struct LocalExecutor {
    pub local_pool: RefCell<LocalPool>,
    pub local_spawner: LocalSpawner,
}

impl LocalExecutor {
    pub fn new() -> Self {
        let local_pool = LocalPool::new();
        let local_spawner = local_pool.spawner();
        Self {
            local_pool: RefCell::new(local_pool),
            local_spawner,
        }
    }
}

impl any_spawner::CustomExecutor for LocalExecutor {
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
