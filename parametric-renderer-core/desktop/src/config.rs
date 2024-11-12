use core::fmt;
use nanoserde::{DeJson, DeJsonErr, SerJson};
use std::path::Path;

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

// See also https://www.reddit.com/r/rust/comments/gj8inf/comment/fqlmknt/
#[derive(Debug)]
pub enum LoadConfigError {
    Io(std::io::Error),
    Parse(DeJsonErr),
}

impl fmt::Display for LoadConfigError {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        match self {
            LoadConfigError::Io(v) => write!(fmt, "IO error: {v}"),
            LoadConfigError::Parse(v) => write!(fmt, "Failed to parse config file: {v}"),
        }
    }
}
impl From<std::io::Error> for LoadConfigError {
    fn from(source: std::io::Error) -> Self {
        LoadConfigError::Io(source)
    }
}
impl From<DeJsonErr> for LoadConfigError {
    fn from(source: DeJsonErr) -> Self {
        LoadConfigError::Parse(source)
    }
}
