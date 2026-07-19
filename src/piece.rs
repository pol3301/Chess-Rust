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

#[inline(always)]
pub fn get_color(piece: Piece) -> PieceColor {
    match piece & 0b1000 {
        0b0000 => PieceColor::White,
        0b1000 => PieceColor::Black,
        _ => unreachable!(),
    }
}

#[inline(always)]
pub fn get_type(piece: Piece) -> PieceType {
    match piece & 0b111 {
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
pub fn make_piece(piece_type: PieceType, piece_color: PieceColor) -> u8 {
    ((piece_color as u8) << 3) + (piece_type as u8)
}

impl PieceColor {
    #[inline]
    pub fn invert(self) -> Self {
        if self == PieceColor::White {
            PieceColor::Black
        } else {
            PieceColor::White
        }
    }
}
