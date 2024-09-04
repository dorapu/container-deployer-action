use std::{env, path::Path};

use crate::util::FileUtil;

pub struct Containerfile {
    pub paths: Vec<String>,
}

impl Containerfile {
    pub fn new() -> Self {
        let root = env::current_dir().unwrap();
        let root = root.as_os_str();
        let root = Path::new(root);
        let files = FileUtil::list_files(&root);
        let mut paths: Vec<String> = vec![];

        print!("populating all Containerfile in this repository... ");

        for file in files {
            if file.ends_with("Containerfile") {
                let path = Path::new(file.as_str()).parent().unwrap();
                let path = path.to_str().unwrap().to_string();
                paths.push(path);
            }
        }

        paths.sort();

        if paths.is_empty() {
            println!("found nothing. action aborted!");
        } else {
            println!("found {} path(s) with Containerfile:", paths.len());
            for path in paths.clone() {
                println!("  -> {}", path);
            }
        }

        Self { paths }
    }
}
