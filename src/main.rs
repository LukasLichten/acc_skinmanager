use std::path::PathBuf;

use backend::livery_ops;
use clap::Parser;
use dialoguer::{Confirm, Input};
use indicatif::{ProgressBar, ProgressStyle};

use crate::backend::livery_ops::{Livery, Conflict};

pub mod backend;
pub mod model;
pub mod view;

#[derive(Debug, Parser)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short, long, help = "Switches ACC into Liverymode (only use when game is turned off)")]
    mode: bool,

    #[arg(short, long, help = "Installs a zipfile")]
    install: Option<String>,

    #[arg(short, long, help = "Export livery into zipfile based on car.json name or livery foldername")]
    export: Option<String>,

    #[arg(short = 'O', long, help = "exports only the livery folder")]
    export_only_livery: bool,

    #[arg(long, help = "opens the Customs folder")]
    open: bool,

    #[cfg(target_os = "linux")]
    #[arg(long, help = "set the steam root folder manually, instead of using $STEAM_DIR")]
    steam_dir: Option<String>
}

pub struct State {
    root_folder: PathBuf
}

fn main() {
    let args = Args::parse();
    
    #[cfg(target_os = "linux")]
    if let Some(steam_dir) = args.steam_dir {
        std::env::set_var("STEAM_DIR", steam_dir);
    }

    // Getting the root folder
    let res = match backend::get_acc_folder() {
        Ok(res) => res,
        Err(res) => {
            println!("[ERROR] $STEAM_DIR was set, but invalid. Continuing...");
            // dialog::beep(dialog::BeepType::Error);
            // dialog::alert_default("$STEAM_DIR was set, but invalid. Continuing...");
            res
        }
    };

    let acc_settings_folder = if let Some(model) = res {
        model
    } else {
        println!("[ERROR] Fatal: Documents/Assetto Corsa Competizione folder not found");
        println!("Make sure you installed the game and ran it at least once");
        // dialog::beep(dialog::BeepType::Error);
        // dialog::alert_default("Documents/Assetto Corsa Competizione folder not found.\n
        //     Make sure you installed the game and ran the game at least once");
        return;
    };

    let state = State {
        root_folder: acc_settings_folder
    };

    // Opens the folder in your filemanger
    if args.open {
        let mut path = state.root_folder;
        path.push(backend::livery_ops::ACC_CUSTOMS_FOLDER_NAME);
        if !path.exists() {
            panic!("ACC Customs folder is missing!");
        }

        if let Err(e) = open::that(path) {
            panic!("Failed to open file path: {}", e);
        }
        return;
    }


    let mut settings = match backend::app_data::get_settings(&state) {
        Some(set) => set,
        None => {
            panic!("Unable to load app settings, exiting");
        }
    };

    // Switching Livery mode
    if args.mode {
        if let Some(mode_state) = settings.switch_liverymode(&state) {
            println!("Liverymode turned {}", match mode_state {
                true => "on",
                false => "off"
            });

            backend::app_data::write_settings(settings.clone(), &state);
        } else {
            panic!("Failed to switch Liverymode");
        }

        return;
    }

    // Installing Zipfile 
    if let Some(fil) = args.install {
        println!("Import...");
        let path = PathBuf::from(fil);
        println!("Extracting files...");
        if let Some(val) = livery_ops::get_zip_content(&path) {
            let results = livery_ops::group_up(val);

            println!("Installing Liveries from archive {}", path.to_str().expect("it has to exist, else there is no path"));
            //println!("{} liveries found\n", results.len());

            let progressbar = ProgressBar::new(results.len() as u64);
            progressbar.set_style(ProgressStyle::with_template("[{elapsed_precise}] {bar:50.cyan/blue} {pos:>1}/{len:5} {msg}")
                    .expect("Progress Style is valid (At least when it was typed, an update to indicatif might have broken it)")
                    .progress_chars("##-"));
            //return;

            for item in results {
                progressbar.set_message(if let Some(liver) = item.livery_folder.clone() {
                    liver
                } else {
                    if let Some(car) = &item.car_json {
                        car.name.clone()
                    } else {
                        String::new()
                    }
                });
                
                
                fn handle_write(liver: &Livery, state: &State) {
                    if let Err(e) = liver.write(state) {
                        panic!("Error occured when writing file: {}", e);
                    }
                }

                match item.check_if_conflict(&state) {
                    Conflict::None => {
                        // No conflict, continue
                        handle_write(&item, &state);
                    },
                    Conflict::Both => {
                        //Both conflict, so offer override
                        println!("Conflict\n{} and livery folder {} already exist",
                        item.car_json.clone().expect("has to exist to conflict").name,
                        item.livery_folder.clone().expect("has to exist to conflict"));

                        if Confirm::new().with_prompt("Override?").default(true).interact().unwrap_or(false) {
                            handle_write(&item, &state);
                        } else {
                            println!("SKIP");
                        }
                    },
                    Conflict::CarOnly => {
                        // We take the car json and write it with it's new name
                        let mut item = item;
                        let mut car = item.car_json.expect("can't have a conflict if it doesn't exist");
                        item.car_json = None;

                        // Asking for a new name and testing it
                        let mut filename = PathBuf::from(&car.name);
                        filename.set_extension("");
                        let mut filename = filename.to_str().expect("there has to be a filename").to_string();
                        
                        let mut base_folder = state.root_folder.clone();
                        base_folder.push(backend::livery_ops::ACC_CUSTOMS_FOLDER_NAME);
                        base_folder.push(backend::livery_ops::ACC_CAR_FOLDER_NAME);
                        let base_folder = base_folder;
                        
                        let mut target = base_folder.clone();
                        target.push(&filename);
                        target.set_extension("json");

                        let mut skip = false;

                        while target.exists() && !skip {
                            println!("Conflict\nCar json with the name {} already exists", filename);

                            if let Ok(input) = Input::<String>::new().with_prompt("Rename").allow_empty(true).interact_text() {
                                if input.is_empty() {
                                    skip = true;
                                }

                                target = base_folder.clone();
                                target.push(&input);
                                target.set_extension("json");

                                filename = input;
                            } else {
                                skip = true;
                            }
                        }
                        
                        // Editing the car.json
                        if !skip {
                            car.name = backend::get_filename(&target);
                            let alt = Livery {car_json: Some(car), livery_folder: None, livery_files: Vec::<backend::livery_ops::ZipLiveryContent>::new()};
                            handle_write(&alt, &state);
                            handle_write(&item, &state); // Item no longer has a car_json, so no more conflict
                        } else {
                            println!("SKIP");
                        }
                    },
                    Conflict::LiveryOnly => {
                        println!("Conflict\nLivery folder {} already exist", item.livery_folder.clone().expect("has to exist to conflict"));

                        if Confirm::new().with_prompt("Override?").default(true).interact().unwrap_or(false) {
                            handle_write(&item, &state);
                        } else {
                            println!("SKIP");
                        }
                    },
                    Conflict::Identical => {
                        println!("Already Up-to-date, SKIP");
                    }
                }

                progressbar.inc(1);
            }
            progressbar.set_message("DONE");
            progressbar.finish();
        } else {
            panic!("Failed to read zip file");
        }

        println!("Finished!");

        return;
    }

    //Extract Liveryfile
    if let Some(name) = args.export {
        println!("Export...");
        print!("Trying to find {}... ", &name);

        let bundle = if let Some(car) = livery_ops::get_car_file(&name, &state) {
            let (folder, content) = if let Some(folder) = livery_ops::read_car_for_livery_folder(&car) {
                if let Some(content) = livery_ops::get_livery_files(&folder, &state) {
                    (Some(folder), content)
                } else {
                    (None, Vec::<livery_ops::ZipLiveryContent>::new())
                }
            } else {
                (None, Vec::<livery_ops::ZipLiveryContent>::new())
            };

            // Handling export flag
            if !args.export_only_livery {
                Livery {car_json: Some(car), livery_folder: folder, livery_files: content}
            } else {
                Livery {car_json: None, livery_folder: folder, livery_files: content}
            }
        } else if let Some(content) = livery_ops::get_livery_files(&name, &state) {
            //Challenge: we need to find the car_json that contains our folder

            //Except when we don't:
            if args.export_only_livery {
                Livery {car_json: None, livery_folder: Some(name.clone()), livery_files: content}
            } else {
                let all_cars = livery_ops::get_all_car_json(&state);

                let mut car = None;

                // We go through all car.json to find one which points to this folder 
                for item in all_cars {
                    if let Some(folder) = livery_ops::read_car_for_livery_folder(&item) {
                        if folder == name {
                            car = Some(item);
                            break;
                        }
                    }
                }
                
                Livery {car_json: car, livery_folder: Some(name.clone()), livery_files: content }
            }
        } else {
            panic!("No file or folder found!");
        };

        println!("FOUND!");

        if bundle.car_json.is_none() && bundle.livery_files.is_empty() {
            // Odd case when car.json is found, but not included due to only exporting livery files
            panic!("{} exists as a car.json, but no livery files could be found, exiting", name);
        }


        //Completing the export
        if let Ok(target_name) = livery_ops::write_livery_in_zip(bundle) {
            println!("Exported {} successfully!", target_name);
        } else {
            panic!("Error while trying to create zip file");
        }

        
        return;
    }


    //Graphic app
    
}
