use std::{env, fs, path::Path};

fn main() {
    let out_dir = env::var_os("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join("generated_piece_match.rs");

    let piece_types = ["Pawn", "Knight", "Bishop", "Rook", "Queen", "King"];
    let piece_colors = ["White", "Black"];
    let piece_sets = ["Lichess", "ChessDotCom", "MaxArt", "MaxArtV2"];

    let mut code = String::new();
    code.push_str("match (piece.get_type(), piece.get_color(), image_set) {\n");

    for set in piece_sets {
        let folder = match set {
            "Lichess" => "res-lichess",
            "MaxArt" => "res-max",
            "ChessDotCom" => "res-chessdotcom",
            "MaxArtV2" => "res-max-v2",
            _ => unreachable!(),
        };

        for color in piece_colors {
            let color_char = if color == "White" { 'w' } else { 'b' };

            for pt in piece_types {
                let pt_char = match pt {
                    "Pawn" => 'p',
                    "Knight" => 'n',
                    "Bishop" => 'b',
                    "Rook" => 'r',
                    "Queen" => 'q',
                    "King" => 'k',
                    _ => unreachable!(),
                };

                code.push_str(&format!("    (PieceType::{}, PieceColor::{}, PieceSet::{}) => include_image!(concat!(env!(\"CARGO_MANIFEST_DIR\"), \"/{}/{}{}.png\")),\n", pt, color, set, folder, color_char, pt_char));
            }
        }
    }
    code.push_str("    _ => unreachable!()\n}");

    fs::write(dest_path, code).unwrap();

    println!("cargo:rerun-if-changed=build.rs");
}
