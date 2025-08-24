use serde::Deserialize;

#[derive(Deserialize, Debug, Clone)]
pub struct Config {
    pub frappe_bench_dir: String,
    pub app_relative_path: String,

    #[allow(dead_code)]
    pub app_name: String,

    #[serde(default)]
    pub app_absolute_path: String,
}

impl Config {
    pub fn from_toml_str(toml_str: &str) -> Result<Config, toml::de::Error> {
        toml::from_str(toml_str)
    }
    pub fn from_file(file_path: &str) -> Result<Config, Box<dyn std::error::Error>> {
        let content = std::fs::read_to_string(file_path)?;
        let mut config = Self::from_toml_str(&content)?;
        config.app_absolute_path =
            format!("{}/{}", config.frappe_bench_dir, config.app_relative_path);
        Ok(config)
    }
}
