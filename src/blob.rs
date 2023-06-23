//! A "blob" representation.
//!
//! This module provides a utility struct called [`Blob`].
//!
//! You can iterate over a blob chunks to download it contents:
//!
//! ```ignore
//! let response = reqwest::get("...");
//! let mut blob = Blob::from(response)
//!
//! while let Some(chunk) = blob.chunk().await? {
//!     out_file.write_all(&chunk)?;
//! }
//! ```

use crate::errors::ErrorResponse;
use crate::manifest::Digest;
use bytes::Bytes;
use reqwest;
#[cfg(feature = "sha256")]
use sha2::{Digest as Sha256Digest, Sha256};

/// Blob represents a downloaded content in a Image Registry.
pub struct Blob {
    response: reqwest::Response,
    len: Option<usize>,
    content_type: Option<String>,
    #[cfg(feature = "sha256")]
    hasher: Sha256,
}

impl Blob {
    /// Returns the total length of this blob.
    #[allow(clippy::len_without_is_empty)]
    pub fn len(&self) -> Option<usize> {
        self.len
    }

    /// Returns the content type of this blob (example:
    /// Some("application/vnd.docker.image.rootfs.foreign.diff.tar.gzip"))
    pub fn content_type(&self) -> &Option<String> {
        &self.content_type
    }

    /// Stream a chunk of the blob contents.
    pub async fn chunk(&mut self) -> Result<Option<Bytes>, ErrorResponse> {
        match self.response.chunk().await {
            Ok(Some(chunk)) => {
                #[cfg(feature = "sha256")]
                self.hasher.input(&chunk);
                Ok(Some(chunk))
            }
            Ok(None) => Ok(None),
            Err(err) => Err(ErrorResponse::RequestError(err)),
        }
    }

    /// Returns the sha256 hash of the downloaded content.
    #[cfg(feature = "sha256")]
    pub fn digest(self) -> Digest {
        Digest::from_sha256(self.hasher.result())
    }
}

impl From<reqwest::Response> for Blob {
    fn from(response: reqwest::Response) -> Self {
        let headers = response.headers();
        let content_type = headers
            .get(reqwest::header::CONTENT_TYPE)
            .map(|v| std::str::from_utf8(v.as_ref()).unwrap().to_string());
        let len = response.content_length().map(|v| v as usize);

        Self {
            len,
            content_type,
            response,
            hasher: Sha256::new(),
        }
    }
}
