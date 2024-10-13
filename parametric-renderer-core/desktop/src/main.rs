mod application;
mod config;

use application::run;

fn main() -> anyhow::Result<()> {
    // Set up tracing https://tokio.rs/tokio/topics/tracing
    let subscriber = tracing_subscriber::FmtSubscriber::new();
    tracing::subscriber::set_global_default(subscriber)?;

    any_spawner::Executor::init_futures_executor().expect("Futures executor failed to init");

    run()
}
