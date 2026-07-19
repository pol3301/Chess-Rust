mod app;
mod bitboard;
mod board;
mod fen;
mod move_generator;
mod moves;
mod piece;
mod squares;

use crate::app::ChessApp;
// use crate::fen::load_fen;

fn main() -> eframe::Result {
    let viewport = egui::viewport::ViewportBuilder::default()
        .with_title("Chess")
        .with_inner_size([800.0, 800.0])
        .with_min_inner_size([800.0, 800.0])
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
