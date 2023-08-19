use std::path::PathBuf;

use backend::livery_ops;
use clap::Parser;
use dialoguer::{Confirm, Input};
use indicatif::{ProgressBar, ProgressStyle};

use crate::backend::livery_ops::Livery;

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
    export_only_livery: bool
}

fn main() {
    let args = Args::parse();
    let mut settings = match backend::app_data::get_settings() {
        Some(set) => set,
        None => {
            panic!("Unable to load app settings, exiting");
        }
    };

    // Switching Livery mode
    if args.mode {
        if let Some(state) = settings.switch_liverymode() {
            println!("Liverymode turned {}", match state {
                true => "on",
                false => "off"
            });

            backend::app_data::write_settings(settings.clone());
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
                
                
                fn handle_write(liver: &Livery) {
                    if let Err(e) = liver.write() {
                        panic!("Error occured when writing file: {}", e);
                    }
                }

                match item.check_if_conflict() {
                    None => {
                        // No conflict, continue
                        handle_write(&item);
                    },
                    Some((carjson_match, folder_conflict)) => {
                        // Conflict
                        if carjson_match && folder_conflict {
                            //Both conflict, so offer override
                            println!("Conflict\n{} and livery folder {} already exist",
                                item.car_json.clone().expect("has to exist to conflict").name,
                                item.livery_folder.clone().expect("has to exist to conflict"));

                            if Confirm::new().with_prompt("Override?").default(true).interact().unwrap_or(false) {
                                    handle_write(&item);
                            } else {
                                println!("SKIP");
                            }
                        } else if carjson_match {
                            // We take the car json and write it with it's new name
                            let mut item = item;
                            let mut car = item.car_json.expect("can't have a conflict if it doesn't exist");
                            item.car_json = None;

                            // Asking for a new name and testing it
                            let mut filename = PathBuf::from(&car.name);
                            filename.set_extension("");
                            let mut filename = filename.to_str().expect("there has to be a filename").to_string();
                            
                            let mut base_folder = backend::get_acc_folder();
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
                                handle_write(&alt);
                                handle_write(&item); // Item no longer has a car_json, so no more conflict
                            } else {
                                println!("SKIP");
                            }
                        } else /* if folder_conflict */  { // this is the last option
                            println!("Conflict\nLivery folder {} already exist", item.livery_folder.clone().expect("has to exist to conflict"));

                            if Confirm::new().with_prompt("Override?").default(true).interact().unwrap_or(false) {
                                    handle_write(&item);
                            } else {
                                println!("SKIP");
                            }
                        }
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

        let bundle = if let Some(car) = livery_ops::get_car_file(&name) {
            let (folder, content) = if let Some(folder) = livery_ops::read_car_for_livery_folder(&car) {
                if let Some(content) = livery_ops::get_livery_files(&folder) {
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
        } else if let Some(content) = livery_ops::get_livery_files(&name) {
            //Challenge: we need to find the car_json that contains our folder

            //Except when we don't:
            if args.export_only_livery {
                Livery {car_json: None, livery_folder: Some(name.clone()), livery_files: content}
            } else {
                let all_cars = livery_ops::get_all_car_json();

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
