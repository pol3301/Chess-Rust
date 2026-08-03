use crate::{
    bitboard::{A_FILE, Bitboard, BitboardExt, H_FILE, RANK_2, RANK_3, RANK_6, RANK_7},
    board::{Board, CastlingRights},
    fen::{START_POS, load_fen},
    moves::{Move, MoveList},
    piece::{PieceColor, PieceType},
    squares::Squares,
};

#[derive(Clone, Copy)]
struct MagicInfo {
    mask: Bitboard,
    magic: u64,
    offset: u64,
    shift: i32,
}

impl MagicInfo {
    pub const fn new() -> Self {
        MagicInfo {
            mask: 0,
            magic: 0,
            offset: 0,
            shift: 0,
        }
    }
}

const ROOK_OFFSETS: [usize; 64] = {
    let mut array = [0; 64];
    let mut offset_count = 0;

    let mut i = 0;
    while i < 64 {
        array[i] = offset_count as usize;
        let blocker = get_blocker_mask(i as u8, PieceType::Rook);
        offset_count += 1 << blocker.count_ones();

        i += 1;
    }

    array
};

const BISHOP_OFFSETS: [usize; 64] = {
    let mut array = [0; 64];
    let mut offset_count = 0;

    let mut i = 0;
    while i < 64 {
        array[i] = offset_count as usize;
        let blocker = get_blocker_mask(i as u8, PieceType::Bishop);
        offset_count += 1 << blocker.count_ones();

        i += 1;
    }

    array
};

//Magic numbers
#[rustfmt::skip]
const ROOK_MAGICS: [u64; 64] = [ 4935945329314512900, 1170936040690548800, 2954396541018767624, 936784113040410660, 720611640449457200, 9367496072596852737, 864699941756338697, 36028938753036928, 90212732187443360, 844699877277954, 2533347949543489, 3769654175621711872, 608127237073404928, 162411074455864320, 4785285057609732, 1191624317134177536, 1765552341242626048, 10380867784713179138, 282575564177424, 290511863047848032, 53876204044556, 8800925126720, 576465150487465985, 36039792173880068, 9376564795077066784, 9183126483914752, 326863920513941520, 153131185572151424, 11540495521056768, 9228440003776088080, 10016287119264317540, 576742596647747844, 36030171421089794, 281749871403648, 1443544555507945472, 292736278166774016, 5766437127571130368, 18177128386265600, 112598791139365392, 83457331676841216, 4611756389598265348, 4620693356463308800, 4613480425699999808, 580546436595720, 9226842095686287488, 43347322776191012, 17696456704010, 145245487530901505, 222435744227197184, 649714808358306304, 2310504939052368768, 4972256603371733248, 2308095378254266624, 562995251937408, 576463260850594816, 176489889008128, 5802924477880729859, 594625036585992226, 18023332042522885, 1153203014549377025, 9314007014520922370, 759138047119065217, 72339071230410757, 290486595541557378, ];
#[rustfmt::skip]
const BISHOP_MAGICS: [u64; 64] = [45071610159833120, 577604253527377920, 1158552112276310211, 1200668846067712, 298385603536486432, 1172347685743657508, 4611905930702880768, 72200535113155584, 667113304195597312, 18049875543343168, 2738224861687087109, 69845394250530848, 4415764856841, 577591256549965824, 288241923240501248, 10142447225085987, 1135529827763233, 18718154740007968, 7516648808768602176, 4910049502365876228, 1230609173723545744, 14160584999853688832, 6830312295236608, 562960163805224, 37172291328587904, 9225799759133149824, 9847419669605255200, 9302189986860367876, 73466070581657600, 2598578084546379920, 2308097439712741384, 594758824816182272, 1335345052452921890, 1585633274373120, 10699493712599044, 4616262187974393984, 38282813035819024, 141841295609860, 15277063195720713216, 10982595632048906753, 1450445021827318272, 1200735553389698, 4629719143128498432, 36171742189715968, 144185625842553088, 83606933039227136, 4649975416788877380, 9243647105388515456, 2306150907400159232, 291409254940672, 108931918737702976, 324613244035792960, 324259448132796483, 360340755606340096, 2319358274934521856, 2260600403009536, 74769005355008, 19796596101122, 81681623888170000, 37225074277242890, 596766812488606212, 90641543988512, 1243582972852568580, 18015807294423104];

const fn calc_shift(index: u8, piece_type: PieceType) -> i32 {
    (64 - get_blocker_mask(index, piece_type).count_ones()) as i32
}

const ROOK_MAGIC_INFO: [MagicInfo; 64] = {
    let mut i = 0;
    let mut array = [MagicInfo::new(); 64];

    while i < 64 {
        array[i] = MagicInfo {
            mask: get_blocker_mask(i as u8, PieceType::Rook),
            magic: ROOK_MAGICS[i],
            offset: ROOK_OFFSETS[i] as u64,
            shift: calc_shift(i as u8, PieceType::Rook),
        };

        i += 1;
    }

    array
};

const BISHOP_MAGIC_INFO: [MagicInfo; 64] = {
    let mut i = 0;
    let mut array = [MagicInfo::new(); 64];

    while i < 64 {
        array[i] = MagicInfo {
            mask: get_blocker_mask(i as u8, PieceType::Bishop),
            magic: BISHOP_MAGICS[i],
            offset: BISHOP_OFFSETS[i] as u64,
            shift: calc_shift(i as u8, PieceType::Bishop),
        };

        i += 1;
    }

    array
};

const fn get_blocker_mask(index: u8, piece_type: PieceType) -> Bitboard {
    let start_x = (index % 8) as i32;
    let start_y = (index / 8) as i32;

    let mut mask = 0;

    let (dx_arr, dy_arr) = match piece_type {
        PieceType::Rook => ([0, 0, -1, 1], [-1, 1, 0, 0]),
        PieceType::Bishop => ([-1, 1, -1, 1], [-1, 1, 1, -1]),
        _ => unreachable!(),
    };

    let mut dir_idx = 0;

    while dir_idx < dx_arr.len() {
        let dx = dx_arr[dir_idx];
        let dy = dy_arr[dir_idx];

        let mut x = start_x + dx;
        let mut y = start_y + dy;

        while x >= 0 && x < 8 && y >= 0 && y < 8 {
            let hit_x_edge = start_x != x && (x < 1 || x >= 7);
            let hit_y_edge = start_y != y && (y < 1 || y >= 7);

            if hit_x_edge || hit_y_edge {
                break;
            }

            let curr_idx = (x + (y * 8)) as u8;
            mask |= 1u64 << curr_idx;

            x += dx;
            y += dy;
        }

        dir_idx += 1;
    }

    mask
}

include!(concat!(env!("OUT_DIR"), "/rooks_attack_table.rs"));
include!(concat!(env!("OUT_DIR"), "/bishops_attack_table.rs"));

static KNIGHT_ATTACKS: [Bitboard; 64] = {
    let mut array = [0; 64];

    let (dx_arr, dy_arr) = { ([1, -1, 1, -1, 2, -2, 2, -2], [2, 2, -2, -2, 1, 1, -1, -1]) };

    let mut i = 0;
    while i < 64 {
        let x = i % 8;
        let y = i / 8;

        let mut moves: Bitboard = 0;

        let mut dir_index = 0;
        while dir_index < 8 {
            let dx = dx_arr[dir_index];
            let dy = dy_arr[dir_index];

            let curr_x = x + dx;
            let curr_y = y + dy;

            let curr_idx = curr_y * 8 + curr_x;

            if (curr_x < 8 && curr_x >= 0) && (curr_y < 8 && curr_y >= 0) {
                moves |= 1u64 << curr_idx;
            }

            dir_index += 1;
        }

        array[i as usize] = moves;

        i += 1;
    }

    array
};

static KING_ATTACKS: [Bitboard; 64] = {
    let mut array = [0; 64];

    let (dx_arr, dy_arr) = { ([1, 1, 1, -1, -1, -1, 0, 0], [1, -1, 0, 1, -1, 0, 1, -1]) };

    let mut i = 0;
    while i < 64 {
        let x = i % 8;
        let y = i / 8;

        let mut moves: Bitboard = 0;

        let mut dir_index = 0;
        while dir_index < 8 {
            let dx = dx_arr[dir_index];
            let dy = dy_arr[dir_index];

            let curr_x = x + dx;
            let curr_y = y + dy;

            let curr_idx = curr_y * 8 + curr_x;

            if (curr_x < 8 && curr_x >= 0) && (curr_y < 8 && curr_y >= 0) {
                moves |= 1u64 << curr_idx;
            }

            dir_index += 1;
        }

        array[i as usize] = moves;

        i += 1;
    }

    array
};

fn rook_lookup(index: u8, blocker: Bitboard) -> Bitboard {
    let info = &ROOK_MAGIC_INFO[index as usize];
    let blocker = info.mask & blocker;
    ROOK_ATTACKS[((blocker.wrapping_mul(info.magic) >> info.shift) + info.offset) as usize]
}

fn bishop_lookup(index: u8, blocker: Bitboard) -> Bitboard {
    let info = &BISHOP_MAGIC_INFO[index as usize];
    let blocker = info.mask & blocker;
    BISHOP_ATTACKS[((blocker.wrapping_mul(info.magic) >> info.shift) + info.offset) as usize]
}

pub fn gen_pawn_moves<const TACTICALS: bool, const QUIETS: bool>(
    pawns: Bitboard,
    color: PieceColor,
    enemies: Bitboard,
    all_pieces: Bitboard,
    move_list: &mut MoveList,
    en_passant_bb: Bitboard,
) {
    macro_rules! emit {
        ($bb:expr, $offset:expr, $flag:expr) => {
            let mut b = $bb;
            while b != 0 {
                let to = b.pop_lsb();
                let from = (to as i32 + $offset) as u8;

                let mv = Move::new(from, to, $flag);
                move_list.add(mv);
            }
        };
    }

    macro_rules! emit_promotions {
        ($bb:expr, $offset:expr, $base_flag:expr) => {
            let mut b = $bb;
            while b != 0 {
                let to = b.pop_lsb();
                let from = (to as i32 + $offset) as u8;

                for i in (0..4).rev() {
                    let mv = Move::new(from, to, $base_flag + i);
                    move_list.add(mv);
                }
            }
        };
    }

    if TACTICALS {
        if color == PieceColor::White {
            let pawns_no_promotion = pawns & !RANK_7;
            let pawns_promotion = pawns & RANK_7;

            let moved_pawns_right = ((pawns_no_promotion & !H_FILE) << 9) & enemies;
            let moved_pawns_left = ((pawns_no_promotion & !A_FILE) << 7) & enemies;

            emit!(moved_pawns_left, -7, Move::FLAG_CAPTURE);
            emit!(moved_pawns_right, -9, Move::FLAG_CAPTURE);

            let moved_pawns_right = ((pawns_no_promotion & !H_FILE) << 9) & en_passant_bb;
            let moved_pawns_left = ((pawns_no_promotion & !A_FILE) << 7) & en_passant_bb;

            emit!(moved_pawns_left, -7, Move::FLAG_EN_PASSANT);
            emit!(moved_pawns_right, -9, Move::FLAG_EN_PASSANT);

            let moved_pawns_right = ((pawns_promotion & !H_FILE) << 9) & enemies;
            let moved_pawns_left = ((pawns_promotion & !A_FILE) << 7) & enemies;

            emit_promotions!(moved_pawns_left, -7, Move::FLAG_PROMOTE_KNIGHT_CAPTURE);
            emit_promotions!(moved_pawns_right, -9, Move::FLAG_PROMOTE_KNIGHT_CAPTURE);

            let pawns_promoted = (pawns_promotion << 8) & !all_pieces;

            emit_promotions!(pawns_promoted, -8, Move::FLAG_PROMOTE_KNIGHT);
        } else {
            let pawns_no_promotion = pawns & !RANK_2;
            let pawns_promotion = pawns & RANK_2;

            let moved_pawns_right = ((pawns_no_promotion & !H_FILE) >> 7) & enemies;
            let moved_pawns_left = ((pawns_no_promotion & !A_FILE) >> 9) & enemies;

            emit!(moved_pawns_right, 7, Move::FLAG_CAPTURE);
            emit!(moved_pawns_left, 9, Move::FLAG_CAPTURE);

            let moved_pawns_right = ((pawns_no_promotion & !H_FILE) >> 7) & en_passant_bb;
            let moved_pawns_left = ((pawns_no_promotion & !A_FILE) >> 9) & en_passant_bb;

            emit!(moved_pawns_right, 7, Move::FLAG_EN_PASSANT);
            emit!(moved_pawns_left, 9, Move::FLAG_EN_PASSANT);

            let moved_pawns_right = ((pawns_promotion & !H_FILE) >> 7) & enemies;
            let moved_pawns_left = ((pawns_promotion & !A_FILE) >> 9) & enemies;

            emit_promotions!(moved_pawns_right, 7, Move::FLAG_PROMOTE_KNIGHT_CAPTURE);
            emit_promotions!(moved_pawns_left, 9, Move::FLAG_PROMOTE_KNIGHT_CAPTURE);

            let pawns_promoted = (pawns_promotion >> 8) & !all_pieces;

            emit_promotions!(pawns_promoted, 8, Move::FLAG_PROMOTE_KNIGHT);
        }
    }
    if QUIETS {
        if color == PieceColor::White {
            let safe_pawns = pawns & !RANK_7;

            let moved_pawns_single = (safe_pawns << 8) & !all_pieces;
            let moved_pawns_double = ((moved_pawns_single & RANK_3) << 8) & !all_pieces;

            emit!(moved_pawns_single, -8, Move::FLAG_QUIET);
            emit!(moved_pawns_double, -16, Move::FLAG_DOUBLE_PAWN_PUSH);
        } else {
            let safe_pawns = pawns & !RANK_2;

            let moved_pawns_single = (safe_pawns >> 8) & !all_pieces;
            let moved_pawns_double = ((moved_pawns_single & RANK_6) >> 8) & !all_pieces;

            emit!(moved_pawns_single, 8, Move::FLAG_QUIET);
            emit!(moved_pawns_double, 16, Move::FLAG_DOUBLE_PAWN_PUSH);
        }
    }
}

pub fn generate_piece_moves<const CAPTURES: bool, const QUIETS: bool>(
    index: u8,
    attacks: Bitboard,
    enemies: Bitboard,
    all_pieces: Bitboard,
    move_list: &mut MoveList,
) {
    if CAPTURES {
        let mut bb = attacks & enemies;
        while bb != 0 {
            move_list.add(Move::new(index, bb.pop_lsb(), Move::FLAG_CAPTURE));
        }
    }

    if QUIETS {
        let mut bb = attacks & !all_pieces;
        while bb != 0 {
            move_list.add(Move::new(index, bb.pop_lsb(), Move::FLAG_QUIET));
        }
    }
}

const WHITE_KING_EMPTY: Bitboard = 0x60;
const WHITE_QUEEN_EMPTY: Bitboard = 0x0E;
const BLACK_KING_EMPTY: Bitboard = 0x60 << (8 * 7);
const BLACK_QUEEN_EMPTY: Bitboard = 0x0E << (8 * 7);

pub fn generate_castles(
    board: &Board,
    color: PieceColor,
    castling_rights: CastlingRights,
    all_pieces: Bitboard,
    move_list: &mut MoveList,
) {
    match color {
        PieceColor::White => {
            if (castling_rights & CastlingRights::WHITE_KING) != CastlingRights::empty()
                && (all_pieces & WHITE_KING_EMPTY) == 0
                && !is_square_attacked(board, Squares::E1 as u8, PieceColor::Black)
                && !is_square_attacked(board, Squares::F1 as u8, PieceColor::Black)
                && !is_square_attacked(board, Squares::G1 as u8, PieceColor::Black)
            {
                move_list.add(Move::new(
                    Squares::E1 as u8,
                    Squares::G1 as u8,
                    Move::FLAG_KING_CASTLE,
                ));
            }

            if (castling_rights & CastlingRights::WHITE_QUEEN) != CastlingRights::empty()
                && (all_pieces & WHITE_QUEEN_EMPTY) == 0
                && !is_square_attacked(board, Squares::E1 as u8, PieceColor::Black)
                && !is_square_attacked(board, Squares::D1 as u8, PieceColor::Black)
                && !is_square_attacked(board, Squares::C1 as u8, PieceColor::Black)
            {
                move_list.add(Move::new(
                    Squares::E1 as u8,
                    Squares::C1 as u8,
                    Move::FLAG_QUEEN_CASTLE,
                ));
            }
        }
        PieceColor::Black => {
            if (castling_rights & CastlingRights::BLACK_KING) != CastlingRights::empty()
                && (all_pieces & BLACK_KING_EMPTY) == 0
                && !is_square_attacked(board, Squares::E8 as u8, PieceColor::White)
                && !is_square_attacked(board, Squares::F8 as u8, PieceColor::White)
                && !is_square_attacked(board, Squares::G8 as u8, PieceColor::White)
            {
                move_list.add(Move::new(
                    Squares::E8 as u8,
                    Squares::G8 as u8,
                    Move::FLAG_KING_CASTLE,
                ));
            }

            if (castling_rights & CastlingRights::BLACK_QUEEN) != CastlingRights::empty()
                && (all_pieces & BLACK_QUEEN_EMPTY) == 0
                && !is_square_attacked(board, Squares::E8 as u8, PieceColor::White)
                && !is_square_attacked(board, Squares::D8 as u8, PieceColor::White)
                && !is_square_attacked(board, Squares::C8 as u8, PieceColor::White)
            {
                move_list.add(Move::new(
                    Squares::E8 as u8,
                    Squares::C8 as u8,
                    Move::FLAG_QUEEN_CASTLE,
                ));
            }
        }
    }
}

pub fn generate_moves<const CAPTURES: bool, const QUIETS: bool>(
    board: &Board,
    turn: PieceColor,
) -> MoveList {
    let mut ml = MoveList::new();

    let friendlies = board.get_bb_by_color(turn);

    let enemies = board.get_bb_by_color(turn.flip());

    let all_pieces = board.get_all_pieces();

    let pieces = board.get_bb_by_type(PieceType::Pawn) & friendlies;

    gen_pawn_moves::<CAPTURES, QUIETS>(
        pieces,
        turn,
        enemies,
        all_pieces,
        &mut ml,
        board.get_en_passant_bb(),
    );

    generate_castles(board, turn, board.get_rights(), all_pieces, &mut ml);

    macro_rules! gen_move_loop {
        ($pieces:expr, $attacks:expr) => {
            let mut pieces = $pieces;
            while pieces != 0 {
                let from = pieces.pop_lsb();
                let attacks = $attacks(from, all_pieces);
                generate_piece_moves::<CAPTURES, QUIETS>(
                    from, attacks, enemies, all_pieces, &mut ml,
                );
            }
        };
    }

    let pieces = board.get_bb_by_type(PieceType::Rook) & friendlies;
    gen_move_loop!(pieces, rook_lookup);

    let pieces = board.get_bb_by_type(PieceType::Bishop) & friendlies;
    gen_move_loop!(pieces, bishop_lookup);

    let pieces = board.get_bb_by_type(PieceType::Queen) & friendlies;
    gen_move_loop!(pieces, |from, occ| {
        rook_lookup(from, occ) | bishop_lookup(from, occ)
    });

    let pieces = board.get_bb_by_type(PieceType::Knight) & friendlies;
    gen_move_loop!(pieces, |from, _| {
        KNIGHT_ATTACKS[from as usize] & !friendlies
    });

    let pieces = board.get_bb_by_type(PieceType::King) & friendlies;
    gen_move_loop!(pieces, |from, _| {
        KING_ATTACKS[from as usize] & !friendlies
    });

    ml
}

fn is_square_attacked(board: &Board, square: u8, attacker_color: PieceColor) -> bool {
    let all_pieces = board.get_all_pieces();
    let sq_bb = 1u64 << square;

    let enemy_knights =
        board.get_bb_by_type(PieceType::Knight) & board.get_bb_by_color(attacker_color);
    if (KNIGHT_ATTACKS[square as usize] & enemy_knights) != 0 {
        return true;
    }

    let enemy_king = board.get_bb_by_type(PieceType::King) & board.get_bb_by_color(attacker_color);
    if (KING_ATTACKS[square as usize] & enemy_king) != 0 {
        return true;
    }

    let enemy_rooks_queens = (board.get_bb_by_type(PieceType::Rook)
        | board.get_bb_by_type(PieceType::Queen))
        & board.get_bb_by_color(attacker_color);
    if (rook_lookup(square, all_pieces) & enemy_rooks_queens) != 0 {
        return true;
    }

    let enemy_bishops_queens = (board.get_bb_by_type(PieceType::Bishop)
        | board.get_bb_by_type(PieceType::Queen))
        & board.get_bb_by_color(attacker_color);
    if (bishop_lookup(square, all_pieces) & enemy_bishops_queens) != 0 {
        return true;
    }

    let enemy_pawns = board.get_bb_by_type(PieceType::Pawn) & board.get_bb_by_color(attacker_color);
    if attacker_color == PieceColor::White {
        let attacks_left = (enemy_pawns & !A_FILE) << 7;
        let attacks_right = (enemy_pawns & !H_FILE) << 9;
        if ((attacks_left | attacks_right) & sq_bb) != 0 {
            return true;
        }
    } else {
        let attacks_right = (enemy_pawns & !H_FILE) >> 7;
        let attacks_left = (enemy_pawns & !A_FILE) >> 9;
        if ((attacks_left | attacks_right) & sq_bb) != 0 {
            return true;
        }
    }

    false
}

pub fn is_in_check(board: &Board, color: PieceColor) -> bool {
    let mut king_bb = board.get_bb_by_type(PieceType::King) & board.get_bb_by_color(color);
    if king_bb == 0 {
        return true;
    }
    let king_sq = king_bb.pop_lsb();

    let enemy_color = color.flip();

    is_square_attacked(board, king_sq, enemy_color)
}

pub fn generate_legal_moves(board: &mut Board) -> MoveList {
    let my_color = board.get_turn();
    let ml = generate_moves::<true, true>(board, my_color);
    let mut ml_final = MoveList::new();

    for i in 0..ml.size() {
        let candidate_move = ml.move_at(i);
        board.do_move(candidate_move);

        if !is_in_check(board, my_color) {
            ml_final.add(candidate_move);
        }

        board.undo_move();
    }

    ml_final
}

fn perft(depth: u32, board: &mut Board) -> u64 {
    if depth == 0 {
        return 1;
    }

    let turn = board.get_turn();
    let ml = generate_moves::<true, true>(board, turn);

    if depth == 1 {
        let mut legal_moves = 0;
        for m in 0..ml.size() {
            board.do_move(ml.move_at(m));
            if !is_in_check(board, turn) {
                legal_moves += 1;
            }
            board.undo_move();
        }
        return legal_moves;
    }

    let mut nodes = 0;

    for m in 0..ml.size() {
        let candidate_move = ml.move_at(m);
        board.do_move(candidate_move);

        if !is_in_check(board, turn) {
            nodes += perft(depth - 1, board);
        }

        board.undo_move();
    }

    nodes
}

#[test]
fn blocker_mask_test() {
    let bb = get_blocker_mask(0, PieceType::Rook);
    let mut count = 0;

    for i in 0..64 {
        if ((bb >> i) & 1) == 1 {
            count += 1;
        }
    }

    println!("{}", bb);
    assert_eq!(bb, 0x000101010101017E);
    assert_eq!(1 << count, 4096);
    assert_eq!(count, 12);
}

#[test]
fn perft_startpos() {
    let mut b = load_fen(START_POS).unwrap();

    assert_eq!(perft(1, &mut b), 20);
    assert_eq!(perft(2, &mut b), 400);
    assert_eq!(perft(3, &mut b), 8902);
    assert_eq!(perft(4, &mut b), 197281);
    assert_eq!(perft(5, &mut b), 4865609);
    assert_eq!(perft(6, &mut b), 119060324);
    // assert_eq!(perft(7, &mut b), 3195901860);
    // assert_eq!(perft(8, &mut b), 84998978956);
}

#[test]
fn speed() {
    let mut b = load_fen(START_POS).unwrap();
    // assert_eq!(perft(6, &mut b), 119060324);
    assert_eq!(perft(7, &mut b), 3195901860);
    // assert_eq!(perft(8, &mut b), 84998978956);
}

#[test]
fn perft_kiwipete() {
    let mut k =
        load_fen(" r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - ").unwrap();

    assert_eq!(perft(1, &mut k), 48);
    assert_eq!(perft(2, &mut k), 2039);
    assert_eq!(perft(3, &mut k), 97862);
    assert_eq!(perft(4, &mut k), 4085603);
    assert_eq!(perft(5, &mut k), 193690690);
}
