mod config;
mod docker;
mod util;

use docker::Docker;

fn main() {
    docker::check();

    let app_config = config::AppConfig::lookup();
    let deployment_config = config::DeploymentConfig::lookup(app_config.config_file);
    let mut error_count = 0;

    for (name, registries) in deployment_config.registries {
        println!("starting the deployment process for \"{}\" registry:", name);

        if registries.is_empty() {
            println!("  -> skipped. there is no image config.");
        } else {
            let first_registry = registries.first().unwrap();
            let hostname = first_registry.hostname.clone();
            let username = first_registry.username.clone();
            let password = first_registry.password.clone();

            if !docker::login(hostname, username, password) {
                continue;
            }
        }

        for registry in registries {
            for image in registry.images {
                println!(
                    "  -> processing image \"{}\" with tag \"{}\":",
                    image.name, image.tag
                );

                if image.ignore {
                    println!("    -> set as to ignore by the config. ignored.");
                } else {
                    let mut docker = Docker::new(
                        image.path,
                        image.source,
                        registry.hostname.clone(),
                        registry.password.clone(),
                        image.repository,
                        image.name,
                        image.tag.clone(),
                    );
                    let mut publish = true;

                    if !image.replace {
                        let tags = docker.tag_list();
                        if tags.contains(&image.tag) {
                            publish = false;
                        }
                    }

                    if publish {
                        docker.build();

                        if is_dry_run() {
                            println!("    -> dry run mode: no more action.");
                        } else {
                            docker.tag();
                            docker.push();
                        }

                        docker.cleanup();

                        if !docker.has_hash() {
                            error_count += 1;
                        }
                    } else {
                        println!("    -> will not publish since tag already exist. \"replace\" field config is set to false.");
                    }
                }
            }
        }
    }

    if error_count > 0 {
        panic!("found error.");
    } else {
        println!("all done.");
    }
}

fn is_dry_run() -> bool {
    match std::env::var("DRY_RUN") {
        Ok(value) => value == "true",
        Err(_) => false,
    }
}
