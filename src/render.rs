use crate::config::{Config, ConfigItem, ValuesMap, value_as_string};
use std::{fs, path::Path};

pub fn render_config_item(
    config: &Config,
    item: &ConfigItem,
) -> Result<String, Box<dyn std::error::Error>> {
    let template_name = item.template.as_deref().unwrap_or(&config.template);

    let template_path = Path::new(&config.path_to_template).join(template_name);
    let template_content = fs::read_to_string(&template_path)?;

    let values = get_values(&config.values, item);
    let render = apply_values(&template_content, &values);

    Ok(render)
}

fn get_values(default_values: &ValuesMap, item: &ConfigItem) -> ValuesMap {
    let mut values = default_values.clone();

    values.insert(
        "name".to_string(),
        serde_yaml_ng::Value::String(item.name.clone()),
    );

    for (key, val) in &item.values {
        values.insert(key.clone(), val.clone());
    }

    values
}

fn apply_values(template_content: &str, values: &ValuesMap) -> String {
    let regexp =
        regex::Regex::new(r"\{\{\s*([a-zA-Z0-9_-]+)(?:\s*\|\s*([a-zA-Z0-9_-]+))?\s*\}\}").unwrap();

    regexp
        .replace_all(template_content, |caps: &regex::Captures| {
            let key = &caps[1];
            let filter = caps.get(2).map(|m| m.as_str());

            match values.get(key) {
                Some(v) => {
                    apply_filter(value_as_string(v), filter).unwrap_or_else(|| caps[0].to_string())
                }
                None => caps[0].to_string(),
            }
        })
        .to_string()
}

fn apply_filter(value: String, filter: Option<&str>) -> Option<String> {
    match filter {
        None => Some(value),
        Some("lower") => Some(value.to_lowercase()),
        Some("upper") => Some(value.to_uppercase()),
        Some(_) => None,
    }
}
