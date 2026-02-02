use secrecy::{ExposeSecret, SecretString};







#[derive(serde::Deserialize)]
pub struct Settings {
    pub database: DatabaseSettings,
    pub application_port: u16,
}
#[derive(serde::Deserialize)]
pub struct DatabaseSettings {
    pub username: String,
    pub password: SecretString,
    pub port: u16,
    pub host: String,
    pub database_name: String,
}

pub fn get_configuration() -> Result<Settings, config::ConfigError> {
    // Initialise our configuration reader
    let settings = config::Config::builder()
        // Add configuration values from a file named `configuration.yaml`.
        .add_source(config::File::new(
            "configuration.yaml",
            config::FileFormat::Yaml,
        ))
        .build()?;
    // Try to convert the configuration values it read into
    // our Settings type
    settings.try_deserialize::<Settings>()
}

impl DatabaseSettings {
    /// 生成MySQL连接字符串（用于SeaORM连接）
    pub fn connection_string(&self) -> SecretString {
        // MySQL标准格式：mysql://用户名:密码@主机:端口/数据库名
        format!(
            "mysql://{}:{}@{}:{}/{}",
            self.username, self.password.expose_secret(), self.host, self.port, self.database_name
        ).into()
    }

    /// 生成不带数据库名的连接字符串（用于创建数据库）
    pub fn connection_string_without_db(&self) -> SecretString {
        format!(
            "mysql://{}:{}@{}:{}",
            self.username, self.password.expose_secret(), self.host, self.port
        ).into()
    }
}
