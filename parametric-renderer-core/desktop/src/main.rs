mod application;
mod camera;
mod config;
mod mesh;
mod shaders;
mod texture;

use application::run;
use config::ConfigFile;

fn main() -> anyhow::Result<()> {
    // Set up tracing https://tokio.rs/tokio/topics/tracing
    let subscriber = tracing_subscriber::FmtSubscriber::new();
    tracing::subscriber::set_global_default(subscriber)?;

    let config_file = ConfigFile::from_file("config.json").unwrap_or_default();
    pollster::block_on(run())
}
