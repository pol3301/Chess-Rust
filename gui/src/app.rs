use std::{thread::sleep, time::Duration};

use crate::config::AppConfig;
use crate::game::Game;
use crate::{board_ui::BoardUI, game::OnlineGame};
use chess_engine::{Move, PieceColor, fen};
use egui::{Color32, Frame, RichText};
use networking::{Connection, Message, bind_socket};
use tokio::sync::mpsc::{Receiver, Sender};

enum AppState {
    PlayingLocal(Box<Game>),
    PeerQuery(String),
    PlayingOnline {
        game: Box<Game>,
        tx: Sender<Message>,
        rx: Receiver<Message>,
    },
    Connecting {
        tx: Sender<Message>,
        rx: Receiver<Message>,
    },
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
                                self.state = AppState::PlayingLocal(Box::new(Game::new(None)));
                            }

                            ui.add_space(30.0);

                            let online_button = egui::Button::new(
                                egui::RichText::new("Online Game")
                                    .size(24.0)
                                    .color(Color32::WHITE),
                            )
                            .fill(Color32::from_rgb(70, 130, 180));

                            if ui.add_sized([200.0, 60.0], online_button).clicked() {
                                self.state = AppState::PeerQuery(String::new());
                            }
                        });
                    });
            }

            AppState::PlayingLocal(game) => {
                egui::Frame::default()
                    .fill(Color32::from_rgb(30, 30, 30))
                    .show(ui, |ui| {
                        BoardUI::ui(ui, game, &self.config);
                    });
            }

            AppState::PlayingOnline { game, tx: _tx, rx } => {
                egui::Frame::default()
                    .fill(Color32::from_rgb(30, 30, 30))
                    .show(ui, |ui| {
                        BoardUI::ui(ui, game, &self.config);
                    });

                if let Some(online) = &game.online()
                    && online.color_us != game.board().get_turn()
                    && let Ok(message) = rx.try_recv()
                    && let Message::Move(m) = message
                {
                    let m = Move::from_bytes(m);
                    game.try_move(m.from_square(), m.to_square(), m.promotion_type())
                        .expect("Peer sent bad move");
                }
            }
            AppState::Connecting { tx: _tx, rx } => {
                Frame::default()
                    .fill(Color32::from_rgb(30, 30, 30))
                    .show(ui, |ui| {
                        ui.set_min_size(ui.available_size());

                        ui.with_layout(egui::Layout::top_down(egui::Align::Center), |ui| {
                            ui.add_space(ui.available_height() / 2.5);
                            ui.label(
                                RichText::new("Connecting to peer")
                                    .size(24.0)
                                    .color(Color32::WHITE),
                            );
                            ui.add_space(15.0);
                            ui.spinner();
                        });
                    });

                if let Ok(message) = rx.try_recv() {
                    if message == Message::EstablishedConnection {
                        let old_state = std::mem::replace(&mut self.state, AppState::MainMenu);

                        if let AppState::Connecting { tx, rx } = old_state {
                            let online = OnlineGame {
                                color_us: PieceColor::White,
                                tx: tx.clone(),
                            };

                            self.state = AppState::PlayingOnline {
                                game: Box::new(Game::new(Some(online))),
                                tx,
                                rx,
                            };
                        }
                    } else if message == Message::DropConnection {
                        self.state = AppState::MainMenu;
                    }
                }
            }
            AppState::PeerQuery(buffer) => {
                let mut new_state = None;
                Frame::default()
                    .fill(Color32::from_rgb(30, 30, 30))
                    .show(ui, |ui| {
                        ui.set_min_size(ui.available_size());

                        ui.with_layout(egui::Layout::top_down(egui::Align::Center), |ui| {
                            ui.add_space(ui.available_height() / 2.5);
                            ui.text_edit_singleline(buffer);

                            let connect = egui::Button::new(
                                RichText::new("Connect").size(24.0).color(Color32::WHITE),
                            );

                            if ui.add_sized([200.0, 60.0], connect).clicked() {
                                if let Some(socket) = bind_socket() {
                                    let (tx, rx) = Connection::create_connection(
                                        socket,
                                        buffer.parse().unwrap(),
                                    );

                                    new_state = Some(AppState::Connecting { tx, rx });
                                } else {
                                    eprintln!("Couldn't bind to any socket");
                                }
                            }
                        });
                    });

                if let Some(state) = new_state {
                    self.state = state;
                }
            }
        }

        sleep(Duration::from_millis(16));
        ui.request_repaint();
    }
}
