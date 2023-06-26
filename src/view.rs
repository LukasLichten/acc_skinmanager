use fltk::image::PngImage;



pub fn get_icon() -> Option<PngImage> {
    let logo_in_bytes = include_bytes!("./assets/logo128.png");

    let logo_loaded = PngImage::from_data(logo_in_bytes);
    
    logo_loaded.ok()
}