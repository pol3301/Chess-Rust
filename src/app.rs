use crate::{
    bitboard::BitboardExt,
    board::Board,
    fen::{START_POS, load_fen},
    move_generator::generate_legal_moves,
    moves::{Move, MoveList},
    piece::{Piece, PieceColor, PieceType, get_color, get_type},
};

const BOARD_HEIGHT: f32 = 800.0;
const BOARD_WIDTH: f32 = 800.0;

#[derive(Default, Clone, Copy, Debug)]
enum SelectionState {
    #[default]
    None,
    Selected(u8),
    Dragging(u8),
}

#[derive(Default, Clone, Copy, Debug)]
pub struct ChessApp {
    board: Board,
    legal_move_list: MoveList,
    state: SelectionState,
}

impl ChessApp {
    const SQUARE_SIZE: f32 = 100.0;

    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        egui_extras::install_image_loaders(&cc.egui_ctx);

        let mut board = load_fen(START_POS).unwrap_or_default();
        let legal_move_list = generate_legal_moves(&mut board);

        Self {
            board,
            legal_move_list,
            ..Default::default()
        }
    }

    fn get_rect_from_square(min: egui::Pos2, file: u8, rank: u8) -> egui::Rect {
        let min_point = min
            + egui::Vec2::new(
                file as f32 * Self::SQUARE_SIZE,
                rank as f32 * Self::SQUARE_SIZE,
            );
        let max_point = min_point + egui::vec2(Self::SQUARE_SIZE, Self::SQUARE_SIZE);

        egui::Rect::from_min_max(min_point, max_point)
    }

    fn screen_pos_to_index(pos: egui::Pos2, board_min: egui::Pos2) -> Option<u8> {
        let x = ((pos.x - board_min.x) / Self::SQUARE_SIZE) as i32;
        let y = ((pos.y - board_min.y) / Self::SQUARE_SIZE) as i32;

        if (0..8).contains(&x) && (0..8).contains(&y) {
            Some((x + (7 - y) * 8) as u8)
        } else {
            None
        }
    }

    fn get_piece_image<'a>(piece: Piece) -> egui::Image<'a> {
        match (get_color(piece), get_type(piece)) {
            (PieceColor::White, PieceType::Pawn) => {
                egui::Image::new(egui::include_image!("../res/wp.png"))
            }
            (PieceColor::White, PieceType::Knight) => {
                egui::Image::new(egui::include_image!("../res/wn.png"))
            }
            (PieceColor::White, PieceType::Bishop) => {
                egui::Image::new(egui::include_image!("../res/wb.png"))
            }
            (PieceColor::White, PieceType::Rook) => {
                egui::Image::new(egui::include_image!("../res/wr.png"))
            }
            (PieceColor::White, PieceType::Queen) => {
                egui::Image::new(egui::include_image!("../res/wq.png"))
            }
            (PieceColor::White, PieceType::King) => {
                egui::Image::new(egui::include_image!("../res/wk.png"))
            }
            (PieceColor::Black, PieceType::Pawn) => {
                egui::Image::new(egui::include_image!("../res/bp.png"))
            }
            (PieceColor::Black, PieceType::Knight) => {
                egui::Image::new(egui::include_image!("../res/bn.png"))
            }
            (PieceColor::Black, PieceType::Bishop) => {
                egui::Image::new(egui::include_image!("../res/bb.png"))
            }
            (PieceColor::Black, PieceType::Rook) => {
                egui::Image::new(egui::include_image!("../res/br.png"))
            }
            (PieceColor::Black, PieceType::Queen) => {
                egui::Image::new(egui::include_image!("../res/bq.png"))
            }
            (PieceColor::Black, PieceType::King) => {
                egui::Image::new(egui::include_image!("../res/bk.png"))
            }
            _ => unreachable!(),
        }
    }

    fn draw_pieces(&self, min: egui::Pos2, ui: &egui::Ui) {
        let mut pieces = self.board.get_all_pieces();
        while pieces != 0 {
            let index = pieces.pop_lsb();
            let is_dragged =
                matches!(self.state, SelectionState::Dragging(dragged_idx) if dragged_idx == index);

            if !is_dragged {
                let square_rect = Self::get_rect_from_square(min, index % 8, 7 - (index / 8));
                Self::get_piece_image(self.board.piece_at(index)).paint_at(ui, square_rect);
            }
        }
    }

    fn draw_legal_moves(&self, min: egui::Pos2, held_piece_index: u8, painter: &egui::Painter) {
        for m in self.legal_move_list.as_slice() {
            if m.from_square() == held_piece_index {
                let file = m.to_square() % 8;
                let rank = 7 - (m.to_square() / 8);

                let color = egui::Color32::from_rgb(255, 0, 0);
                let square_rect = Self::get_rect_from_square(min, file, rank);

                painter.rect_filled(square_rect, 0.0, color);
            }
        }
    }

    fn draw_board(min: egui::Pos2, painter: &egui::Painter) {
        for file in 0..8 {
            for rank in 0..8 {
                let is_light = (file + rank) % 2 == 0;
                let color = if is_light {
                    egui::Color32::from_rgb(240, 217, 181)
                } else {
                    egui::Color32::from_rgb(181, 136, 99)
                };

                let square_rect = Self::get_rect_from_square(min, file, rank);

                painter.rect_filled(square_rect, 0.0, color);
            }
        }
    }

    fn draw_held(&self, ui: &egui::Ui, mouse_pos: egui::Pos2, index: u8) {
        if self.board.is_empty(index) {
            return;
        }

        let piece = self.board.piece_at(index);
        let rect = egui::Rect::from_center_size(
            mouse_pos,
            egui::Vec2::new(Self::SQUARE_SIZE, Self::SQUARE_SIZE),
        );

        Self::get_piece_image(piece).paint_at(ui, rect);
    }
}

impl eframe::App for ChessApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let board_size = egui::Vec2::new(BOARD_WIDTH, BOARD_HEIGHT);
        let avail_space = ui.available_rect_before_wrap();
        let board_rect = egui::Rect::from_center_size(avail_space.center(), board_size);
        let response = ui.allocate_rect(board_rect, egui::Sense::click_and_drag());
        let painter = ui.painter();
        let board_min = response.rect.min;

        Self::draw_board(board_min, painter);

        match self.state {
            SelectionState::Selected(idx) | SelectionState::Dragging(idx) => {
                self.draw_legal_moves(board_min, idx, painter);
            }
            _ => {}
        }

        self.draw_pieces(board_min, ui);

        if let Some(mouse_pos) = response.interact_pointer_pos() {
            let hovered_index = Self::screen_pos_to_index(mouse_pos, board_min);

            if response.drag_started() {
                if let Some(idx) = hovered_index
                    && !self.board.is_empty(idx)
                {
                    self.state = SelectionState::Dragging(idx);
                }
            } else if response.drag_stopped()
                && let SelectionState::Dragging(from_idx) = self.state
            {
                if let Some(to_idx) = hovered_index
                    && from_idx != to_idx
                {
                    let tmp_move = Move::new(from_idx, to_idx, Move::FLAG_QUIET);
                    if let Some(m) = self.legal_move_list.contains(tmp_move) {
                        self.board.do_move(m);
                        self.legal_move_list = generate_legal_moves(&mut self.board);
                    }
                }
                self.state = SelectionState::None;
            }
            if response.clicked()
                && let Some(to_idx) = hovered_index
            {
                match self.state {
                    SelectionState::Selected(from_idx) => {
                        if from_idx != to_idx {
                            let tmp_move = Move::new(from_idx, to_idx, Move::FLAG_CAPTURE);
                            self.board.do_move(tmp_move);
                            self.legal_move_list = generate_legal_moves(&mut self.board);
                        }
                        self.state = SelectionState::None;
                    }
                    _ => {
                        if !self.board.is_empty(to_idx) {
                            self.state = SelectionState::Selected(to_idx);
                        } else {
                            self.state = SelectionState::None;
                        }
                    }
                }
            }
            if response.dragged()
                && let SelectionState::Dragging(idx) = self.state
            {
                self.draw_held(ui, mouse_pos, idx);
            }
        }
    }
}
