use oci_registry_client::DockerRegistryClientV2;
use serde_json;
use std::env;
use std::error::Error;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let mut args = env::args();
    let image = args.nth(1).unwrap_or("library/alpine".to_string());
    let reference = args.next().unwrap_or("latest".to_string());

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
            match serde_json::to_string(&manifest) {
                Ok(repr) => println!("{}", repr),
                Err(err) => {
                    eprintln!("failed to parse manifest json; err={}", err);
                    std::process::exit(-1);
                }
            }

            match client.config(&image, &manifest.config.digest).await {
                Ok(config) => match serde_json::to_string(&config) {
                    Ok(repr) => println!("{}", repr),
                    Err(err) => {
                        eprintln!("failed to parse image config json; err={}", err);
                        std::process::exit(-1);
                    }
                },
                Err(err) => {
                    eprintln!("failed to get image config; err={}", err);
                    std::process::exit(-1);
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
