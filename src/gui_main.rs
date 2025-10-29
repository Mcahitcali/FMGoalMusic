mod audio;
mod audio_converter;
mod capture;
mod config;
mod gui;
mod ocr;
mod region_selector;
mod utils;

fn main() -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([800.0, 600.0])
            .with_min_inner_size([600.0, 400.0])
            .with_icon(
                // Default icon
                eframe::icon_data::from_png_bytes(&[]).unwrap_or_default(),
            ),
        ..Default::default()
    };

    eframe::run_native(
        "FM Goal Musics",
        options,
        Box::new(|cc| Ok(Box::new(gui::FMGoalMusicsApp::new(cc)))),
    )
}
