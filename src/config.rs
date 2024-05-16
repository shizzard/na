//!
//! Module containing server configuration data structures and functions for
//! building the configuration from various sources.

use config::Config;
use config::ConfigError;
use config::Environment;
use config::File;
use serde_derive::Deserialize;

/// Database configuration.
#[derive(Deserialize, Debug, Clone, Default)]
pub struct DatabaseConfig {
    /// Database connection URI.
    ///
    /// Example: "postgres://postgres:postgres@localhost:5432/na"
    pub url: String,
}

/// HTTP Configuration.
#[derive(Deserialize, Debug, Clone, Default)]
pub struct HttpConfig {
    /// Host to bind the listener to.
    pub listen_host: String,
    /// Port to bind the listener to.
    pub listen_port: u16,
}

impl HttpConfig {
    /// Returns a string representing the configured network endpoint.
    pub fn as_bind_str(&self) -> String {
        format!("{}:{}", self.listen_host, self.listen_port)
    }
}

/// JWT configuration.
#[derive(Deserialize, Debug, Clone, Default)]
pub struct JwtConfig {
    /// Shared secret, used to encrypt and decrypt the JWT tokens.
    ///
    /// Sensitive.
    pub secret: String,
}

///
/// Server configuration
#[derive(Deserialize, Debug, Clone, Default)]
pub struct ServerConfig {
    /// Database configuration.
    pub database: DatabaseConfig,
    /// HTTP configuration.
    pub http: HttpConfig,
    /// JWT configuration.
    pub jwt: JwtConfig,
}

impl ServerConfig {
    ///
    /// Create a new server config structure.
    ///
    /// Two configuration sources are considered:
    /// - config/default.toml: default configuration, suitable for local setup
    /// - env variables
    ///
    /// Env variables are parsed as follows:
    /// - Global prefix is `NA__`
    /// - Path separator is `__`
    ///
    /// Example:
    /// To set `ServerConfig.http.listen_port` via env variable, you should
    /// have env variable `NA__HTTP__LISTEN_PORT` set to desired port number.
    pub fn new() -> Result<Self, ConfigError> {
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
        Ok(cfg)
    }

    ///
    /// Create a new leaked (&'static) config structure.
    pub fn new_leaked() -> &'static ServerConfig {
        Box::leak(Box::new(
            ServerConfig::new()
                .map_err(move |err| {
                    log::error!("Failed to configure: {}", err);
                    std::process::exit(1);
                })
                .unwrap(),
        ))
    }
}
