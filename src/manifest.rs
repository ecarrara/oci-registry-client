//! Image manifest structs.
//!
//! See [Imag Manifest V2, Schema 2](https://docs.docker.com/registry/spec/manifest-v2-2/)
//! for more details.

use std::collections::HashMap;

/// The [`ManifestList`] is the "fat manifest" which points
/// to specific image manifests for one or more platforms.
#[derive(serde::Serialize, serde::Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ManifestList {
    pub schema_version: i32,
    pub media_type: String,
    pub manifests: Vec<ManifestItem>,
}

/// [`ManifestItem`] for a specific platform.
#[derive(serde::Serialize, serde::Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ManifestItem {
    pub media_type: String,
    pub size: usize,
    pub digest: String,
    pub platform: Platform,
}

/// The [`Platform`] describes the platform which the image in the
/// manifest runs on.
#[derive(serde::Serialize, serde::Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Platform {
    pub architecture: String,
    pub os: String,
    pub os_version: Option<String>,
    pub os_features: Option<Vec<String>>,
    pub variant: Option<String>,
    pub features: Option<Vec<String>>,
}

/// The [`Manifest`] provides a configuration and a set of layers for a
/// container image.
#[derive(serde::Serialize, serde::Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Manifest {
    pub schema_version: i32,
    pub media_type: String,
    pub config: ManifestConfig,
    pub layers: Vec<Layer>,
}

/// The [`ManifestConfig`] references a configuration object for a container.
#[derive(serde::Serialize, serde::Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ManifestConfig {
    pub media_type: String,
    pub size: usize,
    pub digest: String,
}

/// The [`Layer`] references a [`crate::blob::Blob`] by digest.
#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Layer {
    pub media_type: String,
    pub size: usize,
    pub digest: String,
}

/// Image configuration.
///
/// Describes some basic information about the image such as date
/// created, author, as well as execution/runtime configuration like
/// entrypoint, default arguments, networking and volumes.
#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Image {
    pub architecture: String,
    pub os: String,
    pub created: Option<String>,
    pub author: Option<String>,
    pub config: Option<ImageConfig>,
    pub rootfs: RootFS,
    pub history: Option<Vec<LayerHistory>>,
}

/// Image execution default parameters.
#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct ImageConfig {
    pub user: Option<String>,
    pub exposed_ports: Option<HashMap<String, serde_json::Value>>,
    pub env: Option<Vec<String>>,
    pub entrypoint: Option<Vec<String>>,
    pub cmd: Option<Vec<String>>,
    pub volumes: Option<HashMap<String, serde_json::Value>>,
    pub labels: Option<HashMap<String, String>>,
    pub stop_signal: Option<String>,
}

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
pub struct RootFS {
    pub r#type: String,
    diff_ids: Vec<String>,
}

/// Describe the history of a layer.
#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
pub struct LayerHistory {
    pub created: Option<String>,
    pub author: Option<String>,
    pub created_by: Option<String>,
    pub comment: Option<String>,
    pub empty_layer: Option<bool>,
}
