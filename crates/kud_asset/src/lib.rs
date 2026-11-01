//! Asset identity, manifests, and runtime registry.
//!
//! Assets are addressed by stable IDs, not by scene-graph object names. File
//! paths are import-time metadata and may change without breaking save data.

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Stable runtime identifier for an imported asset.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct AssetId(pub u64);

/// Broad asset classification used by the import pipeline.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AssetKind {
    /// A mesh or skeletal mesh.
    Mesh,
    /// A material definition.
    Material,
    /// A texture.
    Texture,
    /// An animation clip.
    Animation,
    /// An audio clip.
    Audio,
    /// A scene or prefab.
    Scene,
    /// A behavior/data table.
    Data,
}

/// Import-time metadata for one asset.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetRecord {
    /// Stable runtime ID.
    pub id: AssetId,
    /// Asset classification.
    pub kind: AssetKind,
    /// Source path relative to the project asset root.
    pub source_path: PathBuf,
    /// Optional human-readable label.
    pub label: Option<String>,
}

/// A serializable collection of asset records.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AssetManifest {
    /// Manifest schema version.
    pub version: u32,
    /// Imported assets.
    pub assets: Vec<AssetRecord>,
}

/// Errors produced by the asset layer.
#[derive(Debug, Error)]
pub enum AssetError {
    /// The manifest could not be read or parsed.
    #[error("asset manifest I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// The manifest JSON is invalid.
    #[error("asset manifest JSON error: {0}")]
    Json(#[from] serde_json::Error),

    /// Two records use the same stable ID.
    #[error("duplicate asset id: {0:?}")]
    DuplicateId(AssetId),

    /// A referenced asset does not exist.
    #[error("unknown asset id: {0:?}")]
    UnknownId(AssetId),
}

/// In-memory index from stable asset IDs to records.
#[derive(Debug, Default)]
pub struct AssetRegistry {
    records: HashMap<AssetId, AssetRecord>,
}

impl AssetRegistry {
    /// Builds a registry from a manifest, rejecting duplicate IDs.
    pub fn from_manifest(manifest: AssetManifest) -> Result<Self, AssetError> {
        let mut records = HashMap::with_capacity(manifest.assets.len());
        for record in manifest.assets {
            let id = record.id;
            if records.insert(id, record).is_some() {
                return Err(AssetError::DuplicateId(id));
            }
        }
        Ok(Self { records })
    }

    /// Loads a JSON manifest from disk.
    pub fn load_json(path: impl AsRef<Path>) -> Result<Self, AssetError> {
        let text = std::fs::read_to_string(path)?;
        let manifest: AssetManifest = serde_json::from_str(&text)?;
        Self::from_manifest(manifest)
    }

    /// Returns a record by stable ID.
    pub fn get(&self, id: AssetId) -> Result<&AssetRecord, AssetError> {
        self.records.get(&id).ok_or(AssetError::UnknownId(id))
    }

    /// Returns the number of registered assets.
    pub fn len(&self) -> usize {
        self.records.len()
    }

    /// Returns true when no assets are registered.
    pub fn is_empty(&self) -> bool {
        self.records.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_duplicate_ids() {
        let record = AssetRecord {
            id: AssetId(1),
            kind: AssetKind::Data,
            source_path: PathBuf::from("data/test.json"),
            label: None,
        };
        let manifest = AssetManifest {
            version: 1,
            assets: vec![record.clone(), record],
        };
        assert!(AssetRegistry::from_manifest(manifest).is_err());
    }
}
