use chess_engine::{
    Board, Move, MoveList, Piece, PieceColor, PieceTrait, PieceType, bitboard::BitboardExt, fen,
    generate_legal_moves,
};
use egui::{Color32, Id, Image, Painter, Pos2, Rect, Sense, Stroke, Vec2, include_image};

#[derive(Clone, Copy)]
struct ColorSet(Color32, Color32); //(Light, Dark)

impl ColorSet {
    const LICHESS_COLORS: Self = Self(
        Color32::from_rgb(0xF0, 0xD9, 0xB5),
        Color32::from_rgb(0xB5, 0x88, 0x63),
    );

    const BLACK_WHITE: Self = Self(
        Color32::from_rgb(0x00, 0x00, 0x00),
        Color32::from_rgb(0xFF, 0xFF, 0xFF),
    );

    const CHESSDOTCOM_COLORS: Self = Self(
        Color32::from_rgb(0xEA, 0xED, 0xD1),
        Color32::from_rgb(0x77, 0x95, 0x57),
    );
}

#[derive(Clone, Copy)]
enum PieceSet {
    MaxArt,
    ChessDotCom,
    Lichess,
}

#[derive(PartialEq, Eq)]
enum GameMoment {
    Playing,
    Winner(PieceColor),
    Draw,
    Promoting(u8),
}

struct AppConfig {
    piece_set: PieceSet,
    color_set: ColorSet,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            piece_set: PieceSet::Lichess,
            color_set: ColorSet::LICHESS_COLORS,
        }
    }
}

struct BoardUI;

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
        let rect_size = egui::Vec2::new(size, size);

        let rect = Rect::from_min_size(pos, rect_size);
        Self::get_piece_image(piece, set).paint_at(ui, rect);
    }

    fn draw_legal_moves(painter: &Painter, index: u8, game: &Game, origin: Pos2, square_size: f32) {
        let legal_moves_color = Color32::from_rgba_unmultiplied(0x30, 0x30, 0x30, 128);

        for m in game.legal_moves.as_slice() {
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

    fn promotion_box(
        ui: &egui::Ui,
        index: u8,
        game: &Game,
        config: &AppConfig,
        origin: Pos2,
        square_size: f32,
    ) {
        let turn = game.board.get_turn();
        let promotions = [
            PieceType::Queen,
            PieceType::Rook,
            PieceType::Bishop,
            PieceType::Knight,
        ]
        .map(|pt| Piece::make(pt, turn));

        for (i, piece) in promotions.iter().enumerate() {
            if turn == PieceColor::White {
                let coords_raw = Self::coords_from_index(index - (8 * i) as u8, game.perspective);
                let coords = origin
                    + Vec2::new(
                        coords_raw.0 as f32 * square_size,
                        coords_raw.1 as f32 * square_size,
                    );

                Self::draw_piece(ui, coords, *piece, config.piece_set, square_size);
            }
        }
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

        let mut pieces = game.board.get_all_pieces();

        let dragging_index = match game.selected_piece {
            SelectionState::Dragging(index) => Some(index),
            _ => None,
        };

        while pieces != 0 {
            let index = pieces.pop_lsb();

            if dragging_index == Some(index) {
                continue;
            }

            let (x_raw, y_raw) = Self::coords_from_index(index, game.perspective);
            let (x, y) = (x_raw as f32 * square_size, y_raw as f32 * square_size);
            let pos = board_origin + Vec2::new(x, y);

            let piece = game.board.piece_at(index);

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

            if game.moment == GameMoment::Playing {
                if response.drag_started() {
                    game.handle_drag_start(mouse_index);
                } else if response.drag_stopped() {
                    game.handle_drag_end(mouse_index);
                }

                if let SelectionState::Dragging(index) = game.selected_piece {
                    Self::draw_legal_moves(ui.painter(), index, game, board_origin, square_size);

                    if ui.rect_contains_pointer(board_rect) {
                        Self::draw_piece(
                            ui,
                            mouse_pos - Vec2::new(square_size / 2.0, square_size / 2.0),
                            game.board.piece_at(index),
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

        if let GameMoment::Promoting(index) = game.moment {
            Self::promotion_box(ui, index, game, config, board_origin, square_size);
        }

        if let SelectionState::Selected(index) = game.selected_piece {
            Self::draw_legal_moves(ui.painter(), index, game, board_origin, square_size);
        }

        if ui.input(|i| i.key_pressed(egui::Key::F)) {
            game.perspective = game.perspective.flip();
        }
    }
}

struct Game {
    board: Board,
    legal_moves: MoveList,
    selected_piece: SelectionState,
    perspective: PieceColor,
    moment: GameMoment,
}

impl Game {
    pub fn new() -> Self {
        let mut board = fen::load_fen(fen::START_POS).unwrap();
        let legal_moves = generate_legal_moves(&mut board);

        Self {
            board,
            legal_moves,
            selected_piece: SelectionState::None,
            perspective: PieceColor::White,
            moment: GameMoment::Playing,
        }
    }

    pub fn try_move(&mut self, from: u8, to: u8, flag: Option<u16>) {
        let tmp_move = Move::new(from, to, 0);
        if let Some(legal_move) = self.legal_moves.contains(tmp_move) {
            if legal_move.is_promotion() {
                self.moment = GameMoment::Promoting(legal_move.to_square());
                return;
            }

            self.board.do_move(legal_move);

            self.legal_moves = generate_legal_moves(&mut self.board);
            if self.legal_moves.size() == 0 {
                self.moment = GameMoment::Winner(self.board.get_turn().flip());
            }
        }
    }

    pub fn handle_drag_start(&mut self, index: u8) {
        if self.board.piece_at(index) != Piece::NO_PIECE {
            self.selected_piece = SelectionState::Dragging(index);
        }
    }

    pub fn handle_drag_end(&mut self, index: u8) {
        if let SelectionState::Dragging(from) = self.selected_piece {
            self.try_move(from, index, None);
        }

        self.selected_piece = SelectionState::None;
    }

    pub fn handle_click(&mut self, index: u8) {
        match self.selected_piece {
            SelectionState::None => self.selected_piece = SelectionState::Selected(index),
            SelectionState::Selected(from) => {
                self.try_move(from, index, None);
                self.selected_piece = SelectionState::None;
            }
            SelectionState::Dragging(_) => {}
        }
    }
}

enum AppState {
    Playing(Box<Game>),
    MainMenu,
}

#[derive(Default, Clone, Copy, Debug, PartialEq, Eq)]
enum SelectionState {
    #[default]
    None,
    Selected(u8),
    Dragging(u8),
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
