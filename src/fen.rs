use crate::{
    board::{Board, CastlingRights},
    piece::{PieceColor, PieceType, make_piece},
};

pub const START_POS: &str = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";

#[derive(Debug)]
pub enum FenError {
    InvalidPartCount,
    InvalidPiecePlacement,
    InvalidTurn,
    InvalidRights,
    InvalidEnPassant,
    InvalidHalfMoveClock,
    InvalidFullMoveClock,
    OutOfBounds,
}

pub fn load_fen(fen: &str) -> Result<Board, FenError> {
    let mut board: Board = Board::new();

    let parts: Vec<&str> = fen.split_whitespace().collect();

    if parts.len() > 6 {
        return Err(FenError::InvalidPartCount);
    }

    handle_piece_placement(&mut board, parts[0])?;

    if parts.len() >= 2 {
        handle_turn(&mut board, parts[1])?;
    }

    if parts.len() >= 3 {
        handle_rights(&mut board, parts[2])?;
    }

    if parts.len() >= 4 {
        handle_en_passant(&mut board, parts[3])?;
    }

    if parts.len() >= 5 {
        handle_half_move_clock(&mut board, parts[4])?;
    }

    if parts.len() == 6 {
        handle_full_move_clock(&mut board, parts[5])?;
    }

    Ok(board)
}

fn handle_half_move_clock(board: &mut Board, fen_part: &str) -> Result<(), FenError> {
    //TODO:
    Ok(())
}

fn handle_full_move_clock(board: &mut Board, fen_part: &str) -> Result<(), FenError> {
    //TODO:
    Ok(())
}

fn handle_en_passant(board: &mut Board, fen_part: &str) -> Result<(), FenError> {
    if fen_part == "-" {
        board.set_en_passant(0);
        return Ok(());
    }

    if fen_part.len() != 2 {
        return Err(FenError::InvalidEnPassant);
    }

    let bytes = fen_part.as_bytes();
    let file_char = bytes[0];
    let rank_char = bytes[1];

    if !(b'a'..=b'h').contains(&file_char) || (b'1'..=b'8').contains(&rank_char) {
        return Err(FenError::InvalidEnPassant);
    }

    let file = file_char - b'a';
    let rank = rank_char - b'1';

    let index = file + (rank * 8);
    board.set_en_passant(1u64 << index);

    Ok(())
}

fn handle_rights(board: &mut Board, fen_part: &str) -> Result<(), FenError> {
    if fen_part.len() > 4 {
        return Err(FenError::InvalidRights);
    }

    if fen_part == "-" {
        board.set_rights(CastlingRights::empty());
        return Ok(());
    }

    let mut rights = CastlingRights::empty();

    for c in fen_part.chars() {
        match c {
            'K' => rights |= CastlingRights::WHITE_KING,
            'Q' => rights |= CastlingRights::WHITE_QUEEN,
            'k' => rights |= CastlingRights::BLACK_KING,
            'q' => rights |= CastlingRights::BLACK_QUEEN,
            _ => return Err(FenError::InvalidRights),
        }
    }

    board.set_rights(rights);

    Ok(())
}

fn handle_turn(board: &mut Board, fen_part: &str) -> Result<(), FenError> {
    if fen_part.len() != 1 {
        return Err(FenError::InvalidTurn);
    }

    match fen_part {
        "w" => board.set_turn(PieceColor::White),
        "b" => board.set_turn(PieceColor::Black),
        _ => return Err(FenError::InvalidTurn),
    }

    Ok(())
}

fn handle_piece_placement(board: &mut Board, fen_part: &str) -> Result<(), FenError> {
    let mut col = 0;
    let mut row = 7;

    for c in fen_part.chars() {
        let index = col + row * 8;

        match c {
            'p' => board.put_piece(make_piece(PieceType::Pawn, PieceColor::Black), index),
            'n' => board.put_piece(make_piece(PieceType::Knight, PieceColor::Black), index),
            'b' => board.put_piece(make_piece(PieceType::Bishop, PieceColor::Black), index),
            'r' => board.put_piece(make_piece(PieceType::Rook, PieceColor::Black), index),
            'q' => board.put_piece(make_piece(PieceType::Queen, PieceColor::Black), index),
            'k' => board.put_piece(make_piece(PieceType::King, PieceColor::Black), index),

            'P' => board.put_piece(make_piece(PieceType::Pawn, PieceColor::White), index),
            'N' => board.put_piece(make_piece(PieceType::Knight, PieceColor::White), index),
            'B' => board.put_piece(make_piece(PieceType::Bishop, PieceColor::White), index),
            'R' => board.put_piece(make_piece(PieceType::Rook, PieceColor::White), index),
            'Q' => board.put_piece(make_piece(PieceType::Queen, PieceColor::White), index),
            'K' => board.put_piece(make_piece(PieceType::King, PieceColor::White), index),

            '/' => {
                if row == 0 {
                    return Err(FenError::OutOfBounds);
                }
                col = 0;
                row -= 1;
                continue;
            }
            '0'..='9' => {
                col += c.to_digit(10).unwrap() as u8;
                continue;
            }
            _ => return Err(FenError::InvalidPiecePlacement),
        };

        if col > 8 || row > 7 {
            return Err(FenError::OutOfBounds);
        };

        col += 1;
    }

    Ok(())
}

#[test]
fn load_start_pos() {
    let b = load_fen(START_POS);

    assert!(b.is_ok());
}
