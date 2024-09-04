use std::{fs, path::Path, str::FromStr};

pub struct FileUtil;

impl FileUtil {
    pub fn list_files(path: &Path) -> Vec<String> {
        let mut files: Vec<String> = vec![];

        if path.is_file() {
            let path = String::from_str(path.to_str().unwrap()).unwrap();
            files.push(path);
        } else if path.is_dir() {
            for path in fs::read_dir(path).unwrap() {
                let path = path.unwrap().path();
                files.append(&mut Self::list_files(&path));
            }
        }

        return files;
    }
}
