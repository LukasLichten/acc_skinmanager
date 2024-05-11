use std::path::PathBuf;

use json::JsonValue;
use serde::{Serialize, Deserialize};

use crate::State;

use super::SafeRead;



pub const ACC_CONFIG_FOLDER_NAME: &str = "Config";

enum ConfigName {
    MenuSettings,

}

fn get_config_file(filename: ConfigName, state: &State) -> Option<(PathBuf, JsonValue)> {
    let name = match filename {
        ConfigName::MenuSettings => "menuSettings"
    };

    return super::get_config_file(state, ACC_CONFIG_FOLDER_NAME, name);
}

pub fn set_dds_generation(mode_state: bool, state: &State) -> Option<bool> {
    if let Some((path, mut content)) = get_config_file(ConfigName::MenuSettings, state){
        if let Some(old_state) = content.get("texDDS") {
            if let Some(old_state) = old_state.as_i32() {
                let state_i = match mode_state { true => 1, false => 0 };
                
                if old_state != state_i {
                    //updating
                    if content.set("texDDS", state_i.into()) {
                        if super::write_json(path.as_path(), content).is_ok() {
                            return Some(old_state ==  1);
                        }
                    }
                } else {
                    return Some(old_state ==  1);
                }
            }
        }
    }

    None
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct GraphicSettings {
    pub resolution: (u32, u32),
    pub fullscreen: bool
} 

pub fn set_graphic_settings(settings: GraphicSettings, state: &State) -> Option<GraphicSettings> {
    if let Some((path, mut content)) = get_config_file(ConfigName::MenuSettings, state){
        if let Some(graphic) = content.get("graphicOptions") {
            let mut graphic_new = graphic.clone();
            
            let full = if let Some(fullscreen) = graphic.get("useFullscreen") {
                if let Some(fullscreen) = fullscreen.as_bool() {
                    if settings.fullscreen != fullscreen {
                        graphic_new.set("useFullscreen", settings.fullscreen.into());
                    }

                    fullscreen
                } else {
                    return None;
                }
            } else {
                return None;
            };

            let res = if let Some(resolution) = graphic.get("resolution") {
                if let (Some(old_x), Some(old_y))  = (resolution.get("x"), resolution.get("y")) {
                    if let (Some(old_x), Some(old_y))  = (old_x.as_u32(), old_y.as_u32()) {
                        if settings.resolution != (old_x, old_y) {
                            let mut new_resultion = resolution.clone();
                            new_resultion.set("x", settings.resolution.0.into());
                            new_resultion.set("y", settings.resolution.1.into());

                            graphic_new.set("resolution", new_resultion);
                        }

                        (old_x, old_y)
                    } else {
                        return None;
                    }
                } else {
                    return None;
                }
            } else {
                return None;
            };

            let old_settings = GraphicSettings { resolution: res, fullscreen: full };
            if old_settings != settings {
                content.set("graphicOptions", graphic_new);
                if super::write_json(path.as_path(), content).is_ok() {
                    return Some(old_settings);
                }
            } else {
                return Some(old_settings);
            }
        }
    }
    None
}


// "audio":
// {
//     "main": 0.64999997615814209,
//     "engineExt": 0.94999998807907104,
//     "engineInt": 0.79999995231628418,
//     "wheel": 1,
//     "wind": 1,
//     "environment": 1,
//     "damage": 1,
//     "comms": 1,
//     "startingComms": 1,
//     "spotter": true,
//     "music": 0.79999995231628418,
//     "opponent": 0.79999995231628418,
//     "gui": 1,
//     "video": 1,
//     "bodywork": 0.89999997615814209,
//     "driverAudio":
//     {
//         "index": 255,
//         "name": "System default driver"
//     }
// },

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct AudioSettings {
    pub master: f64,
    pub music: f64
} 

pub fn set_audio_settings(settings: AudioSettings, state: &State) -> Option<AudioSettings> {
    if let Some((path, mut content)) = get_config_file(ConfigName::MenuSettings, state){
        if let Some(audio) = content.get("audio") {
            let mut audio_new = audio.clone();

            if let (Some(old_master), Some(old_music)) = (audio.get("main"),audio.get("music")) {
                if let (Some(old_master), Some(old_music)) = (old_master.as_f64(), old_music.as_f64()) {
                    if old_master != settings.master {
                        audio_new.set("main", settings.master.into());
                    }

                    if old_music != settings.music {
                        audio_new.set("music", settings.music.into());
                    }

                    let old_settings = AudioSettings {master: old_master, music: old_music};
                    if old_settings != settings {
                        content.set("audio", audio_new);
                        if super::write_json(path.as_path(), content).is_ok() {
                            return Some(old_settings);
                        }
                    } else {
                        return Some(old_settings);
                    }
                }
            }
        }
    }

    None
}

// menuSettings.json:
//  "multiplayerCarGroupSelection":
// 	{
// 		"FREE_FOR_ALL": "76-230226-102925.json",
// 		"GT3": "140-230514-202639.json",
// 		"GT4": "#140_TeamIrisFlatout_Mas.json",
// 		"GTC": "981-210204-230832.json",
// 		"TCX": "2-230425-215025.json",
// 		"GT2": "None"
// 	}
//  "mPShowroomCarGroup": "FREE_FOR_ALL",

