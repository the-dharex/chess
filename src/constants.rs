use ggez::graphics::Color;

pub const SCREEN_SIZE: (f32, f32) = (800.0, 800.0);
pub const GRID_SIZE: i32 = 8;
pub const CELL_SIZE: f32 = SCREEN_SIZE.0 / GRID_SIZE as f32;

pub const WHITE_COLOR: Color = Color::new(0.9, 0.9, 0.9, 1.0); // Casilla clara
pub const BLACK_COLOR: Color = Color::new(0.4, 0.4, 0.4, 1.0); // Casilla oscura
pub const HIGHLIGHT_COLOR: Color = Color::new(0.8, 0.8, 0.2, 0.5); // Resaltado de selección
pub const VALID_MOVE_COLOR: Color = Color::new(0.2, 0.8, 0.2, 0.5); // Resaltado de movimiento válido
