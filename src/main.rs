use clap::Parser;

pub mod backend;
pub mod model;
pub mod view;

#[derive(Debug, Parser)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short, long, help = "Switches ACC into Liverymode (only use when game is turned off)")]
    mode: bool
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

    //Graphic app
}
