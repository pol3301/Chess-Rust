mod app;
mod game;

use crate::app::ChessApp;

fn main() -> eframe::Result {
    let viewport = egui::viewport::ViewportBuilder::default()
        .with_title("Chess")
        .with_inner_size([1100.0, 800.0])
        .with_min_inner_size([600.0, 600.0])
        .with_resizable(true);

    let native_options = eframe::NativeOptions {
        viewport,
        ..Default::default()
    };
    eframe::run_native(
        "Chess",
        native_options,
        Box::new(|cc| Ok(Box::new(ChessApp::new(cc)))),
    )
}
