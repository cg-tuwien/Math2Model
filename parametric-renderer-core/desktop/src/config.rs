use std::path::Path;

use nanoserde::{DeJson, DeJsonErr, SerJson};
use thiserror::Error;

#[derive(DeJson, SerJson, Debug, Clone)]
pub struct ConfigFile {}

impl ConfigFile {
    pub fn new() -> Self {
        Self {}
    }

    pub fn from_file(path: impl AsRef<Path>) -> Result<Self, LoadConfigError> {
        let content = std::fs::read_to_string(path)?;
        Ok(DeJson::deserialize_json(&content)?)
    }

    pub fn save_to_file(&self, path: impl AsRef<Path>) -> Result<(), std::io::Error> {
        let content = SerJson::serialize_json(self);
        std::fs::write(path, content)
    }
}

impl Default for ConfigFile {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(DeJson, SerJson, Debug, Clone)]
pub struct CacheFile {
    pub camera: Option<CachedCamera>,
}

#[derive(DeJson, SerJson, Debug, Clone)]
pub struct CachedCamera {
    pub position: [f32; 3],
    pub orientation: [f32; 4],
    pub distance_to_center: f32,
    pub chosen: CachedChosenController,
}
#[derive(DeJson, SerJson, Debug, Clone)]
pub enum CachedChosenController {
    Orbitcam,
    Freecam,
}

impl CacheFile {
    pub fn new() -> Self {
        Self { camera: None }
    }

    pub fn from_file(path: impl AsRef<Path>) -> Result<Self, LoadConfigError> {
        let content = std::fs::read_to_string(path)?;
        Ok(DeJson::deserialize_json(&content)?)
    }

    pub fn save_to_file(&self, path: impl AsRef<Path>) -> Result<(), std::io::Error> {
        let content = SerJson::serialize_json(self);
        std::fs::write(path, content)
    }
}

impl Default for CacheFile {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Error)]
pub enum LoadConfigError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Failed to parse config file: {0}")]
    Parse(#[from] DeJsonErr),
}
