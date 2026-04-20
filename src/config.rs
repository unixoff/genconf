use serde::Deserialize;
use std::collections::HashMap;
use std::path::PathBuf;
use std::{fs, io};

pub type ValuesMap = HashMap<String, serde_yaml_ng::Value>;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Config {
    pub path_to_target: String,
    pub path_to_template: String,
    pub template: String,
    pub target_extension: String,
    #[serde(default)]
    pub clean_target: bool,
    pub values: ValuesMap,
    pub configs: Vec<ConfigItem>,
}

#[derive(Debug, Deserialize)]
pub struct ConfigItem {
    pub name: String,

    #[serde(default)]
    pub template: Option<String>,

    #[serde(default)]
    pub values: ValuesMap,
}

pub fn load_config(config_paths: &[PathBuf]) -> Result<Config, Box<dyn std::error::Error>> {
    let paths = if config_paths.is_empty() {
        vec![PathBuf::from("values.yaml")]
    } else {
        config_paths.to_vec()
    };

    let mut merged = serde_yaml_ng::Value::Mapping(Default::default());

    for path in paths {
        let content = fs::read_to_string(&path).map_err(|err| {
            io::Error::new(
                err.kind(),
                format!("failed to read config '{}': {err}", path.display()),
            )
        })?;
        let value: serde_yaml_ng::Value = serde_yaml_ng::from_str(&content).map_err(|err| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                format!("failed to parse config '{}': {err}", path.display()),
            )
        })?;
        merge_yaml(&mut merged, value);
    }

    Ok(serde_yaml_ng::from_value(merged)?)
}

fn merge_yaml(base: &mut serde_yaml_ng::Value, override_value: serde_yaml_ng::Value) {
    match (base, override_value) {
        (
            serde_yaml_ng::Value::Mapping(base_mapping),
            serde_yaml_ng::Value::Mapping(override_mapping),
        ) => {
            for (key, value) in override_mapping {
                match base_mapping.get_mut(&key) {
                    Some(base_value) => merge_yaml(base_value, value),
                    None => {
                        base_mapping.insert(key, value);
                    }
                }
            }
        }
        (base_value, override_value) => {
            *base_value = override_value;
        }
    }
}

pub fn value_as_string(value: &serde_yaml_ng::Value) -> String {
    match value {
        serde_yaml_ng::Value::String(v) => v.clone(),
        serde_yaml_ng::Value::Bool(v) => v.to_string(),
        serde_yaml_ng::Value::Number(v) => v.to_string(),
        serde_yaml_ng::Value::Null => String::new(),
        other => serde_yaml_ng::to_string(other)
            .unwrap_or_default()
            .trim()
            .to_string(),
    }
}
