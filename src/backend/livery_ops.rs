use std::{path::PathBuf, fs::{self, File}, io::{Cursor, self, Write}};

use indicatif::{ProgressBar, ProgressStyle};

use crate::State;

use super::{SafeRead, get_filename};

pub const ACC_TEMP_FOLDER:&str = "temp";

pub const ACC_CUSTOMS_FOLDER_NAME: &str = "Customs";
pub const ACC_CAR_FOLDER_NAME: &str = "Cars";
pub const ACC_LIVERY_FOLDER_NAME: &str = "Liveries";

pub fn get_temp_folder(state: &State) -> Option<PathBuf> {
    let mut folder = state.root_folder.clone();

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

#[derive(Debug)]
pub enum Conflict {
    None,
    CarOnly,
    LiveryOnly,
    Both,
    Identical
}

#[derive(Debug, Clone)]
pub struct ZipLiveryContent {
    pub upper: CustomFolder,
    pub name: String,
    pub file: Vec<u8>
}

impl ZipLiveryContent {
    pub fn get_target(& self, state: &State) -> PathBuf {
        let mut file = state.root_folder.clone();

        file.push(ACC_CUSTOMS_FOLDER_NAME);
        file.push(self.upper.to_string());
        file.push(&self.name);

        file
    }

    pub fn is_same_as_target(& self, state: &State) -> bool {
        if let Ok(content) = fs::read(self.get_target(state)) {
            if content.len() != self.file.len() {
                return false;
            }

            // Why run a hash function if we have one in memory already, and have to load the other into memory to hash anyway
            // We can just iterate over all bytes

            for (base, target) in self.file.iter().zip(content.iter()) {
                if base != target {
                    return false;
                }
            }

            return true;
        }

        false
    }

    pub fn get_interal_path(& self) -> String {
        format!("{}/{}", self.upper.to_string(), &self.name)
    }
}

#[derive(Debug,Clone)]
pub struct Livery {
    pub livery_folder: Option<String>,
    pub car_json: Option<ZipLiveryContent>,
    pub livery_files: Vec<ZipLiveryContent>
}

impl Livery {
    pub fn check_if_conflict(& self, state: &State) -> Conflict {
        let mut conflict = Conflict::None;
        let mut car_json_conflict = false;

        if let Some(car) = &self.car_json {
            if car.get_target(state).exists() {
                car_json_conflict = true;
                if car.is_same_as_target(state) {
                    conflict = Conflict::Identical;
                } else {
                    conflict = Conflict::CarOnly;
                }
            }
        }

        if let Some(folder) = self.get_livery_folder(state) {
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
                            if item.get_target(state).exists() {
                                conflict = match conflict {
                                    Conflict::None => {
                                        if item.is_same_as_target(state) {
                                            Conflict::Identical 
                                        } else {
                                            return Conflict::LiveryOnly;
                                        }
                                    },
                                    Conflict::CarOnly => {
                                        return Conflict::Both;
                                    },
                                    Conflict::Identical => {
                                        if item.is_same_as_target(state) {
                                            Conflict::Identical 
                                        } else if car_json_conflict {
                                            return Conflict::Both;
                                        } else {
                                            return Conflict::LiveryOnly;
                                        }
                                    },
                                    _ => {
                                        return conflict;
                                    }
                                };
                            }
                        }
                    }
                }
            }
        }

        conflict
    }

    fn get_livery_folder(& self, state: &State) -> Option<PathBuf> {
        let mut folder = state.root_folder.clone();
        folder.push(ACC_CUSTOMS_FOLDER_NAME);
        folder.push(ACC_LIVERY_FOLDER_NAME);
        
        folder.push(self.livery_folder.clone()?); // If no folder is defined this will return none here

        Some(folder)
    }

    pub fn write(&self, state: &State) -> io::Result<()> {
        // Setting up car.json
        if let Some(car) = &self.car_json {
            fs::write(car.get_target(state), car.file.clone())?;
        }

        // Creating the folder if necessary
        if let Some(folder) = self.get_livery_folder(state) {
            if !folder.exists() {
                fs::create_dir_all(folder)?;
            }
        }

        // Writing the livery files
        let iter = self.livery_files.iter();
        for item in iter {
            fs::write(item.get_target(state), item.file.clone())?;
        }

        Ok(())
    }
}

/// Finds and read a specific car.json within the cars folder of ACC (aka one that is already installed)
pub fn get_car_file(car: &String, state: &State) -> Option<ZipLiveryContent> {
    let mut name = PathBuf::from(car);
    name.set_extension("json");
    let name = get_filename(&name);

    let file = ZipLiveryContent { upper: CustomFolder::Cars, name, file: Vec::<u8>::new()};
    if file.get_target(state).exists() {
        if let Ok(content) = fs::read(file.get_target(state)) {
            return Some(ZipLiveryContent { upper: file.upper, name: file.name, file: content});
        }
    }

    None
}

/// Read all files out of a specific folder within the livery folder of ACC (aka one that is already installed)
pub fn get_livery_files(livery: &String, state: &State) -> Option<Vec<ZipLiveryContent>> {
    let mut folder = state.root_folder.clone();

    folder.push(ACC_CUSTOMS_FOLDER_NAME);
    folder.push(ACC_LIVERY_FOLDER_NAME);
    folder.push(&livery);

    if !folder.exists() || folder.is_file() {
        return None;
    }

    let mut output = Vec::<ZipLiveryContent>::new();

    if let Ok(mut folder_content) = folder.read_dir() {
        while let Some(Ok(item)) = folder_content.next() {
            if let Ok(content) = fs::read(item.path()) {
                output.push(ZipLiveryContent { upper: CustomFolder::Liveries(livery.clone()), name: item.file_name().to_str().expect("it is a string").to_string(), file: content});
            }
        }  
    }

    return Some(output);
}

/// Returns all car.json from the cars folder of ACC (aka all currently installed)
pub fn get_all_car_json(state: &State) -> Vec<ZipLiveryContent> {
    let mut folder = state.root_folder.clone();

    folder.push(ACC_CUSTOMS_FOLDER_NAME);
    folder.push(ACC_CAR_FOLDER_NAME);

    let mut output = Vec::<ZipLiveryContent>::new();

    if let Ok(mut folder_content) = folder.read_dir() {
        while let Some(Ok(item)) = folder_content.next() {
            if let Ok(content) = fs::read(item.path()) {
                output.push(ZipLiveryContent { upper: CustomFolder::Cars, name: item.file_name().to_str().expect("it is a string").to_string(), file: content});
            }
        }  
    }

    output
}

/// Reads a zip file and parses it into an unsorted list of ZipLiveryContent files
/// Run group_up to sort the data
pub fn get_zip_content(zip_file: &PathBuf) -> Option<Vec<ZipLiveryContent>> {
    if let Ok(stream) = fs::read(&zip_file) {
        let stream = Cursor::new(stream);

        let mut content = Vec::<ZipLiveryContent>::new();
        // Reading the zip
        if let Ok(zip_content) = zip::ZipArchive::new(&mut stream.clone()) {
            let iter = zip_content.file_names();
            let progressbar = ProgressBar::new(zip_content.len() as u64);
            progressbar.set_style(ProgressStyle::with_template("[{elapsed_precise}] {bar:50.cyan/blue} {pos:>1}/{len:5}")
                    .expect("Progress Style is valid (At least when it was typed, an update to indicatif might have broken it)")
                    .progress_chars("##-"));

            // Reading the file
            for item in iter {
                let mut data = Vec::<u8>::new();
                let mut internal_path = PathBuf::from(item);
                if zip_extensions::read::zip_extract_file_to_memory(zip_file, &internal_path, &mut data).is_ok() {
                    
                    // 
                    let name = super::get_filename(&internal_path);
                    let upper = if internal_path.pop() && internal_path.file_name().is_some() {
                        let folder_name = super::get_filename(&internal_path).trim().to_string();

                        match folder_name.to_lowercase().as_str() {
                            "cars" => CustomFolder::Cars,
                            _ => CustomFolder::Liveries(folder_name)
                        }
                    } else {
                        let mut origin_file = zip_file.clone();
                        origin_file.set_extension("");
                        CustomFolder::Liveries(super::get_filename(&origin_file))
                    };

                    
                    content.push(ZipLiveryContent { upper, name, file: data });
                }
                progressbar.inc(1);
            }
            progressbar.finish();

            return Some(content);
        }
    }

    None
}

/// Parses the car.json and returns the folder in which the livery files are stored
pub fn read_car_for_livery_folder(car_json: &ZipLiveryContent) -> Option<String> {
    if let Ok(parsed_json) = super::read_json_from_bytes(car_json.file.clone()) {
        // We get the livery folder from the car.json, if not found we just add the livery
        if let Some(target_foldername) = parsed_json.get("customSkinName") {
            if let Some(target_foldername) = target_foldername.as_str() {
                if !target_foldername.is_empty() {
                    return Some(target_foldername.to_string());
                } else {
                    return None;
                }
            }
        }
    }
    
    None
}

/// Takes an unsorted list of Files and groups the car.jsons and livery files together
pub fn group_up(mut files: Vec<ZipLiveryContent>) -> Vec<Livery> {
    let mut liveries = Vec::<Livery>::new();

    while let Some(item) = files.pop() {
        if let CustomFolder::Cars = item.upper {


            if let Some(target_foldername) = read_car_for_livery_folder(&item) {
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
                
                if !found { // Livery group not found, we create a new one
                    liveries.push(Livery { livery_folder: Some(target_foldername), car_json: Some(item), livery_files: Vec::<ZipLiveryContent>::new() });
                }
            } else {
                liveries.push(Livery { livery_folder: None, car_json: Some(item), livery_files: Vec::<ZipLiveryContent>::new() });
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

// car.json
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

pub fn write_livery_in_zip(livery: Livery) -> io::Result<String> {
    let target_name = if let Some(liver) = livery.livery_folder.clone() {
        liver
    } else {
        if let Some(car) = &livery.car_json {
            car.name.clone()
        } else {
            String::new()
        }
    };
    let target_name = format!("{}.zip", target_name);

    println!("Compressing Files...");
    let progressbar = ProgressBar::new(livery.livery_files.len() as u64 + if livery.car_json.is_some() { 1 } else { 0 });
    progressbar.set_style(ProgressStyle::with_template("[{elapsed_precise}] {bar:50.cyan/blue} {pos:>1}/{len:5}")
                    .expect("Progress Style is valid (At least when it was typed, an update to indicatif might have broken it)")
                    .progress_chars("##-"));


    let buffer = File::create(&target_name)?;
    let mut writer = zip::ZipWriter::new(buffer);

    if let Some(car) = livery.car_json {
        writer.start_file(car.get_interal_path(), zip::write::FileOptions::default())?;
        writer.write_all(car.file.as_slice())?;
        progressbar.inc(1);
    }

    if let Some(_) = livery.livery_folder {
        for item in livery.livery_files {
            writer.start_file(item.get_interal_path(), zip::write::FileOptions::default())?;
            writer.write_all(item.file.as_slice())?;
            progressbar.inc(1);
        }
    }

    progressbar.finish();

    writer.finish()?;
    
    Ok(target_name)
}
