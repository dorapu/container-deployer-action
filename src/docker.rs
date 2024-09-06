use base64::Engine;
use serde::Deserialize;
use std::{
    process::{self},
    str::FromStr,
};

#[derive(Debug, Deserialize)]
struct TagListResponse {
    pub tags: Vec<String>,
}

pub struct Docker {
    path: String,
    source: String,
    hostname: String,
    password: String,
    repository: String,
    image: String,
    tag: String,
    hash: String,
}

impl Docker {
    pub fn new(
        path: String,
        source: String,
        hostname: String,
        password: String,
        repository: String,
        image: String,
        tag: String,
    ) -> Self {
        Self {
            path,
            source,
            hostname,
            password,
            repository,
            image,
            tag,
            hash: String::new(),
        }
    }

    pub fn tag_list(&self) -> Vec<String> {
        let client = reqwest::blocking::Client::new();
        let endpoint = format!(
            "https://{}/v2/{}/{}/tags/list",
            self.hostname, self.repository, self.image
        );
        let token = base64::prelude::BASE64_STANDARD.encode(self.password.as_str());
        let token = format!("Bearer {}", token);
        let response = client
            .get(endpoint)
            .header(reqwest::header::AUTHORIZATION, token)
            .send()
            .unwrap();

        if response.status().is_success() {
            let response: TagListResponse = response.json().unwrap();
            response.tags
        } else {
            vec![]
        }
    }

    pub fn build(&mut self) {
        let command = format!(
            "docker build -q -t {}/{}/{} -f {} .",
            self.hostname, self.repository, self.image, self.source
        );
        println!("    -> {}", command);

        let command = process::Command::new("sh")
            .arg("-c")
            .current_dir(self.path.as_str())
            .arg(command)
            .output()
            .unwrap();

        if command.status.success() {
            let image_sha = String::from_utf8(command.stdout).unwrap();
            let image_sha = image_sha.trim();
            self.hash = String::from_str(image_sha).unwrap();

            println!("    -> done ({})", image_sha);
        } else {
            println!("    -> error:");
            println!("{}", format_stderr(command.stderr));
        }
    }

    pub fn tag(&self) {
        if self.hash.is_empty() {
            return;
        }

        let command = format!(
            "docker tag {} {}/{}/{}:{}",
            self.hash, self.hostname, self.repository, self.image, self.tag
        );
        println!("    -> {}", command);

        self.exec(command);
    }

    pub fn push(&self) {
        if self.hash.is_empty() {
            return;
        }

        let command = format!(
            "docker push {}/{}/{}:{}",
            self.hostname, self.repository, self.image, self.tag
        );
        println!("    -> {}", command);

        self.exec(command);
    }

    pub fn cleanup(&self) {
        if self.hash.is_empty() {
            return;
        }

        let command = format!("docker rmi {} -f", self.hash);
        println!("    -> {}", command);

        self.exec(command);
    }

    pub fn has_hash(&self) -> bool {
        !self.hash.is_empty()
    }

    fn exec(&self, command: String) {
        let command = process::Command::new("sh")
            .arg("-c")
            .current_dir(self.path.as_str())
            .arg(command)
            .output()
            .unwrap();

        if command.status.success() {
            println!("    -> done");
        } else {
            println!("    -> error:");
            println!("{}", format_stderr(command.stderr));
        }
    }
}

pub fn check() {
    print!("checking whether docker command exists... ");

    let command = process::Command::new("sh")
        .arg("-c")
        .arg("which docker")
        .output()
        .unwrap();

    if !command.status.success() {
        println!("not found.");
        panic!("docker command is not available");
    }

    println!("it is.");
}

pub fn login(hostname: String, username: String, password: String) -> bool {
    println!(
        "  -> logging in to \"{}\" with username \"{}\":",
        hostname, username
    );
    println!(
        "    -> echo <REDACTED> | docker login {} -u {} --password-stdin",
        hostname, username
    );

    let command = format!(
        "echo {} | docker login {} -u {} --password-stdin",
        password, hostname, username
    );
    let command = process::Command::new("sh")
        .arg("-c")
        .arg(command.as_str())
        .output()
        .unwrap();

    if command.status.success() {
        let output = String::from_utf8(command.stdout).unwrap();
        println!("    -> done ({})", output.trim());

        true
    } else {
        println!("    -> failed. will skip all images that use this credential. error:");
        println!("{}", format_stderr(command.stderr));

        false
    }
}

fn format_stderr(stderr: Vec<u8>) -> String {
    let output = String::from_utf8(stderr).unwrap();
    let splits: Vec<&str> = output.split('\n').collect();
    let mut output = String::new();

    for split in splits {
        let split = split.trim();
        if !split.is_empty() {
            let split = format!("        {}\n", split);
            output.push_str(split.as_str());
        }
    }

    output.trim_end().to_string()
}
