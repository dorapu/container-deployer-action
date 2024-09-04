use base64::Engine;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct TagListResponse {
    pub tags: Vec<String>,
}

pub struct Ghcr {
    pub registry: String,
}

impl Ghcr {
    pub fn new(registry: String) -> Self {
        Self { registry }
    }

    pub fn list_tags(&self, image: String) -> Vec<String> {
        let client = reqwest::blocking::Client::new();
        let endpoint = format!("https://ghcr.io/v2/{}/{}/tags/list", self.registry, image);
        let token = std::env::var("GH_PASSWORD").unwrap();
        let token = base64::prelude::BASE64_STANDARD.encode(token);
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
}
