use oci_registry_client::{
    manifest::{Digest, Layer},
    DockerRegistryClientV2,
};
use std::error::Error;
use std::fs::File;
use std::io::Write;
use tokio::sync::mpsc;

#[derive(Debug)]
struct DownloadProgressReport {
    n: usize,
    digest: Digest,
    downloaded: usize,
    total: usize,
}

async fn download_layer(
    n: usize,
    digest: Digest,
    layer: Layer,
    client: DockerRegistryClientV2,
    tx: mpsc::UnboundedSender<DownloadProgressReport>,
) -> Result<(), Box<dyn Error + Send>> {
    let mut blob = client.blob("library/alpine", &layer.digest).await.unwrap();
    let total = blob.len();
    let mut downloaded = 0usize;
    let mut out_file = File::create(format!("/tmp/{}.tar.gz", layer.digest)).unwrap();

    while let Some(chunk) = blob.chunk().await.unwrap() {
        downloaded += chunk.len();
        if let Some(total) = total {
            tx.send(DownloadProgressReport {
                n,
                digest: digest.clone(),
                downloaded,
                total,
            })
            .unwrap();
        }

        out_file.write_all(&chunk).unwrap();
    }

    Ok(())
}

enum LayerDownloadStatus {
    Unknown(Digest),
    Downloading(Digest, usize, usize),
    Completed(Digest),
}

impl LayerDownloadStatus {
    pub fn completed(&self) -> bool {
        matches!(self, LayerDownloadStatus::Completed(_))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let mut client = DockerRegistryClientV2::new(
        "registry.docker.io",
        "https://registry-1.docker.io",
        "https://auth.docker.io/token",
    );
    let response = client.auth("repository", "library/alpine", "pull").await;
    if let Ok(token) = response {
        client.set_auth_token(Some(token));
    }

    let manifest_list = client.list_manifests("library/alpine", "latest").await?;

    for manifest in &manifest_list.manifests {
        println!("{:?}", manifest);
        if manifest.platform.architecture == "amd64" && manifest.platform.os == "linux" {
            let response = client
                .manifest("library/alpine", &manifest.digest.to_string())
                .await?;

            println!("response: {:?}", response);
        }
    }

    let response = client.manifest("library/alpine", "latest").await?;

    let (tx, mut rx) = mpsc::unbounded_channel::<DownloadProgressReport>();

    let mut layers_status = vec![];

    for (n, layer) in response.layers.iter().cloned().enumerate() {
        let client = client.clone();
        if response.layers[0..n]
            .iter()
            .any(|l| l.digest == layer.digest)
        {
            continue;
        }

        layers_status.push(LayerDownloadStatus::Unknown(layer.digest.clone()));
        tokio::spawn(download_layer(
            layers_status.len() - 1,
            layer.digest.clone(),
            layer.clone(),
            client.clone(),
            tx.clone(),
        ));
    }

    loop {
        let progress = rx.recv().await.unwrap();

        if progress.downloaded == progress.total {
            layers_status[progress.n] = LayerDownloadStatus::Completed(progress.digest);
        } else {
            layers_status[progress.n] = LayerDownloadStatus::Downloading(
                progress.digest,
                progress.downloaded,
                progress.total,
            );
        }

        for status in &layers_status {
            print!("\x1B[K");
            match status {
                LayerDownloadStatus::Unknown(digest) => println!("{}: unknown", digest),
                LayerDownloadStatus::Downloading(digest, downloaded, total) => println!(
                    "{}: {}/{} ({:.2}%)",
                    digest,
                    downloaded / 1024,
                    total / 1024,
                    *downloaded as f32 / *total as f32 * 100f32
                ),
                LayerDownloadStatus::Completed(digest) => println!("{}: completed", digest),
            }
        }

        if layers_status.iter().all(|s| s.completed()) {
            break;
        }

        print!("\x1B[{}A", layers_status.len());
    }

    Ok(())
}
