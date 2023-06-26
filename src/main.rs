pub mod backend;
pub mod model;
pub mod view;

fn main() {
    let re = backend::menu_changer::set_dds_generation(false);

    print!("{}", re);
}
