use std::{env, fs::File, io::Write, path::Path};

#[rustfmt::skip]
const ROOK_MAGICS: [u64; 64] = [ 4935945329314512900, 1170936040690548800, 2954396541018767624, 936784113040410660, 720611640449457200, 9367496072596852737, 864699941756338697, 36028938753036928, 90212732187443360, 844699877277954, 2533347949543489, 3769654175621711872, 608127237073404928, 162411074455864320, 4785285057609732, 1191624317134177536, 1765552341242626048, 10380867784713179138, 282575564177424, 290511863047848032, 53876204044556, 8800925126720, 576465150487465985, 36039792173880068, 9376564795077066784, 9183126483914752, 326863920513941520, 153131185572151424, 11540495521056768, 9228440003776088080, 10016287119264317540, 576742596647747844, 36030171421089794, 281749871403648, 1443544555507945472, 292736278166774016, 5766437127571130368, 18177128386265600, 112598791139365392, 83457331676841216, 4611756389598265348, 4620693356463308800, 4613480425699999808, 580546436595720, 9226842095686287488, 43347322776191012, 17696456704010, 145245487530901505, 222435744227197184, 649714808358306304, 2310504939052368768, 4972256603371733248, 2308095378254266624, 562995251937408, 576463260850594816, 176489889008128, 5802924477880729859, 594625036585992226, 18023332042522885, 1153203014549377025, 9314007014520922370, 759138047119065217, 72339071230410757, 290486595541557378, ];
#[rustfmt::skip]
const BISHOP_MAGICS: [u64; 64] = [45071610159833120, 577604253527377920, 1158552112276310211, 1200668846067712, 298385603536486432, 1172347685743657508, 4611905930702880768, 72200535113155584, 667113304195597312, 18049875543343168, 2738224861687087109, 69845394250530848, 4415764856841, 577591256549965824, 288241923240501248, 10142447225085987, 1135529827763233, 18718154740007968, 7516648808768602176, 4910049502365876228, 1230609173723545744, 14160584999853688832, 6830312295236608, 562960163805224, 37172291328587904, 9225799759133149824, 9847419669605255200, 9302189986860367876, 73466070581657600, 2598578084546379920, 2308097439712741384, 594758824816182272, 1335345052452921890, 1585633274373120, 10699493712599044, 4616262187974393984, 38282813035819024, 141841295609860, 15277063195720713216, 10982595632048906753, 1450445021827318272, 1200735553389698, 4629719143128498432, 36171742189715968, 144185625842553088, 83606933039227136, 4649975416788877380, 9243647105388515456, 2306150907400159232, 291409254940672, 108931918737702976, 324613244035792960, 324259448132796483, 360340755606340096, 2319358274934521856, 2260600403009536, 74769005355008, 19796596101122, 81681623888170000, 37225074277242890, 596766812488606212, 90641543988512, 1243582972852568580, 18015807294423104];

type Bitboard = u64;
enum PieceType {
    Bishop = 2,
    Rook = 3,
}

#[derive(Clone, Copy)]
struct MagicInfo {
    magic: u64,
    offset: u64,
    shift: i32,
}

impl MagicInfo {
    pub const fn new() -> Self {
        MagicInfo {
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

#[inline(always)]
const fn calc_shift(index: u8, piece_type: PieceType) -> i32 {
    (64 - get_blocker_mask(index, piece_type).count_ones()) as i32
}

const ROOK_MAGIC_INFO: [MagicInfo; 64] = {
    let mut i = 0;
    let mut array = [MagicInfo::new(); 64];

    while i < 64 {
        array[i] = MagicInfo {
            magic: ROOK_MAGICS[i],
            offset: ROOK_OFFSETS[i] as u64,
            shift: calc_shift(i as u8, PieceType::Rook),
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

const fn generate_permutations(mask: Bitboard) -> ([Bitboard; 4096], usize) {
    let (bits_array, bits_array_size) = {
        let mut c = 0;
        let mut array = [0; 12];

        let mut m = mask;
        while m != 0 {
            array[c] = m.trailing_zeros();
            c += 1;

            m &= m - 1;
        }

        (array, c)
    };

    let mut permutations: [Bitboard; 4096] = [0; 4096];
    let mut count = 0;

    while count < (1 << bits_array_size) {
        let mut c = 0;
        let mut bb: Bitboard = 0;
        while c < bits_array_size {
            let bit = ((count >> c) & 1) as u64;
            bb |= bit << bits_array[c];

            c += 1;
        }

        permutations[count] = bb;
        count += 1;
    }

    (permutations, count)
}

const fn generate_sliding_attacks(index: u8, blocker: Bitboard, piece_type: PieceType) -> Bitboard {
    let start_x = (index % 8) as i32;
    let start_y = (index / 8) as i32;

    let mut attacks: Bitboard = 0;

    let (dx_arr, dy_arr) = match piece_type {
        PieceType::Rook => ([0, 0, -1, 1], [-1, 1, 0, 0]),
        PieceType::Bishop => ([1, -1, 1, -1], [1, -1, -1, 1]),
    };

    let mut dir_idx = 0;
    while dir_idx < dx_arr.len() {
        let dx = dx_arr[dir_idx];
        let dy = dy_arr[dir_idx];

        let mut x = start_x + dx;
        let mut y = start_y + dy;

        while x >= 0 && x < 8 && y >= 0 && y < 8 {
            let curr_idx = (x + (y * 8)) as u8;
            attacks |= 1u64 << curr_idx;

            if (1u64 << curr_idx) & blocker != 0 {
                break;
            }

            x += dx;
            y += dy;
        }

        dir_idx += 1;
    }

    attacks
}

const fn generate_all_blockers(index: u8, piece_type: PieceType) -> ([Bitboard; 4096], usize) {
    let mask = get_blocker_mask(index, piece_type);
    generate_permutations(mask)
}

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

const BISHOP_MAGIC_INFO: [MagicInfo; 64] = {
    let mut i = 0;
    let mut array = [MagicInfo::new(); 64];

    while i < 64 {
        array[i] = MagicInfo {
            magic: BISHOP_MAGICS[i],
            offset: BISHOP_OFFSETS[i] as u64,
            shift: calc_shift(i as u8, PieceType::Bishop),
        };

        i += 1;
    }

    array
};

fn gen_bishops_attack_table(out_dir: String) {
    let dest_path = Path::new(&out_dir).join("rooks_attack_table.rs");
    let mut f = File::create(&dest_path).unwrap();
    let mut array = [0; 5248];

    for i in 0..64 {
        let (blockers, blockers_size) = generate_all_blockers(i, PieceType::Bishop);

        for &blocker in &blockers[..blockers_size] {
            let magic_info = &BISHOP_MAGIC_INFO[i as usize];
            let array_index = ((blocker.wrapping_mul(magic_info.magic) >> magic_info.shift)
                + magic_info.offset) as usize;

            array[array_index] = generate_sliding_attacks(i, blocker, PieceType::Bishop);
        }
    }

    let mut bishop_attacks_str = String::from("static BISHOP_ATTACKS: [Bitboard; 5248] = [");

    for bb in array {
        bishop_attacks_str.push_str(&format!("{},", bb));
    }

    bishop_attacks_str.push_str("];\n");

    f.write_all(bishop_attacks_str.as_bytes()).unwrap();
}

fn gen_rooks_attack_table(out_dir: String) {
    let dest_path = Path::new(&out_dir).join("bishops_attack_table.rs");
    let mut f = File::create(&dest_path).unwrap();

    let mut array = [0; 102400];

    for i in 0..64 {
        let (blockers, blockers_size) = generate_all_blockers(i, PieceType::Rook);

        for &blocker in &blockers[..blockers_size] {
            let magic_info = &ROOK_MAGIC_INFO[i as usize];
            let array_index = ((blocker.wrapping_mul(magic_info.magic) >> magic_info.shift)
                + magic_info.offset) as usize;

            array[array_index] = generate_sliding_attacks(i, blocker, PieceType::Rook);
        }
    }

    let mut rook_attacks_str = String::from("static ROOK_ATTACKS: [Bitboard; 102400] = [");

    for bb in array {
        rook_attacks_str.push_str(&format!("{},", bb));
    }

    rook_attacks_str.push_str("];\n");

    f.write_all(rook_attacks_str.as_bytes()).unwrap();
}

pub fn main() {
    let out_dir = env::var("OUT_DIR").unwrap();
    println!("cargo:rerun-if-changed=build.rs");

    gen_rooks_attack_table(out_dir.to_owned());
    gen_bishops_attack_table(out_dir.to_owned());
}
