use std::path::PathBuf;
use serde::{de::DeserializeOwned, Serialize};

pub fn config_dir() -> Option<PathBuf> {
    let dir = dirs::config_dir()?.join("RFileMaster");
    std::fs::create_dir_all(&dir).ok()?;
    Some(dir)
}

pub fn save_json<T: Serialize + ?Sized>(filename: &str, value: &T) -> Result<(), String> {
    let dir = config_dir().ok_or("Could not resolve config directory")?;
    let path = dir.join(filename);
    let json = serde_json::to_string_pretty(value).map_err(|e| e.to_string())?;
    std::fs::write(path, json).map_err(|e| e.to_string())
}

pub fn load_json<T: DeserializeOwned>(filename: &str) -> Option<T> {
    let dir = config_dir()?;
    let path = dir.join(filename);
    let text = std::fs::read_to_string(path).ok()?;
    serde_json::from_str(&text).ok()
}