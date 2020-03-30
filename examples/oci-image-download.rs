use oci_registry_client::DockerRegistryClientV2;
use std::{env, error::Error, fs::File, io::Write, path::Path};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let mut args = env::args();
    let image = args.nth(1).unwrap_or("library/alpine".to_string());
    let reference = args.nth(2).unwrap_or("latest".to_string());
    let out_dir = args.nth(3).unwrap_or("/tmp".to_string());

    let mut client = DockerRegistryClientV2::new(
        "registry.docker.io",
        "https://registry-1.docker.io",
        "https://auth.docker.io/token",
    );

    match client.auth("repository", &image, "pull").await {
        Ok(token) => client.set_auth_token(Some(token)),
        Err(err) => {
            eprintln!("auth failed; err={}", err);
            std::process::exit(-1);
        }
    }

    match client.manifest(&image, &reference).await {
        Ok(manifest) => {
            for layer in &manifest.layers {
                println!("Downloading {} ...", layer.digest);
                match File::create(Path::new(&out_dir).join(&layer.digest)) {
                    Ok(mut out_file) => match client.blob(&image, &layer.digest).await {
                        Ok(mut blob) => loop {
                            match blob.chunk().await {
                                Ok(Some(chunk)) => {
                                    if let Err(err) = out_file.write_all(&chunk) {
                                        eprintln!("failed to write layer; err={}", err);
                                        std::process::exit(-1);
                                    }
                                }
                                Ok(None) => break,
                                Err(err) => {
                                    eprintln!("failed to download layer; err={}", err);
                                    std::process::exit(-1);
                                }
                            }
                        },
                        Err(err) => {
                            eprintln!("failed to fetch layer blob; err={}", err);
                            std::process::exit(-1);
                        }
                    },
                    Err(err) => {
                        eprintln!("failed to create layer file; err={}", err);
                        std::process::exit(-1);
                    }
                }
            }
        }
        Err(err) => {
            eprintln!("failed to get manifest; err={}", err);
            std::process::exit(-1);
        }
    }

    Ok(())
}
