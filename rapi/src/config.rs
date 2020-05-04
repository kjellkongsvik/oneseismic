use serde::Deserialize;

#[derive(Clone, Deserialize, Debug)]
pub struct Config {
    pub host_addr: String,
    pub authserver: String,
    pub audience: String,
    pub client_id: String,
    pub client_secret: String,
    pub azure_storage_account: String,
}

// Throw the Config struct into a CONFIG lazy_static to avoid multiple processing
lazy_static! {
    pub static ref CONFIG: Config = get_config();
}

/// Use envy to inject dotenv and env vars into the Config struct
fn get_config() -> Config {
    match envy::from_env::<Config>() {
        Ok(config) => config,
        Err(error) => panic!("Configuration Error: {:#?}", error),
    }
}
