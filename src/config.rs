pub struct Config {
    pub host: String,
    pub port: String,
}

impl Config {
    pub fn new() -> Self {
        Config {
            host: std::env::var("HOST").unwrap_or("0.0.0.0".to_string()),
            port: std::env::var("PORT").unwrap_or("3000".to_string()),
        }
    }

    pub fn addr(&self) -> String {
        format!("{}:{}", self.host, self.port)
    }
}
