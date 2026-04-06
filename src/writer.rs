use crate::config::{Config, ConfigItem};

use std::{
    fs, io,
    path::{Path, PathBuf},
};

#[derive(Debug, PartialEq, Eq)]
pub enum WriteStatus {
    Created,
    Updated,
    Skipped,
}

pub fn get_target_path(config: &Config, item: &ConfigItem) -> PathBuf {
    let file_name = format!("{}.{}", item.name, config.target_extension);
    Path::new(&config.path_to_target).join(file_name)
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
