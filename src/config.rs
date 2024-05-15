use config::Config;
use config::Environment;
use config::File;
use serde_derive::Deserialize;
use std::error::Error;

#[derive(Deserialize, Debug, Clone, Default)]
pub(crate) struct DatabaseConfig {
    pub url: String,
}

#[derive(Deserialize, Debug, Clone, Default)]
pub(crate) struct HttpConfig {
    pub listen_host: String,
    pub listen_port: u16,
}

impl HttpConfig {
    pub fn as_bind_str(&self) -> String {
        format!("{}:{}", self.listen_host, self.listen_port)
    }
}

#[derive(Deserialize, Debug, Clone, Default)]
pub(crate) struct JwtConfig {
    pub secret: String,
}

#[derive(Deserialize, Debug, Clone, Default)]
pub(crate) struct ServerConfig {
    pub database: DatabaseConfig,
    pub http: HttpConfig,
    pub jwt: JwtConfig,
}

impl ServerConfig {
    pub(crate) fn new() -> Result<&'static Self, Box<dyn Error>> {
        let cfg = Config::builder()
            .add_source(File::with_name("config/default.toml"))
            .add_source(
                Environment::with_prefix("na")
                    .separator("__")
                    .list_separator(","),
            )
            .build()?
            .try_deserialize::<ServerConfig>()?;

        // Probably there is a better way to make config global
        Ok(Box::leak(Box::new(cfg)))
    }
}

pub(crate) async fn get_leaked() -> &'static ServerConfig {
    ServerConfig::new()
        .map_err(move |err| {
            log::error!("Failed to configure: {}", err);
            std::process::exit(1);
        })
        .unwrap()
}
