use base64::prelude::*;
use bollard::auth::DockerCredentials;
use serde::Deserialize;
use std::{collections::HashMap, env, fs};

#[derive(Debug)]
struct Repository {
    registry: String,
    _image: String,
}

#[derive(Debug, Deserialize)]
struct DockerConfig {
    auths: HashMap<String, DockerConfigAuth>,
}

#[derive(Debug, Deserialize)]
struct DockerConfigAuth {
    auth: String,
}

impl DockerConfig {
    fn get(&self, registry: &str) -> Option<(String, String)> {
        let encoded = self.auths.get(registry)?;
        let decoded = BASE64_STANDARD.decode(&encoded.auth).ok()?;
        let decoded_str = String::from_utf8(decoded).ok()?;

        decoded_str
            .split_once(':')
            .map(|(username, password)| (username.to_string(), password.to_string()))
    }
}

static DEFAULT_REGISTRY: &str = "docker.io";

pub fn registry_credentials(image: &str) -> Option<DockerCredentials> {
    let repository = parse_repository(image);
    let config = docker_config()?;

    config
        .get(&repository.registry)
        .map(|(username, password)| DockerCredentials {
            username: Some(username),
            password: Some(password),
            serveraddress: Some(repository.registry),
            ..Default::default()
        })
}

fn parse_repository(name: &str) -> Repository {
    let i = name.find('/');
    match i {
        Some(index) => {
            if !name[..index].contains(&['.', ':'][..]) && &name[..index] != "localhost" {
                Repository {
                    registry: DEFAULT_REGISTRY.to_string(),
                    _image: name.to_string(),
                }
            } else {
                Repository {
                    registry: name[..index].to_string(),
                    _image: name[index + 1..].to_string(),
                }
            }
        }
        None => Repository {
            registry: DEFAULT_REGISTRY.to_string(),
            _image: name.to_string(),
        },
    }
}

fn docker_config() -> Option<DockerConfig> {
    let path = env::var("DOCKER_CONFIG").unwrap_or("/config.json".to_string());
    let raw = fs::read_to_string(path).ok()?;

    serde_json::from_str(&raw).unwrap()
}
