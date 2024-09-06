use std::collections::HashMap;

#[derive(Debug)]
pub struct AppConfig {
    pub config_file: String,
}

impl AppConfig {
    pub fn lookup() -> Self {
        let key = "CONFIG_FILE";
        let config_file = match std::env::var(key) {
            Ok(value) => value,
            Err(error) => panic!("unable to get {} => {}", key, error.to_string()),
        };
        let config_file = config_file.trim().to_string();

        if config_file.contains('/') {
            panic!("the given config file is a directory");
        }

        if !config_file.ends_with(".toml") {
            panic!("config file must be a TOML file");
        }

        println!("config file is set to \"{}\".", config_file);

        Self { config_file }
    }
}

#[derive(Debug)]
pub struct DeploymentConfig {
    pub registries: HashMap<String, Vec<RegistryConfig>>,
}

#[derive(Debug)]
pub struct RegistryConfig {
    pub hostname: String,
    pub username: String,
    pub password: String,
    pub images: Vec<ImageConfig>,
}

#[derive(Debug)]
pub struct ImageConfig {
    pub path: String,
    pub source: String,
    pub repository: String,
    pub name: String,
    pub tag: String,
    pub replace: bool,
    pub ignore: bool,
}

impl DeploymentConfig {
    pub fn lookup(config_file: String) -> Self {
        let config_files = Self::collect_config_files(config_file);
        let registries = Self::collect_registries(config_files);

        Self { registries }
    }

    fn collect_config_files(config_file: String) -> Vec<String> {
        let root = std::env::current_dir().unwrap();
        let root = std::path::Path::new(root.as_os_str());
        let files = crate::util::FileUtil::list_files(&root);
        let mut config_files: Vec<String> = vec![];

        print!("collecting all {} file in this repository... ", config_file);

        for file in files {
            if file.ends_with(config_file.as_str()) {
                config_files.push(file);
            }
        }

        config_files.sort();

        if config_files.is_empty() {
            println!("found nothing. action aborted!");
        } else {
            println!("found {}:", config_files.len());
            for file in config_files.clone() {
                println!("  -> {}", file);
            }
        }

        config_files
    }

    fn collect_registries(config_files: Vec<String>) -> HashMap<String, Vec<RegistryConfig>> {
        let mut registries: HashMap<String, Vec<RegistryConfig>> = HashMap::new();

        for config_file in config_files {
            print!("reading config from \"{}\"... ", config_file);

            let config_content = std::fs::read_to_string(config_file.clone()).unwrap();
            let config_table = config_content.as_str().parse::<toml::Table>().unwrap();
            let registry_table = config_table.get("registries").unwrap().as_table().unwrap();
            let mut errors: HashMap<String, Vec<String>> = HashMap::new();

            for key in registry_table.keys() {
                let mut error_messages: Vec<String> = vec![];
                let registry = registry_table.get(key).unwrap().as_table().unwrap();
                let registry_config = Self::read_registry_config(registry, &mut error_messages);
                let image_tables = registry.get("images").unwrap().as_table().unwrap();
                let image_configs =
                    Self::collect_images(config_file.clone(), image_tables, &mut error_messages);

                let registry = RegistryConfig {
                    hostname: registry_config.hostname,
                    username: registry_config.username,
                    password: registry_config.password,
                    images: image_configs,
                };

                if error_messages.is_empty() {
                    if !registries.contains_key(key) {
                        registries.insert(key.to_string(), vec![]);
                    }

                    registries.get_mut(key).unwrap().push(registry);
                } else {
                    if !errors.contains_key(key) {
                        errors.insert(key.to_string(), vec![]);
                    }

                    errors.get_mut(key).unwrap().append(&mut error_messages);
                }
            }

            if errors.is_empty() {
                println!("done.");
            } else {
                println!("failed.");

                for error_map in errors {
                    println!("  -> errors in \"{}\":", error_map.0);
                    let error_messages = error_map.1;
                    for message in error_messages {
                        println!("    -> {}", message);
                    }
                }
            }
        }

        registries
    }

    fn read_registry_config(registry: &toml::Table, errors: &mut Vec<String>) -> RegistryConfig {
        let hostname = Self::resolve_environment_variable(registry, "hostname", errors);
        let username = Self::resolve_environment_variable(registry, "username", errors);
        let password = Self::resolve_environment_variable(registry, "password", errors);

        RegistryConfig {
            hostname,
            username,
            password,
            images: vec![],
        }
    }

    fn resolve_environment_variable(
        registry: &toml::Table,
        key: &'static str,
        errors: &mut Vec<String>,
    ) -> String {
        if !registry.contains_key(key) {
            errors.push(format!("{} field doesn't exist.", key));
            return String::new();
        }

        let original_value = registry
            .get(key)
            .unwrap()
            .as_str()
            .unwrap()
            .trim()
            .to_string();

        if original_value.is_empty() {
            errors.push(format!("{} field is empty.", key));
            return String::new();
        }

        match std::env::var(original_value.trim()) {
            Ok(resolved_value) => resolved_value,
            Err(error) => {
                errors.push(format!(
                    "encountered a problem when trying to resolve {}. ({})",
                    original_value, error
                ));
                String::new()
            }
        }
    }

    fn collect_images(
        path: String,
        image_tables: &toml::Table,
        errors: &mut Vec<String>,
    ) -> Vec<ImageConfig> {
        let mut images: Vec<ImageConfig> = vec![];

        for key in image_tables.keys() {
            let image = image_tables.get(key).unwrap().as_table().unwrap();
            let name_field = Self::resolve_image_field(key, image, "name", true, "", errors);
            let tag_field = Self::resolve_image_field(key, image, "tag", false, "latest", errors);
            let ignore_field =
                Self::resolve_image_field(key, image, "ignore", false, "false", errors);
            let ignore_field = ignore_field == "true";

            if ignore_field {
                let image = ImageConfig {
                    path: String::new(),
                    source: String::new(),
                    repository: String::new(),
                    name: name_field,
                    tag: tag_field,
                    replace: false,
                    ignore: true,
                };

                images.push(image);
            } else {
                let source_field =
                    Self::resolve_image_field(key, image, "source", true, "", errors);
                let repository_field =
                    Self::resolve_image_field(key, image, "repository", true, "", errors);
                let replace_field =
                    Self::resolve_image_field(key, image, "replace", false, "false", errors);
                let replace_field = replace_field == "true";
                let path = std::path::Path::new(path.as_str()).parent().unwrap();
                let path = path.to_str().unwrap().to_string();
                let image = ImageConfig {
                    path,
                    source: source_field,
                    repository: repository_field,
                    name: name_field,
                    tag: tag_field,
                    replace: replace_field,
                    ignore: ignore_field,
                };

                images.push(image);
            }
        }

        images
    }

    fn resolve_image_field(
        section: &String,
        image: &toml::Table,
        field: &'static str,
        required: bool,
        default: &'static str,
        errors: &mut Vec<String>,
    ) -> String {
        if image.contains_key(field) {
            if default == "true" || default == "false" {
                image.get(field).unwrap().as_bool().unwrap().to_string()
            } else {
                image.get(field).unwrap().as_str().unwrap().to_string()
            }
        } else {
            if required {
                errors.push(format!(
                    "in \"{}\" section: missing required field \"{}\".",
                    section, field
                ));
            }
            default.to_string()
        }
    }
}
