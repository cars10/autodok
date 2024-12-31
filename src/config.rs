pub struct Config {
    pub host: String,
    pub port: String,
}

impl Config {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn addr(&self) -> String {
        format!("{}:{}", self.host, self.port)
    }
}

impl Default for Config {
    fn default() -> Self {
        Config {
            host: std::env::var("AUTODOK_HOST").unwrap_or("0.0.0.0".to_string()),
            port: std::env::var("AUTODOK_PORT").unwrap_or("3000".to_string()),
        }
    }
}
