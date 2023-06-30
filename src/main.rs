use std::path::PathBuf;

use backend::livery_ops;
use clap::Parser;

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
            return;
        } else {
            panic!("Failed to switch Liverymode");
        }
    }

    // Installing Zipfile 
    if let Some(fil) = args.install {
        let path = PathBuf::from(fil);
        if let Some(val) = livery_ops::get_zip_content(&path) {
            let results = livery_ops::group_up(val);

            for item in results {
                print!("Installing livery {} ", item.livery_folder.clone().unwrap()); //TODO handle this properly
                // No conflict, continue
                if item.check_if_conflict().is_none() {
                    if let Err(e) = item.write() {
                        panic!("Error occured when writing file: {}", e);
                    }
                }
                
                //TODO handle conflict

                println!("DONE!")
            }

        } else {
            panic!("Failed to read zip file");
        }
    }

    //Graphic app
    
}
