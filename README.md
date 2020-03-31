OCI Registry Client
===================

[![crates.io](https://img.shields.io/crates/v/oci-registry-client.svg)](https://crates.io/crates/oci-registry-client)
[![Documentation](https://docs.rs/oci-registry-client/badge.svg)](https://docs.rs/oci-registry-client)
[![MIT](https://img.shields.io/github/license/ecarrara/oci-registry-client)](./LICENSE)

A async client for [OCI compliant image registries](http://github.com/opencontainers/distribution-spec/blob/master/spec.md/)
and [Docker Registry HTTP V2 protocol](https://docs.docker.com/registry/spec/api/).

# Usage

The [`DockerRegistryClientV2`] provides functions to query Registry API and download blobs.

```rust
use std::{path::Path, fs::File, io::Write};
use oci_registry_client::DockerRegistryClientV2;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut client = DockerRegistryClientV2::new(
        "registry.docker.io",
        "https://registry-1.docker.io",
        "https://auth.docker.io/token"
    );
    let token = client.auth("repository", "library/ubuntu", "latest").await?;
    client.set_auth_token(Some(token));

    let manifest = client.manifest("library/ubuntu", "latest").await?;
    println!("{:?}", manifest);

    for layer in &manifest.layers {
       let mut out_file = File::create(Path::new("/tmp/").join(&layer.digest))?;
       let mut blob = client.blob("library/ubuntu", &layer.digest).await?;

       while let Some(chunk) = blob.chunk().await? {
           out_file.write_all(&chunk)?;
       }
    }

    Ok(())
}
```


## License

This project is licensed under the [MIT
License](https://github.com/ecarrara/oci-registry-client/blob/master/LICENSE).

## Contribution

Unless you explicitly state otherwise, any contribution intentionally
submitted for inclusion in oci-registry-client by you, shall be
licensed as MIT, without any additional terms or conditions.
