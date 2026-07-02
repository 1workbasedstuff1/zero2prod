use config::File;
use secrecy::{ExposeSecret, SecretString};

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
    pub port: u16,
    pub host: String,
    pub database_name: String,
}

impl DatabaseSettings {
    pub fn connection_string(&self) -> SecretString {
        format!(
            "postgres://{}:{}@{}:{}/{}",
            self.username,
            self.password.expose_secret(),
            self.host,
            self.port,
            self.database_name
        )
        .into()
    }
}

// NOTE: added for docker connections to host machines
#[derive(serde::Deserialize)]
pub struct ApplicationSettings {
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
        .build()?;

    // try_into -> try_deserialize with newer version of
    // config crate
    settings.try_deserialize()
}
