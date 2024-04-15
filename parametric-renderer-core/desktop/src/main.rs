mod application;
mod camera;
mod quad;
mod shaders;
mod texture;

use application::run;

fn main() -> anyhow::Result<()> {
    // Set up tracing https://tokio.rs/tokio/topics/tracing
    let subscriber = tracing_subscriber::FmtSubscriber::new();
    tracing::subscriber::set_global_default(subscriber)?;

    pollster::block_on(run())
}
