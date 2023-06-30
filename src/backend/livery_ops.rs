use std::{path::PathBuf, fs, io::{Cursor, self}};

use super::SafeRead;

pub const ACC_TEMP_FOLDER:&str = "temp";

pub const ACC_CUSTOMS_FOLDER_NAME: &str = "Customs";
pub const ACC_CAR_FOLDER_NAME: &str = "Cars";
pub const ACC_LIVERY_FOLDER_NAME: &str = "Liveries";

pub fn get_temp_folder() -> Option<PathBuf> {
    let mut folder = super::get_acc_folder();

    folder.push(ACC_TEMP_FOLDER);
    if folder.exists() {
        return None;
    }

    if fs::create_dir_all(folder.as_path()).is_ok() {
        return Some(folder);
    }

    None
}

#[derive(Debug, Clone, PartialEq)]
pub enum CustomFolder {
    Cars,
    Liveries(String)
}

impl ToString for CustomFolder {
    fn to_string(&self) -> String {
        if let CustomFolder::Liveries(folder) = self {
            format!("{}/{}", ACC_LIVERY_FOLDER_NAME, folder)
        } else {
            format!("{}", ACC_CAR_FOLDER_NAME)
        }
    }
}

#[derive(Debug, Clone)]
pub struct ZipLiveryContent {
    pub upper: CustomFolder,
    pub name: String,
    pub file: Vec<u8>
}

impl ZipLiveryContent {
    pub fn get_target(& self) -> PathBuf {
        let mut file = super::get_acc_folder();

        file.push(ACC_CUSTOMS_FOLDER_NAME);
        file.push(self.upper.to_string());
        file.push(&self.name);

        file
    }
}

pub fn get_zip_content(zip_file: &PathBuf) -> Option<Vec<ZipLiveryContent>> {
    if let Ok(stream) = fs::read(&zip_file) {
        let stream = Cursor::new(stream);

        let mut content = Vec::<ZipLiveryContent>::new();
        // Reading the zip
        if let Ok(zip_content) = zip::ZipArchive::new(&mut stream.clone()) {
            let iter = zip_content.file_names();

            // Reading the file
            for item in iter {
                let mut data = Vec::<u8>::new();
                let mut internal_path = PathBuf::from(item);
                if zip_extensions::read::zip_extract_file_to_memory(zip_file, &internal_path, &mut data).is_ok() {

                    // 
                    let name = get_filename(&internal_path);
                    let upper = if internal_path.pop() && internal_path.file_name().is_some() {
                        let folder_name = get_filename(&internal_path);

                        match folder_name.to_lowercase().as_str() {
                            "cars" => CustomFolder::Cars,
                            _ => CustomFolder::Liveries(folder_name)
                        }
                    } else {
                        let mut origin_file = zip_file.clone();
                        origin_file.set_extension("");
                        CustomFolder::Liveries(get_filename(&origin_file))
                    };


                    content.push(ZipLiveryContent { upper, name, file: data });
                }
            }

            return Some(content);
        }
    }

    None
}

fn get_filename(path: &PathBuf) -> String {
    path.file_name().expect("there must be at least a file name").to_str().expect("osstr to str should always work").to_string()
}

#[derive(Debug,Clone)]
pub struct Livery {
    pub livery_folder: Option<String>,
    pub car_json: Option<ZipLiveryContent>,
    pub livery_files: Vec<ZipLiveryContent>
}

impl Livery {
    pub fn check_if_conflict(& self) -> Option<(bool, bool)> {
        let mut conflict = (false, false);

        if let Some(car) = &self.car_json {
            if car.get_target().exists() {
                conflict = (true, conflict.1);
            }
        }

        if let Some(folder) = self.get_livery_folder() {
            if folder.exists() {
                // we don't have to flag a conflict yet
                let iter = self.livery_files.iter();
                for item in iter {
                    match item.name.to_lowercase().as_str() {
                        "decals.json" => (), // auto generate if you were on the server with someone using this Livery prior to downloading it
                        "sponsors.json" => (), // auto generated
                        "README.txt" => (), // info that is not relevant, although if present probably means relevant files will also be present and have a conflict
                        "awesome.txt" => (), // info that is not relevant
                        _ => {
                            if item.get_target().exists() {
                                conflict = (conflict.0, true);
                            }
                        }
                    }
                }
            }
        }

        if conflict == (false, false) {
            return None;
        }

        Some(conflict)
    }

    fn get_livery_folder(& self) -> Option<PathBuf> {
        let mut folder = super::get_acc_folder();
        folder.push(ACC_CUSTOMS_FOLDER_NAME);
        folder.push(ACC_LIVERY_FOLDER_NAME);
        
        folder.push(self.livery_folder.clone()?); // If no folder is defined this will return none here

        Some(folder)
    }

    pub fn write(&self) -> io::Result<()> {
        // Setting up car.json
        if let Some(car) = &self.car_json {
            fs::write(car.get_target(), car.file.clone())?;
        }

        // Creating the folder if necessary
        if let Some(folder) = self.get_livery_folder() {
            if !folder.exists() {
                fs::create_dir_all(folder)?;
            }
        }

        // Writing the livery files
        let iter = self.livery_files.iter();
        for item in iter {
            fs::write(item.get_target(), item.file.clone())?;
        }

        Ok(())
    }
}

pub fn group_up(mut files: Vec<ZipLiveryContent>) -> Vec<Livery> {
    let mut liveries = Vec::<Livery>::new();

    while let Some(item) = files.pop() {
        if let CustomFolder::Cars = item.upper {
            if let Ok(parsed_json) = super::read_json_from_bytes(item.file.clone()) {
                // We get the livery folder from the car.json, if not found we just add the livery
                if let Some(target_foldername) = parsed_json.get("customSkinName") {
                    if let Some(target_foldername) = target_foldername.as_str() {
                        let target_foldername = target_foldername.to_string();
                        let mut found = false;

                        // We attempt finding a livery group that exists already
                        let mut iter = liveries.iter_mut();
                        while let Some(liver) = iter.next() {
                            if liver.car_json.is_none() { // We only add one, even if this could technically happen
                                if let Some(liver_folder) = liver.livery_folder.clone() {

                                    // Looking for the match
                                    if liver_folder == target_foldername {
                                        liver.car_json = Some(item.clone());
                                        found = true;
                                        break;
                                    }
                                }
                            }
                        }

                        if !found {
                            liveries.push(Livery { livery_folder: Some(target_foldername), car_json: Some(item), livery_files: Vec::<ZipLiveryContent>::new() });
                        }
                    } else {
                        liveries.push(Livery { livery_folder: None, car_json: Some(item), livery_files: Vec::<ZipLiveryContent>::new() });
                    }
                } else {
                    liveries.push(Livery { livery_folder: None, car_json: Some(item), livery_files: Vec::<ZipLiveryContent>::new() });
                };
            }
            
            
        } else if let CustomFolder::Liveries(target_foldername) = item.upper.clone() {
            let mut found = false;
            
            let mut iter = liveries.iter_mut();
            while let Some(liver) = iter.next() {
                if let Some(liver_folder) = liver.livery_folder.clone() {

                    // Looking for the match
                    if liver_folder == target_foldername {
                        liver.livery_files.push(item.clone());
                        found = true;
                        break;
                    }
                }
            }

            if !found {
                let mut list = Vec::<ZipLiveryContent>::new();
                list.push(item);
                liveries.push(Livery { livery_folder: Some(target_foldername), car_json: None, livery_files: list });
            }
            
        }
    }

    liveries
}

// {
//     "carGuid": 0,
//     "teamGuid": 0,
//     "raceNumber": 2,
//     "raceNumberPadding": 0,
//     "auxLightKey": 6,
//     "auxLightColor": 341,
//     "skinTemplateKey": 102,
//     "skinColor1Id": 305,
//     "skinColor2Id": 311,
//     "skinColor3Id": 341,
//     "sponsorId": 0,
//     "skinMaterialType1": 0,
//     "skinMaterialType2": 0,
//     "skinMaterialType3": 0,
//     "rimColor1Id": 305,
//     "rimColor2Id": 341,
//     "rimMaterialType1": 1,
//     "rimMaterialType2": 1,
//     "teamName": "Team Iris",
//     "nationality": 3,
//     "displayName": "",
//     "competitorName": "",
//     "competitorNationality": 3,
//     "teamTemplateKey": 0,
//     "carModelType": 34,
//     "cupCategory": 0,
//     "licenseType": 0,
//     "useEnduranceKit": 1,
//     "customSkinName": "#2_TeamIris_992",
//     "bannerTemplateKey": 2
// }