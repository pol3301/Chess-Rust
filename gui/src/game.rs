use chess_engine::{
    Board, Move, MoveList, Piece, PieceColor, PieceTrait, PieceType, fen, generate_legal_moves,
};
use networking::Message;
use tokio::sync::mpsc::Sender;

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

#[derive(Clone)]
pub struct OnlineGame {
    pub color_us: PieceColor,
    pub tx: Sender<Message>,
}

pub struct Game {
    board: Board,
    legal_moves: MoveList,
    selected_piece: SelectionState,
    moment: GameMoment,
    online: Option<OnlineGame>,
    pub perspective: PieceColor,
}

impl Game {
    pub fn new(online: Option<OnlineGame>) -> Self {
        let mut board = fen::load_fen(fen::START_POS).unwrap();
        let legal_moves = generate_legal_moves(&mut board);

        Self {
            board,
            legal_moves,
            selected_piece: SelectionState::None,
            perspective: PieceColor::White,
            moment: GameMoment::Playing,
            online,
        }
    }

    pub fn board(&self) -> &Board {
        &self.board
    }

    pub fn online(&self) -> &Option<OnlineGame> {
        &self.online
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

    pub fn try_move(
        &mut self,
        from: u8,
        to: u8,
        promotion_choice: Option<PieceType>,
    ) -> Result<Move, &str> {
        let move_choices: Vec<Move> = self
            .legal_moves
            .as_slice()
            .iter()
            .filter(|m| m.to_square() == to && m.from_square() == from)
            .copied()
            .collect();

        if move_choices.is_empty() {
            return Err("Found no matching legal move");
        }

        if move_choices.len() == 1 {
            self.board.do_move(move_choices[0]);
            self.legal_moves = generate_legal_moves(&mut self.board);
            return Ok(move_choices[0]);
        }

        if let Some(promotion_piece) = promotion_choice {
            let chosen_move = move_choices
                .into_iter()
                .find(|m| m.promotion_type() == Some(promotion_piece));

            if let Some(valid_move) = chosen_move {
                self.board.do_move(valid_move);
                self.legal_moves = generate_legal_moves(&mut self.board);
                self.moment = GameMoment::Playing;
                Ok(valid_move)
            } else {
                Err("Found no matching legal move")
            }
        } else {
            self.moment = GameMoment::Promoting(from, to);
            self.selected_piece = SelectionState::None;
            Ok(Move::NULL)
        }
    }

    pub fn handle_drag_start(&mut self, index: u8) {
        if self.board.piece_at(index) != Piece::NO_PIECE {
            self.selected_piece = SelectionState::Dragging(index);
        }
    }

    pub fn handle_drag_end(&mut self, index: u8) {
        if let SelectionState::Dragging(from) = self.selected_piece {
            if let Some(us) = self.online.as_ref().map(|o| &o.color_us)
                && *us != self.board.piece_at(from).get_color()
            {
                self.selected_piece = SelectionState::None;
                return;
            }

            match self.try_move(from, index, None) {
                Ok(m) => {
                    if let Some(tx) = self.online.as_ref().map(|o| &o.tx) {
                        tx.try_send(Message::Move(m.as_bytes()))
                            .expect("Failed to send move within the app");
                    }
                }
                Err(e) => eprintln!("{}", e),
            }
        }

        self.selected_piece = SelectionState::None;
    }

    pub fn handle_select(&mut self, index: u8) {
        match self.selected_piece {
            SelectionState::None => self.selected_piece = SelectionState::Selected(index),
            SelectionState::Selected(from) => {
                if let Some(us) = self.online.as_ref().map(|o| &o.color_us)
                    && *us != self.board.piece_at(from).get_color()
                {
                    self.selected_piece = SelectionState::None;
                    return;
                }

                match self.try_move(from, index, None) {
                    Ok(m) => {
                        if let Some(tx) = self.online.as_ref().map(|o| &o.tx) {
                            tx.try_send(Message::Move(m.as_bytes()))
                                .expect("Failed to send move within the app");
                        }
                    }
                    Err(e) => eprintln!("{}", e),
                }
                self.selected_piece = SelectionState::None;
            }
            SelectionState::Dragging(_) => {}
        }
    }
}
