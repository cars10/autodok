use log::info;
use std::fs;

static DATA_DIR_PATH: &str = "data";
static API_KEY_PATH: &str = "data/api_key";

pub fn api_key() -> String {
    let api_key = existing_api_key().unwrap_or_else(setup_api_key);
    info!("API key: {api_key}");

    api_key
}

fn existing_api_key() -> Option<String> {
    fs::read_to_string(API_KEY_PATH).ok()
}

fn setup_api_key() -> String {
    fs::create_dir_all(DATA_DIR_PATH).unwrap();

    info!("No API key found, generating new API key...");
    let api_key = crate::random::random_string(30);
    fs::write(API_KEY_PATH, &api_key).unwrap();

    api_key
}
