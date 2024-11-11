mod application;
mod config;

use application::run;

fn main() -> anyhow::Result<()> {
    env_logger::init();
    any_spawner::Executor::init_futures_executor().expect("Futures executor failed to init");
    run()
}
