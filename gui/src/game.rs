use chess_engine::{
    Board, Move, MoveList, Piece, PieceColor, PieceTrait, PieceType, fen, generate_legal_moves,
    piece,
};

#[derive(Default, Clone, Copy, Debug, PartialEq, Eq)]
pub enum SelectionState {
    #[default]
    None,
    Selected(u8),
    Dragging(u8),
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum GameMoment {
    Playing,
    Winner(PieceColor),
    Draw,
    Promoting(u8, u8),
}

pub struct Game {
    board: Board,
    legal_moves: MoveList,
    selected_piece: SelectionState,
    moment: GameMoment,
    pub perspective: PieceColor,
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

    pub fn board(&self) -> &Board {
        &self.board
    }

    pub fn legal_moves(&self) -> &MoveList {
        &self.legal_moves
    }

    pub fn selected_piece(&self) -> SelectionState {
        self.selected_piece
    }

    pub fn moment(&self) -> GameMoment {
        self.moment
    }

    pub fn undo(&mut self) {
        self.board.undo_move();
    }

    pub fn try_move(&mut self, from: u8, to: u8, promotion_choice: Option<PieceType>) {
        let move_choices: Vec<Move> = self
            .legal_moves
            .as_slice()
            .iter()
            .filter(|m| m.to_square() == to && m.from_square() == from)
            .copied()
            .collect();

        if move_choices.is_empty() {
            return;
        }

        if move_choices.len() == 1 {
            self.board.do_move(move_choices[0]);
            self.legal_moves = generate_legal_moves(&mut self.board);
            return;
        }

        if let Some(promotion_piece) = promotion_choice {
            let chosen_move = move_choices
                .into_iter()
                .find(|m| m.promotion_type() == Some(promotion_piece));

            if let Some(valid_move) = chosen_move {
                self.board.do_move(valid_move);
                self.moment = GameMoment::Playing;
            }
        } else {
            self.moment = GameMoment::Promoting(from, to);
            self.selected_piece = SelectionState::None;
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
