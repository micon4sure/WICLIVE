use serde::Serialize;

#[derive(Serialize)]
#[allow(non_snake_case)]
pub struct Config {
    pub API_URL: String,
    pub VERSION: String,
}

impl Config {
    pub fn new() -> Self {
        // set api url
        let env = std::env::var("WICLIVE_ENV").unwrap_or_else(|_| "production".to_string());
        let api_url = match env.as_str() {
            "development" => "http://localhost:3243".to_string(),
            "staging" => "https://techtile.media:3243".to_string(),
            "production" => "https://techtile.media:3243".to_string(),
            _ => "http://localhost:3243".to_string(),
        };

        // get version from cargo.toml
        let version = env!("CARGO_PKG_VERSION").to_string();
        Config {
            API_URL: api_url,
            VERSION: version,
        }
    }
}
