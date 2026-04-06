use crate::{
    config::Config,
    render::render_config_item,
    writer::{WriteStatus, get_target_path, write_if_changed},
};

pub fn run(config: &Config) -> Result<(), Box<dyn std::error::Error>> {
    for item in &config.configs {
        let render = render_config_item(config, item)?;
        let target_path = get_target_path(config, item);

        print_status(write_if_changed(&target_path, &render)?, &target_path);
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
