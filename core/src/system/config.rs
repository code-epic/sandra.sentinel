use std::env;

pub struct Config {
    pub sentinel_host: String,
    pub sentinel_port: String,
    pub version: String,
    pub autor: String,
    pub description: String,
    pub format_file: String,
}

impl Config {
    pub fn load() -> Self {
        // Cargar variables de entorno o usar valores por defecto
        Config {
            sentinel_host: env::var("SENTINEL_HOST").unwrap_or_else(|_| "[::1]".to_string()),
            sentinel_port: env::var("SENTINEL_PORT").unwrap_or_else(|_| "50051".to_string()),
            version: env::var("VERSION").unwrap_or_else(|_| "0.1.0".to_string()),
            autor: env::var("AUTOR").unwrap_or_else(|_| "Sandra Team".to_string()),
            description: env::var("DESCRIPTION").unwrap_or_else(|_| "Sandra Sentinel Core".to_string()),
            format_file: env::var("FORMAT_FILE").unwrap_or_else(|_| "json".to_string()),
        }
    }

    pub fn get_url(&self) -> String {
        format!("http://{}:{}", self.sentinel_host, self.sentinel_port)
    }
}
