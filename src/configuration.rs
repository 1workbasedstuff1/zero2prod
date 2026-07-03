use config::File;
use secrecy::{ExposeSecret, SecretString};
// We need this function for getting data
use serde_aux::field_attributes::deserialize_number_from_string;
use sqlx::{
    ConnectOptions,
    postgres::{PgConnectOptions, PgSslMode},
};
use tracing;

#[derive(serde::Deserialize)]
pub struct Settings {
    pub database: DatabaseSettings,
    pub application: ApplicationSettings,
}

// note we had to remove the clone trait
// because SecretBox doesnt implement it
#[derive(serde::Deserialize)]
pub struct DatabaseSettings {
    pub username: String,
    pub password: SecretString,
    // use a custom function deserialize
    // allows it to be deserialized from a string or a raw number
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub port: u16,
    pub host: String,
    pub database_name: String,
    pub require_ssl: bool,
}

impl DatabaseSettings {
    // pub fn connection_string(&self) -> SecretString {
    //     format!(
    //         "postgres://{}:{}@{}:{}/{}",
    //         self.username,
    //         self.password.expose_secret(),
    //         self.host,
    //         self.port,
    //         self.database_name
    //     )
    //     .into()
    // }

    pub fn without_db(&self) -> PgConnectOptions {
        // needed for encrypted communications
        let ssl_mode = if self.require_ssl {
            PgSslMode::Require
        } else {
            PgSslMode::Prefer
        };

        PgConnectOptions::new()
            .host(&self.host)
            .username(&self.username)
            .password(&self.password.expose_secret())
            .port(self.port)
    }

    pub fn with_db(&self) -> PgConnectOptions {
        self.without_db()
            .database(&self.database_name)
            .log_statements(tracing::log::LevelFilter::Trace)
    }
}

// NOTE: added for docker connections to host machines
// NOTE: added serde to deserialize our input
#[derive(serde::Deserialize)]
pub struct ApplicationSettings {
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub port: u16,
    pub host: String,
}

pub enum Envireonment {
    Local,
    Production,
}

impl Envireonment {
    pub fn as_str(&self) -> &'static str {
        match self {
            Envireonment::Local => "local",
            Envireonment::Production => "production",
        }
    }
}

impl TryFrom<String> for Envireonment {
    type Error = String;
    fn try_from(s: String) -> Result<Self, Self::Error> {
        match s.to_lowercase().as_str() {
            "local" => Ok(Self::Local),
            "production" => Ok(Self::Production),
            other => Err(format!(
                "{} is not a supported envireonment, Use either 'local' or 'production'.",
                other
            )),
        }
    }
}

// NOTE: we've had to change this function
// so that it can check if we're in local or production
// NOTE: currently our data base details are stored in our
// config.yaml file, we dont want this to be the case during
// production
// [-]
// We want a way of injecting envireonment variables to inject
// secrets at runtime
pub fn get_configuration() -> Result<Settings, config::ConfigError> {
    let base_path = std::env::current_dir()
        .expect("Failed to determine current Directory");

    let configuration_directory = base_path.join("configuration");

    // Read the "default" configuration file
    let envireonment: Envireonment = std::env::var("APP_ENVIRONMENT")
        .unwrap_or_else(|_| "local".into()) // if nothing is found we default to local
        .try_into()
        .expect("Failed toe parse APP_ENVIRONMENT");

    // Layer on the envireonment-specific values
    let settings = config::Config::builder()
        // take all of our "base.yaml" configuration first
        .add_source(
            config::File::from(configuration_directory.join("base"))
                .required(true),
        )
        // check and parse into our envireonment enum
        // this will be local or production
        .add_source(
            config::File::from(
                configuration_directory.join(envireonment.as_str()),
            )
            .required(true),
        )
        .add_source(
            config::Environment::with_prefix("APP").separator("__"),
        )
        .build()?;

    // try_into -> try_deserialize with newer version of
    // config crate
    settings.try_deserialize()
}
