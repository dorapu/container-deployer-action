use std::{
    process::{self},
    str::FromStr,
};

pub struct Docker {
    path: String,
    registry: String,
    image: String,
    version: String,
    hash: String,
}

impl Docker {
    pub fn new(path: String, registry: String, image: String, version: String) -> Self {
        Self {
            path,
            registry,
            image,
            version,
            hash: String::new(),
        }
    }

    pub fn build(&mut self) {
        println!(
            "building image \"ghcr.io/{}/{}\" in \"{}\":",
            self.registry, self.image, self.path
        );

        let command = format!(
            "docker build -q -t ghcr.io/{}/{} -f Containerfile .",
            self.registry, self.image
        );
        println!("  -> {}", command);

        let command = self.exec(command);

        if command.status.success() {
            let image_sha = String::from_utf8(command.stdout).unwrap();
            let image_sha = image_sha.trim();
            self.hash = String::from_str(image_sha).unwrap();

            println!("  -> done ({})", image_sha);
        } else {
            println!("  -> error:");
            println!("{}", format_stderr(command.stderr));
        }
    }

    pub fn tag(&self) {
        println!(
            "tagging latest image \"ghcr.io/{}/{}\" with tag \"{}\":",
            self.registry, self.image, self.version
        );

        let command = format!(
            "docker tag {} ghcr.io/{}/{}:{}",
            self.hash, self.registry, self.image, self.version
        );
        println!("  -> {}", command);

        let command = self.exec(command);

        if command.status.success() {
            println!("  -> done");
        } else {
            println!("  -> error:");
            println!("{}", format_stderr(command.stderr));
        }
    }

    pub fn push(&self) {
        println!(
            "pushing tagged image \"ghcr.io/{}/{}:{}\":",
            self.registry, self.image, self.version
        );

        let command = format!(
            "docker push ghcr.io/{}/{}:{}",
            self.registry, self.image, self.version
        );
        println!("  -> {}", command);

        let command = self.exec(command);

        if command.status.success() {
            println!("  -> done");
        } else {
            println!("  -> error:");
            println!("{}", format_stderr(command.stderr));
        }
    }

    fn exec(&self, command: String) -> process::Output {
        process::Command::new("sh")
            .arg("-c")
            .current_dir(self.path.as_str())
            .arg(command)
            .output()
            .unwrap()
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

pub fn login(username: String, password: String) {
    println!("logging to ghcr.io with username \"{}\":", username);
    println!(
        "  -> echo <REDACTED> | docker login ghcr.io -u {} --password-stdin",
        username
    );

    let command = format!(
        "echo {} | docker login ghcr.io -u {} --password-stdin",
        password, username
    );
    let command = process::Command::new("sh")
        .arg("-c")
        .arg(command.as_str())
        .output()
        .unwrap();

    if command.status.success() {
        let output = String::from_utf8(command.stdout).unwrap();
        println!("  -> done ({})", output.trim());
    } else {
        println!("  -> error:");
        println!("{}", format_stderr(command.stderr));
        panic!("unable to login!");
    }
}

fn format_stderr(stderr: Vec<u8>) -> String {
    let output = String::from_utf8(stderr).unwrap();
    let splits: Vec<&str> = output.split('\n').collect();
    let mut output = String::new();

    for split in splits {
        let split = split.trim();
        if !split.is_empty() {
            let split = format!("      {}\n", split);
            output.push_str(split.as_str());
        }
    }

    output
}
