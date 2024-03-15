use config::Config;
use secrecy::{ExposeSecret, Secret};

#[derive(serde::Deserialize)]
pub struct Settings {
    pub database: DatabaseSettings,
    pub application: ApplicationSettings,
}

#[derive(serde::Deserialize)]
pub struct ApplicationSettings {
    pub port: u16,
    pub host: String,
}

#[derive(serde::Deserialize)]
pub struct DatabaseSettings {
    pub username: String,
    pub password: Secret<String>,
    pub port: u16,
    pub host: String,
    pub database_name: String,
}

impl DatabaseSettings {
    pub fn connection_string(&self) -> Secret<String> {
        Secret::new(format!("{}/{}", self.connection_string_without_db().expose_secret(), self.database_name))
    }
    pub fn connection_string_without_db(&self) -> Secret<String> {
        Secret::new(format!(
            "postgres://{}:{}@{}:{}",
            self.username, self.password.expose_secret(), self.host, self.port
        ))
    }
}

pub fn get_configuration() -> Result<Settings, config::ConfigError> {
    let builder =
        Config::builder().add_source(config::File::new("configuration", config::FileFormat::Json));

    // TODO: add environment specific file handling, e.g. a different database server for testing?

    // look for 'configuration' top level file (yaml, json, etc)
    let s = builder.build()?;

    s.try_deserialize()
}
