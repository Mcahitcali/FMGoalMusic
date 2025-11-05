use eframe::egui;

mod gui;

fn main() -> Result<(), eframe::Error> {
    // Initialize logging for production debugging
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();
    
    tracing::info!("Starting FM Goal Musics application");

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([800.0, 600.0])
            .with_min_inner_size([600.0, 400.0])
            .with_title("FM Goal Musics"),
        ..Default::default()
    };

    eframe::run_native(
        "FM Goal Musics",
        options,
        Box::new(|cc| {
            // Configure egui
            cc.egui_ctx.set_visuals(egui::Visuals::dark());
            Box::new(gui::FMGoalMusicsApp::new(cc))
        }),
    )
}
