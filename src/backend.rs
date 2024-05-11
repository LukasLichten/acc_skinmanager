use std::{path::{PathBuf, Path}, fs};

use json::{JsonValue, stringify_pretty};
use proton_finder::GameDrive;

use crate::State;

pub mod livery_ops;
pub mod menu_changer;
pub mod app_data;

//Folder Strcuture in ACC:
//User/Documents
//  ACC
//      Custom
//          Config
//              
//          Cars
//              [car.json]
//          Liveries
//              [CustomLivery]
//                  [decals.png] 
//                  [decals_0.dds] // Generated when loading Showroom, useless
//                  [decals_1.dds] // Generated when ingame, ultra useful
//                  [decals.json] // Always Present
//                  [sponsors.png]
//                  [sponsors_0.dds]
//                  [sponsors_1.dds]
//                  [sponsors.json] // Always Present
//                  [awesome.txt] // Present if livery was downloaded via awesome-simracing service


pub const ACC_ROOT_FOLDER_NAME: &str = "Assetto Corsa Competizione";

pub const DATE_FORMAT_STR: &str = "%Y.%m.%d";

pub const FILE_ENDING: &str = "json";


pub fn get_acc_folder() -> Result<Option<PathBuf>,Option<PathBuf>> {
    let (game_drive, err) = match proton_finder::get_game_drive(805550) {
        Ok(res) => (res, false),
        Err(res) => (res, true)
    };


    fn internal_pathing(game_drive: Option<GameDrive>) -> Option<PathBuf> {
        let mut root_path = game_drive?.document_dir()?;
        root_path.push(ACC_ROOT_FOLDER_NAME);

        if root_path.is_dir() {
            let mut builder = root_path.clone();

            builder.push(livery_ops::ACC_CUSTOMS_FOLDER_NAME);

            if builder.is_dir() {
                return Some(root_path);
            }
        }

        None
    }
    
    let res = internal_pathing(game_drive);
    match err {
        false => Ok(res),
        true => Err(res)
    }
}

fn get_config_file(state: &State, foldername: &str, filename: &str) -> Option<(PathBuf, JsonValue)> {
    let mut folder = state.root_folder.clone();

    folder.push(foldername);
    

    if folder.exists() {
        folder.push(filename);
        folder.set_extension(FILE_ENDING);

        if folder.exists() {
            if let Ok(content) = read_json(folder.as_path()) {
                return Some((folder, content));
            }
        }
    }

    None
}

pub fn read_json_from_bytes(data: Vec<u8>) -> json::Result<JsonValue> {
    if let Ok(text) = String::from_utf8(data) {
        // We are technically reading utf8 byte strings, so it producess funny results
        // But there is not convenient way of converting Vec<u8> to Vec<u16>, so just running a
        // replace is simpler
        let text = text.replace("\u{0}", "");
        return json::parse(text.as_str());
    }

    Err(json::Error::WrongType("File System Error".to_string()))
}

pub fn read_json(file: &Path) -> json::Result<JsonValue>{
    
    if let Ok(read) = fs::read_to_string(file) {
        return json::parse(read.as_str());
    }

    Err(json::Error::WrongType("File System Error".to_string()))
}

pub fn write_json(file: &Path, data: JsonValue) -> Result<(), std::io::Error> {
    fs::write(file, stringify_pretty(data, 4))
}

pub fn get_filename(path: &PathBuf) -> String {
    path.file_name().expect("there must be at least a file name").to_str().expect("osstr to str should always work").to_string()
}

trait SafeRead {
    fn get<'a>(&'a self, key: &str) -> Option<&'a JsonValue>;
    fn set(&mut self, key: &str, value: JsonValue) -> bool;
}

impl SafeRead for JsonValue {
    fn get<'a>(&'a self, key: &str) -> Option<&'a JsonValue> {
        if self.has_key(key) {
            return Some(&self[key]);
        }

        None
    }

    fn set(&mut self, key: &str, value: JsonValue) -> bool {
        if self.has_key(key) {
            self[key] = value;
            return true;
        }
        if self.insert(key, "").is_ok() {
            self[key] = value;
            return true;
        }

        false
    }
}
