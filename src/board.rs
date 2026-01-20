use crate::pieces::{Piece, PieceColor, PieceType};


pub const BOARD_SIZE: usize = 8;

#[derive(Clone, Debug)]
pub struct Board {
    pub grid: [[Option<Piece>; BOARD_SIZE]; BOARD_SIZE],
    pub last_move: Option<((usize, usize), (usize, usize))>,
}

impl Board {
    pub fn new() -> Self {
        let mut grid = [[None; BOARD_SIZE]; BOARD_SIZE];
        
        // Configurar Peones
        for x in 0..BOARD_SIZE {
            grid[1][x] = Some(Piece::new(PieceType::Pawn, PieceColor::Black));
            grid[6][x] = Some(Piece::new(PieceType::Pawn, PieceColor::White));
        }

        // Configurar otras piezas
        let layout = [
            PieceType::Rook, PieceType::Knight, PieceType::Bishop, PieceType::Queen,
            PieceType::King, PieceType::Bishop, PieceType::Knight, PieceType::Rook
        ];

        for (x, &piece_type) in layout.iter().enumerate() {
            grid[0][x] = Some(Piece::new(piece_type, PieceColor::Black));
            grid[7][x] = Some(Piece::new(piece_type, PieceColor::White));
        }

        Self { grid, last_move: None }
    }

    pub fn move_piece(&mut self, from: (usize, usize), to: (usize, usize)) {
        if let Some(mut piece) = self.grid[from.1][from.0].take() {
            let dx = (to.0 as i32 - from.0 as i32).abs();
            let _dy = to.1 as i32 - from.1 as i32;

            // Manejar ejecución de Enroque
            if piece.piece_type == PieceType::King && dx == 2 { // dx is already abs()
                let rook_x = if to.0 > from.0 { 7 } else { 0 };
                let new_rook_x = if to.0 > from.0 { 5 } else { 3 };
                let rook_y = from.1;
                
                if let Some(mut rook) = self.grid[rook_y][rook_x].take() {
                     rook.has_moved = true;
                     self.grid[rook_y][new_rook_x] = Some(rook);
                }
            }

            // Manejar ejecución de En Passant
            if piece.piece_type == PieceType::Pawn && dx.abs() == 1 && self.grid[to.1][to.0].is_none() {
                // Si se mueve en diagonal a una casilla vacía, es en passant
                // Eliminar el peón capturado (que está en [to.0, from.1])
                self.grid[from.1][to.0] = None;
            }

            piece.has_moved = true;
            
            // Promoción (Básica: siempre Reina)
            if piece.piece_type == PieceType::Pawn {
                if (piece.color == PieceColor::White && to.1 == 0) ||
                   (piece.color == PieceColor::Black && to.1 == 7) {
                    piece.piece_type = PieceType::Queen;
                }
            }

            self.grid[to.1][to.0] = Some(piece);
            self.last_move = Some((from, to));
        }
    }

    pub fn get_valid_moves(&self, pos: (usize, usize)) -> Vec<(usize, usize)> {
        let mut moves = Vec::new();
        if let Some(piece) = self.grid[pos.1][pos.0] {
            let candidate_moves = self.get_pseudo_legal_moves(pos, &piece);
            
            for &dest in &candidate_moves {
                // Determinar si el movimiento pone/deja al propio Rey en jaque
                let mut temp_board = self.clone();
                temp_board.move_piece(pos, dest);
                if !temp_board.is_in_check(piece.color) {
                    moves.push(dest);
                }
            }
        }
        moves
    }

    fn get_pseudo_legal_moves(&self, pos: (usize, usize), piece: &Piece) -> Vec<(usize, usize)> {
        let mut moves = Vec::new();
        let (x, y) = (pos.0 as i32, pos.1 as i32);

        match piece.piece_type {
            PieceType::Pawn => {
                let direction = if piece.color == PieceColor::White { -1 } else { 1 };
                
                // Movimiento hacia adelante
                let new_y = y + direction;
                if self.is_valid_pos(x, new_y) && self.is_empty(x, new_y) {
                    moves.push((x as usize, new_y as usize));
                    
                    // Movimiento doble
                    if !piece.has_moved {
                        let double_y = y + 2 * direction;
                        if self.is_valid_pos(x, double_y) && self.is_empty(x, double_y) {
                            moves.push((x as usize, double_y as usize));
                        }
                    }
                }

                // Capturas
                for &dx in &[-1, 1] {
                    let caps_x = x + dx;
                    let caps_y = y + direction;
                     
                    // Captura normal
                    if self.is_valid_pos(caps_x, caps_y) && self.is_enemy(caps_x, caps_y, piece.color) {
                         moves.push((caps_x as usize, caps_y as usize));
                    }

                    // En Passant
                     if self.is_valid_pos(caps_x, caps_y) && self.is_empty(caps_x, caps_y) {
                         if let Some(((_last_from_x, last_from_y), (last_to_x, last_to_y))) = self.last_move {
                             // Verificar si el último movimiento fue un avance doble de peón adyacente
                             if last_to_x == caps_x as usize && last_to_y == y as usize &&
                                (last_from_y as i32 - last_to_y as i32).abs() == 2 {
                                     // Verificar si la pieza allí es un peón (debería serlo)
                                     if let Some(target) = self.grid[last_to_y][last_to_x] {
                                         if target.piece_type == PieceType::Pawn && target.color != piece.color {
                                             moves.push((caps_x as usize, caps_y as usize));
                                         }
                                     }
                                }
                         }
                     }
                }
            }
            PieceType::Rook => self.sliding_moves(&mut moves, x, y, &[(0, 1), (0, -1), (1, 0), (-1, 0)], piece.color),
            PieceType::Bishop => self.sliding_moves(&mut moves, x, y, &[(1, 1), (1, -1), (-1, 1), (-1, -1)], piece.color),
            PieceType::Queen => {
                self.sliding_moves(&mut moves, x, y, &[(0, 1), (0, -1), (1, 0), (-1, 0)], piece.color);
                self.sliding_moves(&mut moves, x, y, &[(1, 1), (1, -1), (-1, 1), (-1, -1)], piece.color);
            }
            PieceType::Knight => {
                let offsets = [
                    (1, 2), (1, -2), (-1, 2), (-1, -2),
                    (2, 1), (2, -1), (-2, 1), (-2, -1)
                ];
                for &(dx, dy) in &offsets {
                    if self.is_valid_pos(x + dx, y + dy) {
                        if self.is_empty(x + dx, y + dy) || self.is_enemy(x + dx, y + dy, piece.color) {
                            moves.push(((x + dx) as usize, (y + dy) as usize));
                        }
                    }
                }
            }
            PieceType::King => {
                let offsets = [
                    (0, 1), (0, -1), (1, 0), (-1, 0),
                    (1, 1), (1, -1), (-1, 1), (-1, -1)
                ];
                for &(dx, dy) in &offsets {
                    if self.is_valid_pos(x + dx, y + dy) {
                        if self.is_empty(x + dx, y + dy) || self.is_enemy(x + dx, y + dy, piece.color) {
                            moves.push(((x + dx) as usize, (y + dy) as usize));
                        }
                    }
                }

                // Enroque
                if !piece.has_moved {
                    // Lado del Rey
                    if self.can_castle(pos, true) {
                        moves.push(((x + 2) as usize, y as usize));
                    }
                    // Lado de la Reina
                    if self.can_castle(pos, false) {
                        moves.push(((x - 2) as usize, y as usize));
                    }
                }
            }
        }

        moves
    }

    fn sliding_moves(&self, moves: &mut Vec<(usize, usize)>, x: i32, y: i32, dirs: &[(i32, i32)], color: PieceColor) {
        for &(dx, dy) in dirs {
            let mut nx = x + dx;
            let mut ny = y + dy;
            while self.is_valid_pos(nx, ny) {
                if self.is_empty(nx, ny) {
                    moves.push((nx as usize, ny as usize));
                } else {
                    if self.is_enemy(nx, ny, color) {
                        moves.push((nx as usize, ny as usize));
                    }
                    break;
                }
                nx += dx;
                ny += dy;
            }
        }
    }

    fn is_valid_pos(&self, x: i32, y: i32) -> bool {
        x >= 0 && x < 8 && y >= 0 && y < 8
    }

    fn is_empty(&self, x: i32, y: i32) -> bool {
        self.grid[y as usize][x as usize].is_none()
    }

    fn is_enemy(&self, x: i32, y: i32, color: PieceColor) -> bool {
        if let Some(piece) = self.grid[y as usize][x as usize] {
            piece.color != color
        } else {
            false
        }
    }

    pub fn is_in_check(&self, color: PieceColor) -> bool {
        let king_pos = self.find_king(color);
        if let Some((kx, ky)) = king_pos {
            self.is_square_attacked((kx, ky), color)
        } else {
            false
        }
    }

    // Ayudante para verificar si una casilla es atacada por el enemigo
    fn is_square_attacked(&self, pos: (usize, usize), color: PieceColor) -> bool {
         for y in 0..BOARD_SIZE {
            for x in 0..BOARD_SIZE {
                if let Some(piece) = self.grid[y][x] {
                    if piece.color != color {
                         let moves = self.get_basic_attacks((x, y), &piece);
                         if moves.contains(&pos) {
                             return true;
                         }
                    }
                }
            }
        }
        false
    }
    
    // Generación de movimientos simplificada que no verifica jaque/enroque/cosas especiales, solo ataques puros.
    fn get_basic_attacks(&self, pos: (usize, usize), piece: &Piece) -> Vec<(usize, usize)> {
        let mut moves = Vec::new();
        let (x, y) = (pos.0 as i32, pos.1 as i32);

        match piece.piece_type {
            PieceType::Pawn => {
                let direction = if piece.color == PieceColor::White { -1 } else { 1 };
                // Los peones solo atacan diagonalmente
                for &dx in &[-1, 1] {
                    let caps_x = x + dx;
                    let caps_y = y + direction;
                     if self.is_valid_pos(caps_x, caps_y) {
                         moves.push((caps_x as usize, caps_y as usize));
                     }
                }
            }
            PieceType::King => {
                 let offsets = [
                    (0, 1), (0, -1), (1, 0), (-1, 0),
                    (1, 1), (1, -1), (-1, 1), (-1, -1)
                ];
                for &(dx, dy) in &offsets {
                    if self.is_valid_pos(x + dx, y + dy) {
                        moves.push(((x + dx) as usize, (y + dy) as usize));
                    }
                }
            }
            // Otros son iguales a lo normal
            PieceType::Rook => self.sliding_moves(&mut moves, x, y, &[(0, 1), (0, -1), (1, 0), (-1, 0)], piece.color),
            PieceType::Bishop => self.sliding_moves(&mut moves, x, y, &[(1, 1), (1, -1), (-1, 1), (-1, -1)], piece.color),
            PieceType::Queen => {
                self.sliding_moves(&mut moves, x, y, &[(0, 1), (0, -1), (1, 0), (-1, 0)], piece.color);
                self.sliding_moves(&mut moves, x, y, &[(1, 1), (1, -1), (-1, 1), (-1, -1)], piece.color);
            }
            PieceType::Knight => {
                let offsets = [
                    (1, 2), (1, -2), (-1, 2), (-1, -2),
                    (2, 1), (2, -1), (-2, 1), (-2, -1)
                ];
                for &(dx, dy) in &offsets {
                    if self.is_valid_pos(x + dx, y + dy) {
                        moves.push(((x + dx) as usize, (y + dy) as usize));
                    }
                }
            }
        }
        moves
    }

    fn can_castle(&self, king_pos: (usize, usize), kingside: bool) -> bool {
        if self.is_in_check(self.grid[king_pos.1][king_pos.0].unwrap().color) {
            return false;
        }

        let y = king_pos.1;
        let _x = king_pos.0;
        let color = self.grid[king_pos.1][king_pos.0].unwrap().color;

        let (rook_x, empty_x_range) = if kingside {
            (7, 5..=6)
        } else {
            (0, 1..=3)
        };

        // Verificar si la Torre está allí y no se ha movido
        if let Some(piece) = self.grid[y][rook_x] {
            if piece.piece_type != PieceType::Rook || piece.has_moved || piece.color != color {
                return false;
            }
        } else {
            return false;
        }

        // Verificar que el camino esté despejado
        for check_x in empty_x_range {
            if !self.grid[y][check_x].is_none() {
                return false;
            }
        }
        
        // Verificar si el Rey pasa por jaque (solo para las dos casillas que mueve)
        let check_squares = if kingside {
            vec![(5, y), (6, y)]
        } else {
            vec![(3, y), (2, y)]
        };
        
        for pos in check_squares {
            if self.is_square_attacked(pos, color) {
                return false;
            }
        }

        true
    }

    
    pub fn is_checkmate(&self, color: PieceColor) -> bool {
        if !self.is_in_check(color) {
            return false;
        }
        // Probar todos los movimientos para todas las piezas
        for y in 0..BOARD_SIZE {
            for x in 0..BOARD_SIZE {
                if let Some(piece) = self.grid[y][x] {
                    if piece.color == color {
                         if !self.get_valid_moves((x, y)).is_empty() {
                             return false;
                         }
                    }
                }
            }
        }
        true
    }

    fn find_king(&self, color: PieceColor) -> Option<(usize, usize)> {
        for y in 0..BOARD_SIZE {
            for x in 0..BOARD_SIZE {
                if let Some(piece) = self.grid[y][x] {
                    if piece.piece_type == PieceType::King && piece.color == color {
                        return Some((x, y));
                    }
                }
            }
        }
        None
    }
}
