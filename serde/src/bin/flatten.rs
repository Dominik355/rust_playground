use serde::{Deserialize, Serialize};
use std::io::Read;

fn main() {
    let mut file = std::fs::OpenOptions::new()
        .read(true)
        .open("serde/example.toml")
        .unwrap();

    let mut text = String::new();
    file.read_to_string(&mut text).unwrap();

    let config: AppConfig = toml::de::from_str(&text).unwrap();

    println!("Config: {:#?}", config);
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct AppConfig {
    pub name: String,
    pub s3_sink: S3SinkConfig,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct S3SinkConfig {
    pub bucket: String,
    #[serde(flatten)]
    pub s3_config: S3Config,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct S3Config {
    pub access_key: String,
    pub secret_key: String,
    pub endpoint: String,
    pub port: u32,
}
