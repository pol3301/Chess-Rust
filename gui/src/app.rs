use crate::board_ui::BoardUI;
use crate::config::AppConfig;
use crate::game::Game;
use egui::Color32;

enum AppState {
    Playing(Box<Game>),
    MainMenu,
}

pub struct ChessApp {
    state: AppState,
    config: AppConfig,
}

impl ChessApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        egui_extras::install_image_loaders(&cc.egui_ctx);

        Self {
            state: AppState::MainMenu,
            config: AppConfig::default(),
        }
    }
}

impl eframe::App for ChessApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        match &mut self.state {
            AppState::MainMenu => {
                egui::Frame::default()
                    .fill(Color32::from_rgb(30, 30, 30))
                    .show(ui, |ui| {
                        ui.set_min_size(ui.available_size());

                        ui.with_layout(egui::Layout::top_down(egui::Align::Center), |ui| {
                            ui.add_space(ui.available_height() / 2.5);

                            ui.label(
                                egui::RichText::new("Chess")
                                    .size(48.0)
                                    .strong()
                                    .color(Color32::WHITE),
                            );

                            ui.add_space(30.0);

                            let start_button = egui::Button::new(
                                egui::RichText::new("Local Game")
                                    .size(24.0)
                                    .color(Color32::WHITE),
                            )
                            .fill(Color32::from_rgb(70, 130, 180));

                            if ui.add_sized([200.0, 60.0], start_button).clicked() {
                                self.state = AppState::Playing(Box::new(Game::new()));
                            }
                        });
                    });
            }

            AppState::Playing(game) => {
                egui::Frame::default()
                    .fill(Color32::from_rgb(30, 30, 30))
                    .show(ui, |ui| {
                        BoardUI::ui(ui, game, &self.config);
                    });
            }
        }
    }
}
