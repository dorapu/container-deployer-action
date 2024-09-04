use std::{env, path::Path, str::FromStr};

mod containerfile;
mod docker;
mod ghcr;
mod util;

use containerfile::Containerfile;
use docker::Docker;
use ghcr::Ghcr;

fn main() {
    docker::check();
    docker::login(get_env("GH_USERNAME"), get_env("GH_PASSWORD"));

    let ghcr = Ghcr::new(get_env("GH_REGISTRY"));
    let containerfile = Containerfile::new();

    for path in containerfile.paths {
        println!("processing \"{}\":", path);

        let splits = path.clone();
        let splits: Vec<&str> = splits.split('/').collect();
        let version_path = path.clone() + "/version";
        let version_path = Path::new(version_path.as_str());
        let image = splits.last().unwrap();
        let mut version = String::new();
        let mut exists = false;

        if version_path.exists() {
            let content = std::fs::read_to_string(version_path).unwrap();
            version.push_str(content.trim());
        }

        if !version.is_empty() {
            let tags = ghcr.list_tags(image.to_string());
            exists = tags.contains(&version);
        }

        println!("  -> image = {}", image);
        println!("  -> version = {}", version);
        println!("  -> exists = {}", exists);

        if exists {
            println!("  -> skipped");
        } else {
            let mut docker = Docker::new(
                path,
                get_env("GH_REGISTRY"),
                String::from_str(image).unwrap(),
                version,
            );

            docker.build();
            docker.tag();
            docker.push();
        }
    }

    println!("all done.");
}

fn get_env(key: &'static str) -> String {
    match env::var(key) {
        Ok(value) => value,
        Err(error) => panic!("unable to get {} => {}", key, error.to_string()),
    }
}
