use crate::game::{Game, GameMoment, SelectionState};
use chess_engine::{Piece, PieceColor, PieceTrait, PieceType, bitboard::BitboardExt};
use egui::{
    Button, Color32, Id, Image, Key, KeyboardShortcut, Modifiers, Painter, Pos2, Rect, Sense,
    Stroke, Vec2, include_image,
};

use crate::config::{AppConfig, ColorSet, PieceSet};

pub struct BoardUI;

impl BoardUI {
    fn coords_from_index(index: u8, perspective: PieceColor) -> (u8, u8) {
        match perspective {
            PieceColor::White => (index % 8, 7 - (index / 8)),
            PieceColor::Black => (index % 8, index / 8),
        }
    }

    fn index_from_coordinates(
        x: f32,
        y: f32,
        square_size: f32,
        board_origin: Pos2,
        perspective: PieceColor,
    ) -> u8 {
        let x = ((x - board_origin.x) / square_size) as u8;
        let y = ((y - board_origin.y) / square_size) as u8;

        match perspective {
            PieceColor::White => x + ((7 - y) * 8),
            PieceColor::Black => x + (y * 8),
        }
    }

    fn draw_tiles(
        painter: &Painter,
        color_set: ColorSet,
        origin: Pos2,
        size: f32,
        perspective: PieceColor,
    ) {
        let color_light = color_set.0;
        let color_dark = color_set.1;

        let rect_size = egui::Vec2::new(size, size);

        for x in 0..8 {
            for y in 0..8 {
                let is_light = match perspective {
                    PieceColor::White => (x + y) % 2 == 0,
                    PieceColor::Black => (x + y) % 2 != 0,
                };
                let min = origin + Vec2::new(x as f32 * size, y as f32 * size);
                let rect = Rect::from_min_size(min, rect_size);

                if is_light {
                    painter.rect_filled(rect, 0.0, color_light);
                } else {
                    painter.rect_filled(rect, 0.0, color_dark);
                }
            }
        }
    }

    fn get_piece_image(piece: Piece, image_set: PieceSet) -> Image<'static> {
        Image::new(include!(concat!(
            env!("OUT_DIR"),
            "/generated_piece_match.rs"
        )))
    }

    fn draw_piece(ui: &egui::Ui, pos: Pos2, piece: Piece, set: PieceSet, size: f32) {
        let rect_size = Vec2::new(size, size);

        let rect = Rect::from_min_size(pos, rect_size);
        Self::get_piece_image(piece, set).paint_at(ui, rect);
    }

    fn draw_legal_moves(painter: &Painter, index: u8, game: &Game, origin: Pos2, square_size: f32) {
        let legal_moves_color = Color32::from_rgba_unmultiplied(0x30, 0x30, 0x30, 128);

        for m in game.legal_moves().as_slice() {
            if m.from_square() == index {
                let center_raw = Self::coords_from_index(m.to_square(), game.perspective);
                let center = origin
                    + Vec2::new(
                        (center_raw.0 as f32 * square_size) + (square_size / 2.0),
                        (center_raw.1 as f32 * square_size) + (square_size / 2.0),
                    );

                painter.circle(center, square_size * 0.15, legal_moves_color, Stroke::NONE);
            }
        }
    }

    fn promotion_query(
        ui: &mut egui::Ui,
        index: u8,
        game: &Game,
        config: &AppConfig,
        origin: Pos2,
        square_size: f32,
    ) -> Option<PieceType> {
        let turn = game.board().get_turn();
        let mut selected_piece = None;

        let coords_raw = Self::coords_from_index(index, game.perspective);
        let popup_pos = origin
            + Vec2::new(
                coords_raw.0 as f32 * square_size,
                coords_raw.1 as f32 * square_size,
            );

        egui::Area::new(Id::new("promotion_popup"))
            .fixed_pos(popup_pos)
            .order(egui::Order::Foreground)
            .show(ui.ctx(), |ui| {
                egui::Frame::popup(ui.style())
                    .fill(Color32::from_rgb(40, 40, 40))
                    .stroke(Stroke::new(2.0, Color32::LIGHT_GRAY))
                    .inner_margin(8.0)
                    .show(ui, |ui| {
                        ui.vertical(|ui| {
                            let promotions = [
                                PieceType::Queen,
                                PieceType::Rook,
                                PieceType::Bishop,
                                PieceType::Knight,
                            ]
                            .map(|pt| Piece::make(pt, turn));

                            for piece in promotions {
                                let img = Self::get_piece_image(piece, config.piece_set);
                                if ui
                                    .add_sized(
                                        [square_size * 0.8, square_size * 0.8],
                                        Button::new(img),
                                    )
                                    .clicked()
                                {
                                    selected_piece = Some(piece.get_type());
                                }
                            }
                        });
                    });
            });

        selected_piece
    }

    pub fn ui(ui: &mut egui::Ui, game: &mut Game, config: &AppConfig) {
        ui.set_min_size(ui.available_size());

        let rect = ui.available_rect_before_wrap();

        let board_size = rect.height().min(rect.width()) * 0.95;
        let square_size = board_size / 8.0;
        let offset_x = (rect.width() - board_size) * 0.95;
        let offset_y = (rect.height() - board_size) / 2.0;
        let board_origin = Pos2::new(offset_x, offset_y);
        let board_rect = Rect::from_min_size(board_origin, Vec2::new(board_size, board_size));

        Self::draw_tiles(
            ui.painter(),
            config.color_set,
            board_origin,
            square_size,
            game.perspective,
        );

        let mut pieces = game.board().get_all_pieces();

        let dragging_index = if let SelectionState::Dragging(index) = game.selected_piece() {
            Some(index)
        } else {
            None
        };

        while pieces != 0 {
            let index = pieces.pop_lsb();

            if dragging_index == Some(index) {
                continue;
            }

            let (x_raw, y_raw) = Self::coords_from_index(index, game.perspective);
            let (x, y) = (x_raw as f32 * square_size, y_raw as f32 * square_size);
            let pos = board_origin + Vec2::new(x, y);

            let piece = game.board().piece_at(index);

            Self::draw_piece(ui, pos, piece, config.piece_set, square_size);
        }

        let response = ui.interact(board_rect, Id::new(1), Sense::click_and_drag());

        if let Some(mouse_pos) = ui.pointer_hover_pos()
            && response.contains_pointer()
        {
            let mouse_index = Self::index_from_coordinates(
                mouse_pos.x,
                mouse_pos.y,
                square_size,
                board_origin,
                game.perspective,
            );

            if game.moment() == GameMoment::Playing {
                if response.drag_started() {
                    game.handle_drag_start(mouse_index);
                } else if response.drag_stopped() {
                    game.handle_drag_end(mouse_index);
                }

                if let SelectionState::Dragging(index) = game.selected_piece() {
                    Self::draw_legal_moves(ui.painter(), index, game, board_origin, square_size);

                    if ui.rect_contains_pointer(board_rect) {
                        Self::draw_piece(
                            ui,
                            mouse_pos - Vec2::new(square_size / 2.0, square_size / 2.0),
                            game.board().piece_at(index),
                            config.piece_set,
                            square_size,
                        );
                    }
                };

                if response.clicked() {
                    game.handle_click(mouse_index);
                }
            }
        }

        if let GameMoment::Promoting(from, to) = game.moment()
            && let Some(piece) =
                Self::promotion_query(ui, to, game, config, board_origin, square_size)
        {
            game.try_move(from, to, Some(piece));
        }

        if let SelectionState::Selected(index) = game.selected_piece() {
            Self::draw_legal_moves(ui.painter(), index, game, board_origin, square_size);
        }

        if ui.input(|i| i.key_pressed(Key::F)) {
            game.perspective = game.perspective.flip();
        }

        if ui.input_mut(|i| i.consume_shortcut(&KeyboardShortcut::new(Modifiers::COMMAND, Key::Z)))
        {
            game.undo();
        }
    }
}
