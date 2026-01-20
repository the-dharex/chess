use ggez::graphics::Image;
use ggez::audio::Source;
use ggez::{Context, GameResult};
use std::collections::HashMap;
use crate::pieces::{PieceType, PieceColor};

pub struct Resources {
    pub pieces: HashMap<(PieceType, PieceColor), Image>,
    pub move_sound: Source,
}

impl Resources {
    pub fn new(ctx: &mut Context) -> GameResult<Self> {
        let mut pieces = HashMap::new();

        let types = [
            PieceType::Pawn,
            PieceType::Rook,
            PieceType::Knight,
            PieceType::Bishop,
            PieceType::Queen,
            PieceType::King,
        ];
        
        let colors = [PieceColor::White, PieceColor::Black];

        for &t in &types {
            for &c in &colors {
                let filename = Self::get_filename(t, c);
                let path = format!("/pieces/{}", filename);
                let image = Image::from_path(ctx, &path)?;
                pieces.insert((t, c), image);
            }
        }

        // Cargar sonido
        let move_sound = Source::new(ctx, "/sounds/wood1.wav")?;

        Ok(Self {
            pieces,
            move_sound,
        })
    }

    fn get_filename(piece_type: PieceType, color: PieceColor) -> &'static str {
        match (piece_type, color) {
            (PieceType::Pawn, PieceColor::White) => "pawn_white.png",
            (PieceType::Pawn, PieceColor::Black) => "pawn_black.png",
            (PieceType::Rook, PieceColor::White) => "rook_white.png",
            (PieceType::Rook, PieceColor::Black) => "rook_black.png",
            (PieceType::Knight, PieceColor::White) => "knight_white.png",
            (PieceType::Knight, PieceColor::Black) => "knight_black.png",
            (PieceType::Bishop, PieceColor::White) => "bishop_white.png",
            (PieceType::Bishop, PieceColor::Black) => "bishop_black.png",
            (PieceType::Queen, PieceColor::White) => "queen_white.png",
            (PieceType::Queen, PieceColor::Black) => "queen_black.png",
            (PieceType::King, PieceColor::White) => "king_white.png",
            (PieceType::King, PieceColor::Black) => "king_black.png",
        }
    }
}
