//! Image manifest structs.
//!
//! See [Imag Manifest V2, Schema 2](https://docs.docker.com/registry/spec/manifest-v2-2/)
//! for more details.

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
