use std::path::PathBuf;

use json::JsonValue;

use super::SafeRead;



pub const ACC_CONFIG_FOLDER_NAME: &str = "Config";

enum ConfigName {
    MenuSettings,

}

fn get_config_file(filename: ConfigName) -> Option<(PathBuf, JsonValue)> {
    let mut folder = super::get_acc_folder();

    folder.push(ACC_CONFIG_FOLDER_NAME);
    

    if folder.exists() {
        folder.push(match filename {
            ConfigName::MenuSettings => "menuSettings"
        });
        folder.set_extension(super::FILE_ENDING);

        

        if folder.exists() {
            if let Ok(content) = super::read_json(folder.as_path()) {
                return Some((folder, content));
            }
        }
    }

    None
}

pub fn set_dds_generation(state: bool) -> bool {
    if let Some((path, mut content)) = get_config_file(ConfigName::MenuSettings){
        if let Some(old_state) = content.get("texDDS") {
            if let Some(old_state) = old_state.as_i32() {
                let state = match state { true => 1, false => 0 };
                if old_state != state {
                    //updating
                    if content.set("texDDS", state.into()) {
                        return super::write_json(path.as_path(), content).is_ok();
                    }
                }
            }
        }
    }

    false
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

// menuSettings.json:
// "graphicOptions":
// {
//     "lastLoadedPresetName": "",
//     "resolution":
//     {
//         "x": 1600,
//         "y": 900
//     },
//     "useFullscreen": false,

// menuSettings.json
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