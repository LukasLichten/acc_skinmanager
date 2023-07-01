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
    install: Option<String>
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


    //Graphic app
    
}
