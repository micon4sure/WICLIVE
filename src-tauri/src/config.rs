use serde::Serialize;

#[derive(Serialize)]
#[allow(non_snake_case)]
pub struct Config {
    pub API_URL: String,
    pub VERSION: String,
    pub MASSGATE_URL: String,
    pub DEBUG: bool,
}

impl Config {
    pub fn new() -> Self {
        // set api url
        let env = env!("WICLIVE_ENV");
        let api_url;
        let massgate_url;
        let debug;
        match env {
            "development" => {
                api_url = "http://localhost:3243".to_string();
                massgate_url = "http://localhost:3243".to_string();
                debug = true;
            }
            "testing" => {
                api_url = "https://techtile.media:3243".to_string();
                massgate_url = "https://www.massgate.org".to_string();
                debug = false;
            }
            "staging" => {
                api_url = "https://techtile.media:3243".to_string();
                massgate_url = "https://www.massgate.org".to_string();
                debug = false;
            }
            "production" => {
                api_url = "https://techtile.media:3243".to_string();
                massgate_url = "https://www.massgate.org".to_string();
                debug = false;
            }
            _ => panic!("Invalid environment"),
        };

        // get version from cargo.toml
        let version = env!("CARGO_PKG_VERSION").to_string();
        Config {
            API_URL: api_url,
            VERSION: version,
            MASSGATE_URL: massgate_url,
            DEBUG: debug,
        }
    }
}
