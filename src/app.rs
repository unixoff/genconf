use crate::{
    config::Config,
    render::render_config_item,
    writer::{WriteStatus, clean_target, get_file_name, get_target_path, write_if_changed},
};

pub fn run(config: &Config) -> Result<(), Box<dyn std::error::Error>> {
    let mut managed_files = Vec::with_capacity(config.configs.len());

    for item in &config.configs {
        let render = render_config_item(config, item)?;
        let file_name = get_file_name(config, item);
        let target_path = get_target_path(config, &file_name);

        print_status(write_if_changed(&target_path, &render)?, &target_path);
        managed_files.push(file_name);
    }

    if config.clean_target {
        for path in clean_target(&config.path_to_target, &managed_files)? {
            println!("removed: {}", path.display());
        }
    }

    Ok(())
}

fn print_status(status: WriteStatus, path: &std::path::Path) {
    match status {
        WriteStatus::Created => println!("created: {}", path.display()),
        WriteStatus::Updated => println!("updated: {}", path.display()),
        WriteStatus::Skipped => println!("skip: no changes: {}", path.display()),
    }
}
