use crate::{
    bitboard::Bitboard,
    board::CastlingRights,
    piece::{Piece, PieceType},
};

use core::fmt;
use std::mem::MaybeUninit;

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct Move(pub u16);

impl fmt::Display for Move {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.0 == 0 {
            return Ok(());
        }

        write!(
            f,
            "From: {} To: {} Flags: {}",
            self.from_square(),
            self.to_square(),
            self.flags()
        )
    }
}

impl Move {
    const FROM_MASK: u16 = 0b111111;
    const TO_MASK: u16 = 0b111111 << 6;
    const FLAG_MASK: u16 = 0b1111 << 12;

    pub const FLAG_QUIET: u16 = 0b0000;
    pub const FLAG_DOUBLE_PAWN_PUSH: u16 = 0b0001;
    pub const FLAG_KING_CASTLE: u16 = 0b0010;
    pub const FLAG_QUEEN_CASTLE: u16 = 0b0011;

    pub const FLAG_CAPTURE: u16 = 0b0100;
    pub const FLAG_EN_PASSANT: u16 = 0b0101;

    pub const FLAG_PROMOTE_KNIGHT: u16 = 0b1000;
    pub const FLAG_PROMOTE_BISHOP: u16 = 0b1001;
    pub const FLAG_PROMOTE_ROOK: u16 = 0b1010;
    pub const FLAG_PROMOTE_QUEEN: u16 = 0b1011;

    pub const FLAG_PROMOTE_KNIGHT_CAPTURE: u16 = 0b1100;
    pub const FLAG_PROMOTE_BISHOP_CAPTURE: u16 = 0b1101;
    pub const FLAG_PROMOTE_ROOK_CAPTURE: u16 = 0b1110;
    pub const FLAG_PROMOTE_QUEEN_CAPTURE: u16 = 0b1111;

    pub const NULL: Move = Move(0);

    #[inline]
    pub fn as_bytes(self) -> u16 {
        self.0
    }

    #[inline]
    pub fn from_bytes(m: u16) -> Move {
        Move(m)
    }

    #[inline(always)]
    pub fn new(from: u8, to: u8, flag: u16) -> Self {
        Self(flag << 12 | (from as u16) | ((to as u16) << 6))
    }

    #[inline(always)]
    pub fn from_square(self) -> u8 {
        (self.0 & Move::FROM_MASK) as u8
    }

    #[inline(always)]
    pub fn to_square(self) -> u8 {
        ((self.0 & Move::TO_MASK) >> 6) as u8
    }

    #[inline(always)]
    pub fn flags(self) -> u16 {
        (self.0 & Move::FLAG_MASK) >> 12
    }

    #[inline(always)]
    pub fn is_en_passant(self) -> bool {
        self.flags() == Move::FLAG_EN_PASSANT
    }

    #[inline(always)]
    pub fn is_double_pawn(self) -> bool {
        self.flags() == Move::FLAG_DOUBLE_PAWN_PUSH
    }

    #[inline(always)]
    pub fn is_capture(self) -> bool {
        (self.flags() & Move::FLAG_CAPTURE) != 0
    }

    #[inline(always)]
    pub fn is_castle(self) -> bool {
        self.flags() == Move::FLAG_KING_CASTLE || self.flags() == Move::FLAG_QUEEN_CASTLE
    }

    #[inline(always)]
    pub fn is_promotion(self) -> bool {
        (self.flags() & 0b1000) != 0
    }

    #[inline(always)]
    pub fn promotion_type(self) -> Option<PieceType> {
        match self.flags() & !Move::FLAG_CAPTURE {
            Move::FLAG_PROMOTE_KNIGHT => Some(PieceType::Knight),
            Move::FLAG_PROMOTE_BISHOP => Some(PieceType::Bishop),
            Move::FLAG_PROMOTE_ROOK => Some(PieceType::Rook),
            Move::FLAG_PROMOTE_QUEEN => Some(PieceType::Queen),
            _ => None,
        }
    }
}

#[repr(align(64))]
#[derive(Clone, Copy, Debug)]
pub struct MoveList {
    moves: [MaybeUninit<Move>; 256],
    len: u8,
}

impl Default for MoveList {
    fn default() -> Self {
        Self::new()
    }
}

impl MoveList {
    pub fn new() -> Self {
        Self {
            moves: [MaybeUninit::uninit(); 256],
            len: 0,
        }
    }

    #[inline(always)]
    pub fn size(&self) -> u8 {
        self.len
    }

    #[inline(always)]
    pub fn move_at(&self, index: u8) -> Move {
        debug_assert!(index < self.len, "Attempted to read uninitialized move");

        unsafe { self.moves[index as usize].assume_init() }
    }

    #[inline(always)]
    pub fn add(&mut self, mv: Move) {
        self.moves[self.len as usize].write(mv);
        self.len += 1;
    }

    #[inline(always)]
    pub fn as_slice(&self) -> &[Move] {
        unsafe { std::slice::from_raw_parts(self.moves.as_ptr() as *const Move, self.len as usize) }
    }

    pub fn contains(&self, move_to_find: Move) -> Option<Move> {
        for m in self.as_slice() {
            if m.from_square() == move_to_find.from_square()
                && m.to_square() == move_to_find.to_square()
            {
                return Some(*m);
            }
        }

        None
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct UndoMove {
    pub mv: Move,
    pub taken_piece: Piece,
    pub castling_rights: CastlingRights,
    pub en_passant_bb: Bitboard,
    pub half_move_counter: u8,
    pub full_move_counter: u16,
}

#[derive(Debug, PartialEq, Eq)]
pub struct UndoList {
    list: [UndoMove; 256],
    size: usize,
}

impl Default for UndoList {
    fn default() -> Self {
        Self {
            list: [UndoMove::default(); 256],
            size: Default::default(),
        }
    }
}

impl UndoList {
    pub fn pop(&mut self) -> Option<UndoMove> {
        if self.size == 0 {
            return None;
        }

        self.size -= 1;
        Some(self.list[self.size])
    }

    pub fn add(&mut self, undo_move: UndoMove) {
        self.list[self.size] = undo_move;
        self.size += 1;
    }
}
