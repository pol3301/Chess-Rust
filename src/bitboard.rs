pub type Bitboard = u64;

pub const A_FILE: Bitboard = 0x0101010101010101;
#[allow(dead_code)]
pub const B_FILE: Bitboard = A_FILE << 1;
#[allow(dead_code)]
pub const C_FILE: Bitboard = A_FILE << 2;
#[allow(dead_code)]
pub const D_FILE: Bitboard = A_FILE << 3;
#[allow(dead_code)]
pub const E_FILE: Bitboard = A_FILE << 4;
#[allow(dead_code)]
pub const F_FILE: Bitboard = A_FILE << 5;
#[allow(dead_code)]
pub const G_FILE: Bitboard = A_FILE << 6;
pub const H_FILE: Bitboard = A_FILE << 7;

pub const RANK_1: Bitboard = 0xFF;
pub const RANK_2: Bitboard = RANK_1 << 8;
#[allow(dead_code)]
pub const RANK_3: Bitboard = RANK_1 << (8 * 2);
#[allow(dead_code)]
pub const RANK_4: Bitboard = RANK_1 << (8 * 3);
#[allow(dead_code)]
pub const RANK_5: Bitboard = RANK_1 << (8 * 4);
pub const RANK_6: Bitboard = RANK_1 << (8 * 5);
#[allow(dead_code)]
pub const RANK_7: Bitboard = RANK_1 << (8 * 6);
#[allow(dead_code)]
pub const RANK_8: Bitboard = RANK_1 << (8 * 7);

pub trait BitboardExt {
    fn pop_lsb(&mut self) -> u8;
}

impl BitboardExt for Bitboard {
    #[inline(always)]
    fn pop_lsb(&mut self) -> u8 {
        let bits = self.trailing_zeros() as u8;
        *self &= *self - 1u64;
        bits
    }
}
