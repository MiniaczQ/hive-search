use std::{fs::{File, OpenOptions}, net::SocketAddr, path::Path, str::FromStr, sync::Arc};

use serde::{Deserialize, Serialize};

use druid::*;

use crate::sync::PauseToken;

use super::widgets::timer::TimerData;

/// Path of the configuration file.
const SETTINGS_PATH: &str = "config";

/// Relative path, from .minecraft to where servers NBT is stored.
pub const SERVERS: &str = r"\servers.dat";

/// Relative path, from .minecraft to where latest client log is stored.
pub const LATEST_LOG: &str = r"\logs\latest.log";

/// All UI states
#[derive(Data, PartialEq, Clone, Copy)]
pub enum State {
    Config,
    Host,
    Client,
}

/// Stores settings required for the Hive Search
#[derive(Clone, Data, Lens, Serialize, Deserialize, PartialEq)]
pub struct Settings {
    pub minecraft_path: String,
    pub server_addr: String,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            minecraft_path: String::new(),
            server_addr: String::new(),
        }
    }
}

/// Stores all global mutable application data.
#[derive(Clone, Data, Lens)]
pub struct AppData {
    pub state: State,
    pub settings: Settings,
    pub stop_token: Option<Arc<PauseToken>>,
    pub pause_token: Option<Arc<PauseToken>>,
    pub timer: TimerData,
    pub void: String,
}

impl Default for AppData {
    fn default() -> Self {
        Self {
            state: State::Config,
            settings: Settings::default(),
            stop_token: None,
            pause_token: None,
            timer: TimerData::default(),
            void: "a".to_owned(),
        }
    }
}

/// All possible validation errors.
pub enum SettingsValidationError {
    InvalidAddr,
    MissingLogs,
    MissingServers,
}

/// Confirms validity of the server address.
/// Confirms existence of latest.log and servers.dat files.
///
/// DOES NOT confirm existence of server under the address.
///
/// DOES NOT confirm validity of latest.log and servers.dat files.
pub fn validate_settings(settings: &Settings) -> Result<(), SettingsValidationError> {
    if let Err(_) = SocketAddr::from_str(&settings.server_addr) {
        return Err(SettingsValidationError::InvalidAddr);
    }
    let logs = settings.minecraft_path.clone() + LATEST_LOG;
    if !Path::new(&logs).exists() {
        return Err(SettingsValidationError::MissingLogs);
    }
    let servers = settings.minecraft_path.clone() + SERVERS;
    if !Path::new(&servers).exists() {
        return Err(SettingsValidationError::MissingServers);
    }
    Ok(())
}

/// Saves settings in the SETTINGS_PATH.
pub fn save_settings(settings: &Settings) {
    if let Ok(file) = OpenOptions::new()
        .write(true)
        .truncate(true)
        .create(true)
        .open(SETTINGS_PATH)
    {
        bincode::serialize_into(file, settings).expect("Failed to serialize settings into a file.");
        return
    }
}

/// Loads settings from SETTINGS_PATH.
/// If failed at any step, returns default settings.
pub fn load_settings() -> Settings {
    if let Ok(file) = File::open(SETTINGS_PATH) {
        if let Ok(settings) = bincode::deserialize_from::<File, Settings>(file) {
            return settings;
        }
    }
    Settings::default()
}
