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

#[cfg(test)]
mod tests {
    use super::{apply_values, get_values};
    use crate::config::{Config, ConfigItem, ValuesMap};

    fn string_value(value: &str) -> serde_yaml_ng::Value {
        serde_yaml_ng::Value::String(value.to_string())
    }

    fn bool_value(value: bool) -> serde_yaml_ng::Value {
        serde_yaml_ng::Value::Bool(value)
    }

    fn number_value(value: i64) -> serde_yaml_ng::Value {
        serde_yaml_ng::Value::Number(value.into())
    }

    fn make_config(values: ValuesMap) -> Config {
        Config {
            path_to_target: String::new(),
            path_to_template: String::new(),
            template: "template.conf".to_string(),
            target_extension: "conf".to_string(),
            clean_target: false,
            values,
            configs: Vec::new(),
        }
    }

    fn make_item(name: &str, values: ValuesMap) -> ConfigItem {
        ConfigItem {
            name: name.to_string(),
            template: None,
            values,
        }
    }

    #[test]
    fn applies_plain_variables() {
        let mut values = ValuesMap::new();
        values.insert(
            "command_args".to_string(),
            string_value("queue:consume default"),
        );

        let rendered = apply_values("command={{ command_args }}", &values);

        assert_eq!(rendered, "command=queue:consume default");
    }

    #[test]
    fn merges_shared_and_local_values_with_local_override() {
        let mut shared = ValuesMap::new();
        shared.insert("startsecs".to_string(), number_value(0));
        shared.insert("user".to_string(), string_value("app"));

        let mut local = ValuesMap::new();
        local.insert("startsecs".to_string(), number_value(5));

        let config = make_config(shared);
        let item = make_item("worker", local);

        let values = get_values(&config.values, &item);

        assert_eq!(values.get("user"), Some(&string_value("app")));
        assert_eq!(values.get("startsecs"), Some(&number_value(5)));
    }

    #[test]
    fn injects_name_from_config_item() {
        let config = make_config(ValuesMap::new());
        let item = make_item("worker-default", ValuesMap::new());

        let values = get_values(&config.values, &item);
        let rendered = apply_values("[program:{{ name }}]", &values);

        assert_eq!(rendered, "[program:worker-default]");
    }

    #[test]
    fn applies_lower_filter() {
        let mut values = ValuesMap::new();
        values.insert("autostart".to_string(), bool_value(true));

        let rendered = apply_values("autostart={{ autostart | lower }}", &values);

        assert_eq!(rendered, "autostart=true");
    }

    #[test]
    fn leaves_missing_values_unchanged() {
        let values = ValuesMap::new();

        let rendered = apply_values("user={{ missing }}", &values);

        assert_eq!(rendered, "user={{ missing }}");
    }
}
