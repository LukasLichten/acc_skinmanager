use std::{path::{PathBuf, Path}, fs};

use json::{JsonValue, stringify_pretty};


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
pub const ACC_CAR_FOLDER_NAME: &str = "Cars";
pub const ACC_LIVERY_FOLDER_NAME: &str = "Liveries";

pub const DATE_FORMAT_STR: &str = "%Y.%m.%d";

pub const FILE_ENDING: &str = "json";

pub fn get_acc_folder() -> PathBuf {
    let mut root_path = dirs::document_dir().unwrap_or_default();
    root_path.push(ACC_ROOT_FOLDER_NAME);

    if !root_path.exists() {
        let user_dir = root_path.parent();

        if user_dir.is_none() {
            //We are in deep trouble
            panic!("Unable to find user path, therefore not able to access ACC folder");
        }

        //Seems ACC folder does not exist, lets generate
        let mut builder = PathBuf::from(user_dir.unwrap());

        builder.push(ACC_ROOT_FOLDER_NAME);
    }

    root_path
}

fn get_config_file(foldername: &str, filename: &str) -> Option<(PathBuf, JsonValue)> {
    let mut folder = get_acc_folder();

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

pub fn read_json(file: &Path) -> json::Result<JsonValue>{
    
    if let Ok(read) = fs::read_to_string(file) {
        return json::parse(read.as_str());
    }

    Err(json::Error::WrongType("File System Error".to_string()))
}

pub fn write_json(file: &Path, data: JsonValue) -> Result<(), std::io::Error> {
    fs::write(file, stringify_pretty(data, 4))
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