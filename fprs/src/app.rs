use std::path::PathBuf;

pub fn config_dir(app_name: &str) -> PathBuf {
    let mut dir = dirs::config_dir().expect("Could not determine config directory");

    dir.push(app_name);
    dir
}

pub fn db_path(app_name: &str) -> PathBuf {
    let mut path = config_dir(app_name);
    path.push("app.db");
    path
}
