pub type Piece = u8;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PieceColor {
    White = 0,
    Black = 1,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PieceType {
    Pawn = 0,
    Knight = 1,
    Bishop = 2,
    Rook = 3,
    Queen = 4,
    King = 5,
    AllPieces = 6,
    NoPiece = 7,
}

impl PieceColor {
    #[inline(always)]
    pub fn flip(self) -> Self {
        if self == PieceColor::White {
            PieceColor::Black
        } else {
            PieceColor::White
        }
    }
}

pub trait PieceTrait {
    fn get_type(self) -> PieceType;
    fn get_color(self) -> PieceColor;
    fn make(piece_type: PieceType, piece_color: PieceColor) -> Self;
    const NO_PIECE: Piece;
}

impl PieceTrait for Piece {
    #[inline(always)]
    fn get_type(self) -> PieceType {
        match self & 0b111 {
            0 => PieceType::Pawn,
            1 => PieceType::Knight,
            2 => PieceType::Bishop,
            3 => PieceType::Rook,
            4 => PieceType::Queen,
            5 => PieceType::King,
            6 => PieceType::AllPieces,
            7 => PieceType::NoPiece,
            _ => unreachable!(),
        }
    }

    #[inline(always)]
    fn get_color(self) -> PieceColor {
        match self & 0b1000 {
            0b0000 => PieceColor::White,
            0b1000 => PieceColor::Black,
            _ => unreachable!(),
        }
    }

    #[inline(always)]
    fn make(piece_type: PieceType, piece_color: PieceColor) -> Self {
        ((piece_color as u8) << 3) + (piece_type as u8)
    }

    const NO_PIECE: Piece = 7;
}
