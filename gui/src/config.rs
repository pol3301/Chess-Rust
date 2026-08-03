use egui::Color32;

#[derive(Clone, Copy)]
pub struct ColorSet(pub Color32, pub Color32); //(Light, Dark)

impl ColorSet {
    const LICHESS_COLORS: Self = Self(
        Color32::from_rgb(0xF0, 0xD9, 0xB5),
        Color32::from_rgb(0xB5, 0x88, 0x63),
    );

    const BLACK_WHITE: Self = Self(
        Color32::from_rgb(0x00, 0x00, 0x00),
        Color32::from_rgb(0xFF, 0xFF, 0xFF),
    );

    const CHESSDOTCOM_COLORS: Self = Self(
        Color32::from_rgb(0xEA, 0xED, 0xD1),
        Color32::from_rgb(0x77, 0x95, 0x57),
    );
}

#[derive(Clone, Copy)]
pub enum PieceSet {
    MaxArt,
    MaxArtV2,
    ChessDotCom,
    Lichess,
}

pub struct AppConfig {
    pub piece_set: PieceSet,
    pub color_set: ColorSet,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            piece_set: PieceSet::Lichess,
            color_set: ColorSet::LICHESS_COLORS,
        }
    }
}
