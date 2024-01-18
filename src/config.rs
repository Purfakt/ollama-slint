use dotenvy::dotenv;
use std::env;
use std::sync::OnceLock;

#[derive(Debug)]
pub struct OllamaConfig {
    host: String,
    port: u16,
    model: String,
}

#[derive(Debug)]
pub struct Configuration {
    ollama: OllamaConfig,
}

impl Configuration {
    pub fn ollama_host(&self) -> &str {
        &self.ollama.host
    }

    pub fn ollama_port(&self) -> u16 {
        self.ollama.port
    }

    pub fn ollama_model(&self) -> &str {
        &self.ollama.model
    }
}

pub static CONFIGURATION: OnceLock<Configuration> = OnceLock::new();

fn load_configuration() -> Configuration {
    dotenv().ok();

    let ollama_host =
        env::var("OLLAMA_HOST").expect("OLLAMA_HOST environment variable should be set");
    let ollama_port = env::var("OLLAMA_PORT")
        .expect("OLLAMA_PORT environment variable should be set")
        .parse::<u16>()
        .expect("OLLAMA_PORT should be a valid port number");
    let ollama_model = env::var("OLLAMA_MODEL").expect("MODEL environment variable should be set");

    Configuration {
        ollama: OllamaConfig {
            host: ollama_host,
            port: ollama_port,
            model: ollama_model,
        },
    }
}

pub fn config() -> &'static Configuration {
    CONFIGURATION.get_or_init(load_configuration)
}
