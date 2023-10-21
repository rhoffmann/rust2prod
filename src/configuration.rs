use config::Config;

#[derive(serde::Deserialize)]
pub struct Settings {
    pub database: DatabaseSettings,
    pub application_port: u16,
}

#[derive(serde::Deserialize)]
pub struct DatabaseSettings {
    pub username: String,
    pub password: String,
    pub port: u16,
    pub host: String,
    pub database_name: String,
}

pub fn get_configuration() -> Result<Settings, config::ConfigError> {
    let builder =
        Config::builder().add_source(config::File::new("configuration", config::FileFormat::Json));

    // look for 'configuration' top level file (yaml, json, etc)
    let s = builder.build()?;

    s.try_deserialize()
}
