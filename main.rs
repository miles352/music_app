#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
// hide console window on Windows in release
use image::{self};
mod app;
mod audio;



fn main() {

    
    let image = image::load_from_memory(include_bytes!("../assets/dababy.jpg")).expect("couldn't find file").into_rgba8();
    let (width, height) = image.dimensions();
    let native_options = eframe::NativeOptions {
        icon_data: Some(eframe::IconData {
            rgba: image.to_vec(),
            width,
            height
        }),
        ..Default::default()
    };
    
    eframe::run_native(
        "Music Player",
        native_options,
        Box::new(|cc| Box::new(app::MusicApp::new(cc))),
    );
}