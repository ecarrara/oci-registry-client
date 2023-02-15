//! A async client for [OCI compliant image
//! registries](http://github.com/opencontainers/distribution-spec/blob/master/spec.md/)
//! and [Docker Registry HTTP V2 protocol](https://docs.docker.com/registry/spec/api/).
//!
//! # Usage
//!
//! The [`DockerRegistryClientV2`] provides functions to query Registry API and download blobs.
//!
//! ```no_run
//! use std::{path::Path, fs::File, io::Write};
//! use oci_registry_client::DockerRegistryClientV2;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let mut client = DockerRegistryClientV2::new(
//!     "registry.docker.io",
//!     "https://registry-1.docker.io",
//!     "https://auth.docker.io/token"
//! );
//! let token = client.auth("repository", "library/ubuntu", "latest").await?;
//! client.set_auth_token(Some(token));
//!
//! let manifest = client.manifest("library/ubuntu", "latest").await?;
//! println!("{:?}", manifest);
//!
//! for layer in &manifest.layers {
//!    let mut out_file = File::create(Path::new("/tmp/").join(&layer.digest.to_string()))?;
//!    let mut blob = client.blob("library/ubuntu", &layer.digest).await?;
//!
//!    while let Some(chunk) = blob.chunk().await? {
//!        out_file.write_all(&chunk)?;
//!    }
//! }
//!
//! # Ok(())
//! # }
//! ```

pub mod blob;
pub mod errors;
pub mod manifest;

use blob::Blob;
use errors::{ErrorList, ErrorResponse};
use manifest::{Digest, Image, Manifest, ManifestList};
use reqwest::{Method, StatusCode};

static USER_AGENT: &str = concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION"));

/// Client to fetch image manifests and download blobs.
///
/// DockerRegistryClientV2 provides functions to fetch manifests and download
/// blobs from a OCI Image Registry (or a Docker Registry API V2).
#[derive(Clone, Debug)]
pub struct DockerRegistryClientV2 {
    service: String,
    api_url: String,
    oauth_url: String,
    auth_token: Option<AuthToken>,
    client: reqwest::Client,
}

#[derive(serde::Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Version {}

const MEDIA_TYPE_JSON: &str = "applicatin/json";
const MEDIA_TYPE_MANIFEST_LIST_V2: &str =
    "application/vnd.docker.distribution.manifest.list.v2+json";
const MEDIA_TYPE_MANIFEST_V2: &str = "application/vnd.docker.distribution.manifest.v2+json";
const MEDIA_TYPE_IMAGE_CONFIG: &str = "application/vnd.docker.container.image.v1+json";

impl DockerRegistryClientV2 {
    /// Returns a new `DockerRegistryClientV2`.
    ///
    /// # Arguments
    ///
    /// * `service` - Name of a Image Registry Service (example: registry.docker.io)
    /// * `api_url` - Service HTTPS address (example: https://registry-1.docker.io)
    /// * `auth_url` - Address to get a OAuth 2.0 token for this service.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use oci_registry_client::DockerRegistryClientV2;
    /// let mut client = DockerRegistryClientV2::new(
    ///     "registry.docker.io",
    ///     "https://registry-1.docker.io",
    ///     "https://auth.docker.io/token"
    /// );
    /// ```
    pub fn new<T: Into<String>>(service: T, api_url: T, oauth_url: T) -> Self {
        let client = reqwest::Client::builder()
            .user_agent(USER_AGENT)
            .build()
            .unwrap();

        Self {
            service: service.into(),
            api_url: api_url.into(),
            oauth_url: oauth_url.into(),
            auth_token: None,
            client,
        }
    }

    /// Set access token to authenticate subsequent requests.
    pub fn set_auth_token(&mut self, token: Option<AuthToken>) {
        self.auth_token = token;
    }

    /// Fetch a access token from `auth_url` for this `service`.
    ///
    /// # Arguments
    ///
    /// * `type` - Scope type (example: "repository").
    /// * `name` - Name of resource (example: "library/ubuntu").
    /// * `action` - List of actions separated by comma (example: "pull").
    pub async fn auth(
        &self,
        r#type: &str,
        name: &str,
        action: &str,
    ) -> Result<AuthToken, ErrorResponse> {
        let response = self
            .client
            .get(&self.oauth_url)
            .query(&[
                ("service", self.service.clone()),
                ("scope", format!("{}:{}:{}", r#type, name, action)),
            ])
            .send()
            .await?;

        match response.status() {
            StatusCode::OK => Ok(response.json::<AuthToken>().await?),
            _ => Err(ErrorResponse::APIError(response.json::<ErrorList>().await?)),
        }
    }

    /// Get API version.
    pub async fn version(&self) -> Result<Version, ErrorResponse> {
        let url = format!("{}/v2", self.api_url);
        self.request(Method::GET, &url, MEDIA_TYPE_JSON).await
    }

    /// List manifests from given image and reference.
    pub async fn list_manifests(
        &self,
        image: &str,
        reference: &str,
    ) -> Result<ManifestList, ErrorResponse> {
        let url = format!("{}/v2/{}/manifests/{}", &self.api_url, image, reference);
        self.request(Method::GET, &url, MEDIA_TYPE_MANIFEST_LIST_V2)
            .await
    }

    /// Get the image manifest.
    pub async fn manifest(&self, image: &str, reference: &str) -> Result<Manifest, ErrorResponse> {
        let url = format!("{}/v2/{}/manifests/{}", &self.api_url, image, reference);
        self.request(Method::GET, &url, MEDIA_TYPE_MANIFEST_V2)
            .await
    }

    /// Get the container config.
    pub async fn config(&self, image: &str, reference: &Digest) -> Result<Image, ErrorResponse> {
        let url = format!("{}/v2/{}/blobs/{}", &self.api_url, image, reference);
        self.request(Method::GET, &url, MEDIA_TYPE_IMAGE_CONFIG)
            .await
    }

    /// Retrieve the blob from the registry identified by `digest`.
    pub async fn blob(&self, image: &str, digest: &Digest) -> Result<Blob, ErrorResponse> {
        let url = format!("{}/v2/{}/blobs/{}", &self.api_url, image, digest);
        let mut request = self.client.get(&url);
        if let Some(token) = self.auth_token.clone() {
            request = request.bearer_auth(token.access_token);
        }

        let response = request.send().await?;

        match response.status() {
            StatusCode::OK => Ok(Blob::from(response)),
            _ => Err(ErrorResponse::APIError(response.json::<ErrorList>().await?)),
        }
    }

    async fn request<T: serde::de::DeserializeOwned>(
        &self,
        method: Method,
        url: &str,
        accept: &str,
    ) -> Result<T, ErrorResponse> {
        let mut request = self
            .client
            .request(method, url)
            .header(reqwest::header::ACCEPT, accept);

        if let Some(token) = self.auth_token.clone() {
            request = request.bearer_auth(token.access_token);
        }

        let response = request.send().await?;

        match response.status() {
            StatusCode::OK => Ok(response.json::<T>().await?),
            _ => Err(ErrorResponse::APIError(response.json::<ErrorList>().await?)),
        }
    }
}

/// OAuth 2.0 token.
#[allow(dead_code)]
#[derive(serde::Deserialize, Clone, Debug)]
pub struct AuthToken {
    access_token: String,
    expires_in: i32,
    issued_at: String,
}
