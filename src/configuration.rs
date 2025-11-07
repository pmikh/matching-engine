use smart_default::SmartDefault;

#[derive(serde::Deserialize, SmartDefault)]
#[serde(default)]
pub struct Settings {
    #[default(Default::default())]
    pub application: AppSettings,
}

#[derive(serde::Deserialize, SmartDefault)]
pub struct AppSettings {
    #[default = "127.0.0.1"]
    pub host: String,
    #[default = 8000]
    pub port: u16,
    #[default = 100_000]
    pub matching_buffer: usize,
}

impl Settings {
    pub fn address(&self) -> String {
        format!("{}:{}", self.application.host, self.application.port)
    }
}

pub fn get_configuration() -> Result<Settings, config::ConfigError> {
    let settings = config::Config::builder()
        .build()
        .expect("Error when reading config");

    settings.try_deserialize()
}
