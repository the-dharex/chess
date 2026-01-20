use crate::board::Board;
use crate::pieces::{Piece, PieceColor, PieceType};
use rand::seq::SliceRandom;
use std::cmp;

// Profundidad de búsqueda
const MAX_DEPTH: i32 = 4;

// Valores básicos de piezas
fn get_piece_value(piece_type: PieceType) -> i32 {
    match piece_type {
        PieceType::Pawn => 10,  // Valor del Peón
        PieceType::Knight => 30,// Valor del Caballo
        PieceType::Bishop => 30,// Valor del Alfil
        PieceType::Rook => 50,  // Valor de la Torre
        PieceType::Queen => 90, // Valor de la Reina
        PieceType::King => 900, // Valor del Rey
    }
}

pub fn get_best_move(board: &Board, color: PieceColor) -> Option<((usize, usize), (usize, usize))> {
    let mut best_move = None;
    let mut best_value = i32::MIN;
    let mut alpha = i32::MIN;
    let beta = i32::MAX;

    // Obtener todos los movimientos posibles para todas las piezas de este color
    let mut all_moves = Vec::new();
    for y in 0..8 {
        for x in 0..8 {
            if let Some(piece) = board.grid[y][x] {
                if piece.color == color {
                    let moves = board.get_valid_moves((x, y));
                    for dest in moves {
                        all_moves.push(((x, y), dest));
                    }
                }
            }
        }
    }

    // Mezclar movimientos para añadir variedad si los puntajes son iguales y mejorar poda
    let mut rng = rand::thread_rng();
    all_moves.shuffle(&mut rng);

    for (from, to) in all_moves {
        let mut new_board = board.clone();
        new_board.move_piece(from, to);
        
        let value = minimax(&new_board, MAX_DEPTH - 1, alpha, beta, false, color);
        
        if value > best_value {
            best_value = value;
            best_move = Some((from, to));
        }
        alpha = cmp::max(alpha, best_value);
    }

    best_move
}

fn minimax(board: &Board, depth: i32, mut alpha: i32, mut beta: i32, is_maximizing: bool, my_color: PieceColor) -> i32 {
    if depth == 0 {
        return evaluate(board, my_color);
    }

    // Verificar fin del juego
    let current_turn_color = if is_maximizing { my_color } else { my_color.opposite() };
    
    let mut all_moves = Vec::new();
    for y in 0..8 {
        for x in 0..8 {
            if let Some(piece) = board.grid[y][x] {
                if piece.color == current_turn_color {
                    let moves = board.get_valid_moves((x, y));
                    for dest in moves {
                        all_moves.push(((x, y), dest));
                    }
                }
            }
        }
    }

    if all_moves.is_empty() {
        // No hay movimientos. Jaque Mate o Ahogado.
        if board.is_in_check(current_turn_color) {
            return if is_maximizing { -99999 } else { 99999 }; // Jaque Mate
        } else {
            return 0; // Ahogado
        }
    }

    if is_maximizing {
        let mut max_eval = i32::MIN;
        for (from, to) in all_moves {
            let mut new_board = board.clone();
            new_board.move_piece(from, to);
            let eval = minimax(&new_board, depth - 1, alpha, beta, false, my_color);
            max_eval = cmp::max(max_eval, eval);
            alpha = cmp::max(alpha, eval);
            if beta <= alpha {
                break;
            }
        }
        max_eval
    } else {
        let mut min_eval = i32::MAX;
        for (from, to) in all_moves {
            let mut new_board = board.clone();
            new_board.move_piece(from, to);
            let eval = minimax(&new_board, depth - 1, alpha, beta, true, my_color);
            min_eval = cmp::min(min_eval, eval);
            beta = cmp::min(beta, eval);
            if beta <= alpha {
                break;
            }
        }
        min_eval
    }
}

fn evaluate(board: &Board, my_color: PieceColor) -> i32 {
    let mut score = 0;

    for y in 0..8 {
        for x in 0..8 {
            if let Some(piece) = board.grid[y][x] {
                let value = get_piece_value(piece.piece_type);               
                if piece.color == my_color {
                    score += value;
                    score += get_position_bonus(piece, x, y);
                } else {
                    score -= value;
                    score -= get_position_bonus(piece, x, y);
                }
            }
        }
    }
    score
}

// Bonificaciones posicionales simples
fn get_position_bonus(piece: Piece, x: usize, y: usize) -> i32 {
    // Invertir y para las negras para simplificar
    let center_bonus = if (3..=4).contains(&x) && (3..=4).contains(&y) {
        20
    } else if (2..=5).contains(&x) && (2..=5).contains(&y) {
        10
    } else {
        0
    };

    let mut bonus = center_bonus;

    match piece.piece_type {
        PieceType::Pawn => {
            // Avanzar peones es generalmente bueno
             let rank = if piece.color == PieceColor::White { 7 - y } else { y };
             bonus += rank as i32 * 10;
        }
        PieceType::Knight => {
             // Los caballos odian los bordes
             if x == 0 || x == 7 || y == 0 || y == 7 {
                 bonus -= 30;
             }
        }
        _ => {}
    }

    bonus
}
