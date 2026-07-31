use crate::PieceTrait;
use crate::bitboard::Bitboard;
use crate::moves::{Move, UndoList, UndoMove};
use crate::piece::{Piece, PieceColor, PieceType};
use crate::squares::Squares;
use bitflags::bitflags;

bitflags! {
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
    pub struct CastlingRights: u8 {
        const WHITE_KING = 0b1000;
        const WHITE_QUEEN = 0b0100;
        const BLACK_KING = 0b0010;
        const BLACK_QUEEN = 0b0001;

        const WHITE = Self::WHITE_KING.bits() | Self::WHITE_QUEEN.bits();
        const BLACK = Self::BLACK_KING.bits() | Self::BLACK_QUEEN.bits();

        const QUEEN = Self::WHITE_QUEEN.bits() | Self::BLACK_QUEEN.bits();
        const KING = Self::WHITE_KING.bits() | Self::BLACK_KING.bits();

        const ALL = Self::WHITE.bits() | Self::BLACK.bits();
    }
}

const CASTLING_RIGHTS_MASK: [CastlingRights; 64] = {
    let mut array = [CastlingRights::ALL; 64];

    array[Squares::A1 as usize] = CastlingRights::ALL.difference(CastlingRights::WHITE_QUEEN);
    array[Squares::E1 as usize] = CastlingRights::ALL.difference(CastlingRights::WHITE);
    array[Squares::H1 as usize] = CastlingRights::ALL.difference(CastlingRights::WHITE_KING);

    array[Squares::A8 as usize] = CastlingRights::ALL.difference(CastlingRights::BLACK_QUEEN);
    array[Squares::E8 as usize] = CastlingRights::ALL.difference(CastlingRights::BLACK);
    array[Squares::H8 as usize] = CastlingRights::ALL.difference(CastlingRights::BLACK_KING);

    array
};

#[derive(Debug, PartialEq, Eq)]
pub struct Board {
    piece_bitboards_color: [Bitboard; 2],
    piece_bitboards_type: [Bitboard; 6],

    turn: PieceColor,

    en_passant_bb: Bitboard,

    castling_rights: CastlingRights,

    mailbox: [Piece; 64],

    undo_list: UndoList,

    half_move_counter: u8,
    full_move_counter: u16,
}

impl Default for Board {
    fn default() -> Self {
        Self::new()
    }
}

impl Board {
    pub fn new() -> Self {
        Board {
            piece_bitboards_color: [0u64; 2],
            piece_bitboards_type: [0u64; 6],

            mailbox: [Piece::NO_PIECE; 64],

            en_passant_bb: 0,

            turn: PieceColor::White,
            castling_rights: CastlingRights::empty(),

            undo_list: UndoList::default(),

            half_move_counter: 0,
            full_move_counter: 0,
        }
    }

    pub fn put_piece(&mut self, piece: Piece, index: u8) {
        let new_piece_bitboard: Bitboard = 1u64 << index;

        self.mailbox[index as usize] = piece;

        self.piece_bitboards_color[piece.get_color() as usize] |= new_piece_bitboard;
        self.piece_bitboards_type[piece.get_type() as usize] |= new_piece_bitboard;
    }

    pub fn remove_piece(&mut self, index: u8) {
        let removed_piece_bitboard: Bitboard = 1u64 << index;

        let piece = self.piece_at(index);

        self.piece_bitboards_color[piece.get_color() as usize] &= !removed_piece_bitboard;
        self.piece_bitboards_type[piece.get_type() as usize] &= !removed_piece_bitboard;

        self.mailbox[index as usize] = Piece::NO_PIECE;
    }

    #[inline(always)]
    pub fn set_turn(&mut self, turn: PieceColor) {
        self.turn = turn;
    }

    #[inline(always)]
    pub fn set_rights(&mut self, rights: CastlingRights) {
        self.castling_rights = rights;
    }

    #[inline(always)]
    pub fn get_rights(&self) -> CastlingRights {
        self.castling_rights
    }

    #[inline(always)]
    pub fn set_en_passant(&mut self, new_bb: Bitboard) {
        self.en_passant_bb = new_bb;
    }

    #[inline(always)]
    pub fn get_en_passant_bb(&self) -> Bitboard {
        self.en_passant_bb
    }

    #[inline(always)]
    pub fn piece_at(&self, index: u8) -> Piece {
        self.mailbox[index as usize]
    }

    #[inline(always)]
    pub fn is_empty(&self, index: u8) -> bool {
        self.get_all_pieces() & 1u64 << index == 0
    }

    #[inline(always)]
    pub fn get_turn(&self) -> PieceColor {
        self.turn
    }

    #[inline(always)]
    pub fn get_bb_by_type(&self, piece_type: PieceType) -> Bitboard {
        self.piece_bitboards_type[piece_type as usize]
    }

    #[inline(always)]
    pub fn get_bb_by_color(&self, piece_color: PieceColor) -> Bitboard {
        self.piece_bitboards_color[piece_color as usize]
    }

    #[inline(always)]
    pub fn get_all_pieces(&self) -> Bitboard {
        self.get_bb_by_color(PieceColor::White) | self.get_bb_by_color(PieceColor::Black)
    }

    pub fn set_half_move_counter(&mut self, new_count: u8) {
        self.half_move_counter = new_count;
    }

    pub fn set_full_move_counter(&mut self, new_count: u16) {
        self.full_move_counter = new_count;
    }

    pub fn do_move(&mut self, move_to_make: Move) {
        let mut captured_piece = Piece::NO_PIECE;
        let moving_piece = self.piece_at(move_to_make.from_square());
        let moving_color = moving_piece.get_color();

        let mut undo_move = UndoMove {
            mv: move_to_make,
            taken_piece: captured_piece,
            castling_rights: self.get_rights(),
            en_passant_bb: self.en_passant_bb,
            half_move_counter: self.half_move_counter,
            full_move_counter: self.full_move_counter,
        };

        self.half_move_counter += 1;

        if moving_color == PieceColor::Black {
            self.full_move_counter += 1;
        }

        let piece = if move_to_make.is_promotion() {
            Piece::make(move_to_make.promotion_type(), moving_color)
        } else {
            moving_piece
        };

        if move_to_make.is_en_passant() {
            if moving_color == PieceColor::White {
                self.remove_piece(move_to_make.to_square() - 8);
            } else {
                self.remove_piece(move_to_make.to_square() + 8);
            }
        } else if move_to_make.is_castle() {
            match move_to_make.flags() {
                Move::FLAG_KING_CASTLE => {
                    self.move_piece(
                        move_to_make.from_square() + 3,
                        move_to_make.from_square() + 1,
                    );
                }
                Move::FLAG_QUEEN_CASTLE => {
                    self.move_piece(
                        move_to_make.from_square() - 4,
                        move_to_make.from_square() - 1,
                    );
                }
                _ => unreachable!(),
            }
        }

        self.castling_rights = self.castling_rights
            & CASTLING_RIGHTS_MASK[move_to_make.from_square() as usize]
            & CASTLING_RIGHTS_MASK[move_to_make.to_square() as usize];

        if move_to_make.is_double_pawn() {
            self.set_en_passant(
                1u64 << ((move_to_make.from_square() + move_to_make.to_square()) / 2),
            );
        } else {
            self.set_en_passant(0);
        }

        if move_to_make.is_capture() && !move_to_make.is_en_passant() {
            captured_piece = self.piece_at(move_to_make.to_square());
            self.remove_piece(move_to_make.to_square());
        }

        self.turn = self.turn.flip();
        self.remove_piece(move_to_make.from_square());

        self.put_piece(piece, move_to_make.to_square());

        undo_move.taken_piece = captured_piece;

        self.undo_list.add(undo_move);
    }

    pub fn undo_move(&mut self) {
        let Some(undo_move) = self.undo_list.pop() else {
            return;
        };

        self.half_move_counter = undo_move.half_move_counter;
        self.full_move_counter = undo_move.full_move_counter;

        let moving_color = self.piece_at(undo_move.mv.to_square()).get_color();

        self.move_piece(undo_move.mv.to_square(), undo_move.mv.from_square());

        if undo_move.mv.is_promotion() {
            self.remove_piece(undo_move.mv.from_square());
            self.put_piece(
                Piece::make(PieceType::Pawn, moving_color),
                undo_move.mv.from_square(),
            );
        }

        self.set_en_passant(undo_move.en_passant_bb);
        self.castling_rights = undo_move.castling_rights;

        if undo_move.taken_piece != Piece::NO_PIECE {
            self.put_piece(undo_move.taken_piece, undo_move.mv.to_square());
        }

        if undo_move.mv.is_en_passant() {
            if moving_color == PieceColor::White {
                self.put_piece(
                    Piece::make(PieceType::Pawn, PieceColor::Black),
                    undo_move.mv.to_square() - 8,
                );
            } else {
                self.put_piece(
                    Piece::make(PieceType::Pawn, PieceColor::White),
                    undo_move.mv.to_square() + 8,
                );
            }
        } else if undo_move.mv.is_castle() {
            match undo_move.mv.flags() {
                Move::FLAG_KING_CASTLE => {
                    self.move_piece(
                        undo_move.mv.from_square() + 1,
                        undo_move.mv.from_square() + 3,
                    );
                }
                Move::FLAG_QUEEN_CASTLE => {
                    self.move_piece(
                        undo_move.mv.from_square() - 1,
                        undo_move.mv.from_square() - 4,
                    );
                }
                _ => unreachable!(),
            }
        }

        self.set_turn(self.get_turn().flip());
    }

    #[inline(always)]
    fn move_piece(&mut self, from: u8, to: u8) {
        let piece = self.piece_at(from);
        self.remove_piece(from);
        self.put_piece(piece, to);
    }
}
