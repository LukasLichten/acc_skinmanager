use std::{path::PathBuf, fs};

use serde::{Serialize, Deserialize};

use crate::State;

use super::menu_changer::{GraphicSettings, AudioSettings, self};

pub const ACC_APP_FOLDER_NAME: &str = "Apps/Skinmanager";
pub const SETTINGS_FILE: &str = "settings.json";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settings {
    livery_mode_settings: MenuSettings,
    backup_settings: Option<MenuSettings>
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MenuSettings {
    dds_generation: bool,
    graphic: GraphicSettings,
    audio: AudioSettings
}

enum FileType {
    MainSettings
}

fn get_file_path(file: FileType, state: &State) -> Option<PathBuf> {
    let mut folder = state.root_folder.clone();

    folder.push(ACC_APP_FOLDER_NAME);

    if folder.exists() {
        folder.push(match file {
            FileType::MainSettings => SETTINGS_FILE
        });

        if folder.exists() {
            return Some(folder);
        }
    }

    None
}

pub fn get_settings(state: &State) -> Option<Settings> {
    if let Some(path) = get_file_path(FileType::MainSettings, state) {
        if let Ok(data) = fs::read_to_string(path.as_path()) {
            if let Ok(settings) = serde_json::from_str(data.as_str()) {
                return Some(settings);
            }
        }
    } else {
        // Defaults don't exist, so lets generate those
        if !generate(state) {
            return None;
        }
        return get_settings(state);
    }

    None
}

pub fn write_settings(settings: Settings, state: &State) -> bool {
    if let Some(path) = get_file_path(FileType::MainSettings, state) {
        if let Ok(data) = serde_json::to_string_pretty(&settings) {
            return fs::write(path.as_path(), data).is_ok();
        }
    } else {
        // Defaults don't exist, so lets generate those
        if !generate(state) {
            return false;
        }
        return write_settings(settings, state);
    }

    false
}

pub fn generate(state: &State) -> bool {
    let mut folder = state.root_folder.clone();

    folder.push(ACC_APP_FOLDER_NAME);

    if fs::create_dir_all(folder.as_path()).is_err() {
        return false;
    }

    // Settings file
    let mut sett_file = folder.clone();
    sett_file.push(SETTINGS_FILE);
    
    let default_settings = Settings {
        livery_mode_settings: MenuSettings {
            dds_generation: false,
            graphic: GraphicSettings { resolution: (1600, 900), fullscreen: false },
            audio: AudioSettings { master: 0.5, music: 0.0 }
        },
        backup_settings: None
    };

    if let Ok(data) = serde_json::to_string_pretty(&default_settings) {
        if fs::write(sett_file, data).is_err() {
            return false;
        }
    }

    true
}

impl Settings {
    pub fn switch_liverymode(&mut self, state: &State) -> Option<bool> {
        if let Some(backup) = self.backup_settings.clone() {
            // A backup exists, therefore this is to exit liverymode
            if menu_changer::set_dds_generation(backup.dds_generation, state).is_some() && menu_changer::set_graphic_settings(backup.graphic, state).is_some()
                    && menu_changer::set_audio_settings(backup.audio, state).is_some() {
                        self.backup_settings = None;
                        return Some(false);
            }
        } else {
            //We are entering liverymode
            let res = Some(MenuSettings {
                dds_generation: menu_changer::set_dds_generation(self.livery_mode_settings.dds_generation, state)?,
                audio: menu_changer::set_audio_settings(self.livery_mode_settings.audio.clone(), state)?,
                graphic: menu_changer::set_graphic_settings(self.livery_mode_settings.graphic.clone(), state)?
            });

            if let Some(old_settings) = res {
                self.backup_settings = Some(old_settings);
                return Some(true)
            }        
        }
        
        None
    }

    pub fn is_in_liverymode(&self) -> bool {
        self.backup_settings.is_some()
    }
}
