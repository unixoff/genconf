use crate::config::{Config, ConfigItem};

use std::{
    collections::HashSet,
    fs, io,
    path::{Path, PathBuf},
};

#[derive(Debug, PartialEq, Eq)]
pub enum WriteStatus {
    Created,
    Updated,
    Skipped,
}

pub fn get_target_path(config: &Config, file_name: &str) -> PathBuf {
    Path::new(&config.path_to_target).join(file_name)
}

pub fn get_file_name(config: &Config, item: &ConfigItem) -> String {
    format!("{}.{}", item.name, config.target_extension)
}

pub fn write_if_changed(path: &Path, new_content: &str) -> io::Result<WriteStatus> {
    match fs::read_to_string(path) {
        Ok(existing_content) => {
            if existing_content == new_content {
                Ok(WriteStatus::Skipped)
            } else {
                fs::write(path, new_content)?;
                Ok(WriteStatus::Updated)
            }
        }
        Err(err) if err.kind() == io::ErrorKind::NotFound => {
            fs::write(path, new_content)?;
            Ok(WriteStatus::Created)
        }
        Err(err) => Err(err),
    }
}

pub fn clean_target(target_path: &str, managed_files: &[String]) -> io::Result<Vec<PathBuf>> {
    let managed_files: HashSet<&str> = managed_files.iter().map(String::as_str).collect();
    let mut removed_files = Vec::new();

    for entry in fs::read_dir(target_path)? {
        let entry = entry?;
        let path = entry.path();

        if !entry.file_type()?.is_file() {
            continue;
        }

        let Some(file_name) = path.file_name().and_then(|name| name.to_str()) else {
            continue;
        };

        if managed_files.contains(file_name) {
            continue;
        }

        fs::remove_file(&path)?;
        removed_files.push(path);
    }

    Ok(removed_files)
}

#[cfg(test)]
mod tests {
    use super::{WriteStatus, clean_target, get_file_name, get_target_path, write_if_changed};
    use crate::config::{Config, ConfigItem, ValuesMap};
    use std::{
        fs,
        path::PathBuf,
        time::{SystemTime, UNIX_EPOCH},
    };

    fn make_config(path_to_target: &str, target_extension: &str) -> Config {
        Config {
            path_to_target: path_to_target.to_string(),
            path_to_template: String::new(),
            template: String::new(),
            target_extension: target_extension.to_string(),
            clean_target: false,
            values: ValuesMap::new(),
            configs: Vec::new(),
        }
    }

    fn make_item(name: &str) -> ConfigItem {
        ConfigItem {
            name: name.to_string(),
            template: None,
            values: ValuesMap::new(),
        }
    }

    fn temp_file_path(test_name: &str) -> PathBuf {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();

        std::env::temp_dir().join(format!("genconf-{test_name}-{timestamp}.conf"))
    }

    fn temp_dir_path(test_name: &str) -> PathBuf {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();

        std::env::temp_dir().join(format!("genconf-{test_name}-{timestamp}"))
    }

    #[test]
    fn builds_target_path_from_name_and_extension() {
        let config = make_config("/tmp/genconf", "conf");
        let item = make_item("worker-default");
        let file_name = get_file_name(&config, &item);

        let path = get_target_path(&config, &file_name);

        assert_eq!(path, PathBuf::from("/tmp/genconf/worker-default.conf"));
    }

    #[test]
    fn returns_created_when_file_does_not_exist() {
        let path = temp_file_path("created");

        let status = write_if_changed(&path, "first content").unwrap();
        let written = fs::read_to_string(&path).unwrap();

        assert_eq!(status, WriteStatus::Created);
        assert_eq!(written, "first content");

        fs::remove_file(path).unwrap();
    }

    #[test]
    fn returns_skipped_for_same_content_and_updated_for_new_content() {
        let path = temp_file_path("updated");
        fs::write(&path, "initial content").unwrap();

        let skipped = write_if_changed(&path, "initial content").unwrap();
        let updated = write_if_changed(&path, "new content").unwrap();
        let written = fs::read_to_string(&path).unwrap();

        assert_eq!(skipped, WriteStatus::Skipped);
        assert_eq!(updated, WriteStatus::Updated);
        assert_eq!(written, "new content");

        fs::remove_file(path).unwrap();
    }

    #[test]
    fn clean_target_removes_files_not_in_managed_list() {
        let dir = temp_dir_path("clean-target");
        fs::create_dir(&dir).unwrap();

        let keep = dir.join("keep.conf");
        let remove = dir.join("remove.conf");
        let nested_dir = dir.join("nested");

        fs::write(&keep, "keep").unwrap();
        fs::write(&remove, "remove").unwrap();
        fs::create_dir(&nested_dir).unwrap();
        fs::write(nested_dir.join("inner.conf"), "nested").unwrap();

        let removed = clean_target(dir.to_str().unwrap(), &[String::from("keep.conf")]).unwrap();

        assert_eq!(removed, vec![remove.clone()]);
        assert!(keep.exists());
        assert!(!remove.exists());
        assert!(nested_dir.exists());

        fs::remove_file(keep).unwrap();
        fs::remove_file(nested_dir.join("inner.conf")).unwrap();
        fs::remove_dir(nested_dir).unwrap();
        fs::remove_dir(dir).unwrap();
    }

    #[test]
    fn clean_target_keeps_all_managed_files() {
        let dir = temp_dir_path("clean-target-keep");
        fs::create_dir(&dir).unwrap();

        let first = dir.join("first.conf");
        let second = dir.join("second.conf");

        fs::write(&first, "first").unwrap();
        fs::write(&second, "second").unwrap();

        let removed = clean_target(
            dir.to_str().unwrap(),
            &[String::from("first.conf"), String::from("second.conf")],
        )
        .unwrap();

        assert!(removed.is_empty());
        assert!(first.exists());
        assert!(second.exists());

        fs::remove_file(first).unwrap();
        fs::remove_file(second).unwrap();
        fs::remove_dir(dir).unwrap();
    }
}
