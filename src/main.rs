use std::path::PathBuf;

use backend::livery_ops;
use clap::Parser;

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
        if let Some(val) = livery_ops::get_zip_content(&path) {
            let results = livery_ops::group_up(val);

            println!("Installing Liveries from archive {}", path.to_str().expect("it has to exist, else there is no path"));
            println!("{} liveries found\n", results.len());

            for item in results {
                print!("Installing livery {} ", if let Some(liver) = item.livery_folder.clone() {
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
                    println!(" DONE!")
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
                            if backend::cli::confirm(format!("Conflict\n{} and livery folder {} already exist\nOverride?",
                                item.car_json.clone().expect("has to exist to conflict").name, item.livery_folder.clone().expect("has to exist to conflict")).as_str(),
                                true, "\n", true) {
                                
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
                            let mut input = PathBuf::from(&car.name);
                            input.set_extension("");
                            let mut input = input.to_str().expect("there has to be a filename").to_string();
                            
                            let mut base_folder = backend::get_acc_folder();
                            base_folder.push(backend::livery_ops::ACC_CUSTOMS_FOLDER_NAME);
                            base_folder.push(backend::livery_ops::ACC_CAR_FOLDER_NAME);
                            let base_folder = base_folder;
                            
                            let mut target = base_folder.clone();
                            target.push(&input);
                            target.set_extension(".json");

                            while target.exists() {
                                input = backend::cli::prompt(format!("Conflict\nCar json with the name {} already exists\nRename?",
                                     &input).as_str(),
                                     None, false, false, ": ", false);

                                let mut target = base_folder.clone();
                                target.push(&input);
                                target.set_extension(".json");
                            }
                            
                            // Editing the car.json
                            car.name = backend::get_filename(&target);
                            let alt = Livery {car_json: Some(car), livery_folder: None, livery_files: Vec::<backend::livery_ops::ZipLiveryContent>::new()};
                            handle_write(&alt);
                            handle_write(&item); // Item no longer has a car_json, so no more conflict

                        } else /* if folder_conflict */  { // this is the last option
                            if backend::cli::confirm(format!("Conflict\nLivery folder {} already exist\nOverride?",
                                item.livery_folder.clone().expect("has to exist to conflict")).as_str(),
                                true, "\n", true) {
                                
                                    handle_write(&item);
                                    println!("");
                            } else {
                                println!("SKIP");
                            }
                        }
                    }
                }

                
            }

        } else {
            panic!("Failed to read zip file");
        }

        println!("Finished!");

        return;
    }

    //Extract Liveryfile
    if let Some(name) = args.export {
        print!("Export: Trying to find {}... ", &name);

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
            panic!("error while trying to create zip file");
        }

        
        return;
    }


    //Graphic app
    
}
