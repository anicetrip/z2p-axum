use reqwest::Url;
use sea_orm::ConnectOptions;
use secrecy::{ExposeSecret, SecretString};
use serde::Deserialize;
use serde_aux::field_attributes::deserialize_number_from_string;
use tracing_log::log::LevelFilter;

#[derive(Deserialize)]
pub struct Settings {
    pub database: DatabaseSettings,
    pub application: ApplicationSettings,
}

#[derive(Deserialize)]
pub struct ApplicationSettings {
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub port: u16,
    pub host: String,
}

#[derive(Deserialize)]
pub struct DatabaseSettings {
    pub username: String,
    pub password: SecretString,
    pub port: u16,
    pub host: String,
    pub database_name: String,
    pub require_ssl: bool,
}

pub fn get_configuration() -> Result<Settings, config::ConfigError> {
    let base_path = std::env::current_dir().expect("Failed to determine the current directory");
    let configuration_directory = base_path.join("configuration");

    let environment: Environment = std::env::var("APP_ENVIRONMENT")
        .unwrap_or_else(|_| "local".into())
        .try_into()
        .expect("Failed to parse APP_ENVIRONMENT.");

    let environment_filename = format!("{}.yaml", environment.as_str());

    let settings = config::Config::builder()
        .add_source(config::File::from(
            configuration_directory.join("base.yaml"),
        ))
        .add_source(config::File::from(
            configuration_directory.join(environment_filename),
        ))
        .add_source(
            config::Environment::with_prefix("APP")
                .prefix_separator("_")
                .separator("__"),
        )
        .build()?;

    settings.try_deserialize::<Settings>()
}

impl DatabaseSettings {
    /// 等价于 PgConnectOptions::with_db
    pub fn with_db(&self) -> ConnectOptions {
        let mut url = self.base_mysql_url();

        // 🔑 等价于 `.database(&self.database_name)`
        url.set_path(&format!("/{}", self.database_name));

        let mut opt = ConnectOptions::new(url.to_string());

        // 🔑 等价于 `.log_statements(Trace)`
        opt.sqlx_logging(true)
            .sqlx_logging_level(LevelFilter::Trace);

        opt
    }

    /// 等价于 PgConnectOptions::without_db
    pub fn without_db(&self) -> ConnectOptions {
        let url = self.base_mysql_url();

        let mut opt = ConnectOptions::new(url.to_string());

        opt.sqlx_logging(true)
            .sqlx_logging_level(LevelFilter::Trace);

        opt
    }

    fn base_mysql_url(&self) -> Url {
        let mut url =
            Url::parse(&format!("mysql://{}", self.host)).expect("Invalid base MySQL URL");

        url.set_username(&self.username)
            .expect("Invalid database username");

        url.set_password(Some(self.password.expose_secret()))
            .expect("Invalid database password");

        url.set_port(Some(self.port))
            .expect("Invalid database port");

        {
            let mut pairs = url.query_pairs_mut();

            if self.require_ssl {
                pairs.append_pair("ssl-mode", "REQUIRED");
            } else {
                pairs.append_pair("ssl-mode", "PREFERRED");
            }

            pairs.append_pair("charset", "utf8mb4");
        }

        url
    }
}

pub enum Environment {
    Local,
    Production,
}

impl Environment {
    pub fn as_str(&self) -> &'static str {
        match self {
            Environment::Local => "local",
            Environment::Production => "production",
        }
    }
}
impl TryFrom<String> for Environment {
    type Error = String;
    fn try_from(s: String) -> Result<Self, Self::Error> {
        match s.to_lowercase().as_str() {
            "local" => Ok(Self::Local),
            "production" => Ok(Self::Production),
            other => Err(format!(
                "{} is not a supported environment. \
Use either `local` or `production`.",
                other
            )),
        }
    }
}
