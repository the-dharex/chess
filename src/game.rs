use ggez::{Context, GameResult, graphics};
use ggez::audio::SoundSource;
use ggez::event::{self, MouseButton};
use ggez::graphics::{Mesh, DrawParam, Color, DrawMode, Rect, Text};
use ggez::input::keyboard::KeyCode;
use rand::Rng;
use std::net::{TcpListener, TcpStream};
use std::thread;
use std::sync::mpsc;
use local_ip_address::local_ip;

use crate::constants::*;
use crate::resources::Resources;
use crate::pieces::{PieceColor};
use crate::board::{Board, BOARD_SIZE};
use crate::ai;
use crate::network::{NetworkClient, NetworkMessage};

#[derive(PartialEq, Clone, Copy)]
enum AppMode {
    Menu,
    HostWait,
    JoinInput,
    Playing,
}

#[derive(PartialEq, Clone, Copy)]
enum GameType {
    LocalAI,
    Multiplayer,
}

pub struct GameState {
    resources: Resources,
    board: Board,
    turn: PieceColor,
    selected_square: Option<(usize, usize)>,
    valid_moves_for_selected: Vec<(usize, usize)>,
    player_color: PieceColor,
    game_over: bool,
    winner: Option<PieceColor>,
    
    // Menú y Red
    mode: AppMode,
    game_type: GameType,
    network_client: Option<NetworkClient>,
    host_listener: Option<mpsc::Receiver<TcpStream>>, // Canal para recibir stream aceptado
    host_ip: String,
    join_ip_input: String,
}

impl GameState {
    pub fn new(ctx: &mut Context) -> GameResult<Self> {
        let resources = Resources::new(ctx)?;
        let board = Board::new();
        
        Ok(Self {
            resources,
            board,
            turn: PieceColor::White,
            selected_square: None,
            valid_moves_for_selected: Vec::new(),
            player_color: PieceColor::White,
            game_over: false,
            winner: None,
            mode: AppMode::Menu,
            game_type: GameType::LocalAI,
            network_client: None,
            host_listener: None,
            host_ip: String::new(),
            join_ip_input: String::new(),
        })
    }

    fn reset_game(&mut self, player_color: PieceColor, game_type: GameType) {
        self.board = Board::new();
        self.turn = PieceColor::White;
        self.selected_square = None;
        self.valid_moves_for_selected.clear();
        self.player_color = player_color;
        self.game_over = false;
        self.winner = None;
        self.game_type = game_type;
        self.mode = AppMode::Playing;
    }

    fn get_view_coords(&self, x: usize, y: usize) -> (usize, usize) {
        if self.player_color == PieceColor::White {
            (x, y)
        } else {
            (7 - x, 7 - y)
        }
    }

    fn start_host(&mut self) {
        if let Ok(ip) = local_ip() {
            self.host_ip = format!("{}:8080", ip);
            println!("Hospedando en {}", self.host_ip);

            // Copiar al portapapeles
            if let Ok(mut clipboard) = arboard::Clipboard::new() {
                let _ = clipboard.set_text(&self.host_ip);
            }
            
            let address = self.host_ip.clone();
            let (tx, rx) = mpsc::channel();
            
            thread::spawn(move || {
                if let Ok(listener) = TcpListener::bind(address) {
                    if let Ok((stream, _)) = listener.accept() {
                        let _ = tx.send(stream);
                    }
                }
            });
            self.host_listener = Some(rx);
            self.mode = AppMode::HostWait;
        } else {
            println!("Error al obtener IP local");
        }
    }

    fn connect_to_host(&mut self) {
        // Intentar conectar
        if let Ok(stream) = TcpStream::connect(&self.join_ip_input) {
            println!("Conectado a {}", self.join_ip_input);
            let client = NetworkClient::new(stream);
            self.network_client = Some(client);
            self.mode = AppMode::Playing;
            self.game_type = GameType::Multiplayer;
        } else {
            println!("Error al conectar a {}", self.join_ip_input);
        }
    }
}

impl event::EventHandler<ggez::GameError> for GameState {
    // Implementar Copy para AppMode y GameType
    fn update(&mut self, ctx: &mut Context) -> GameResult {
        let mode = self.mode; // Acceder a copia
        match mode {
            AppMode::Menu => {},
            AppMode::HostWait => {
                // Verificar si el cliente se conectó
                // necesario limitar el alcance del préstamo
                let mut new_stream: Option<TcpStream> = None;
                if let Some(rx) = &self.host_listener {
                    if let Ok(stream) = rx.try_recv() {
                        new_stream = Some(stream);
                    }
                }

                if let Some(stream) = new_stream {
                    println!("¡Cliente conectado!");
                    let mut client = NetworkClient::new(stream);
                    
                    // Asignar colores
                    let mut rng = rand::thread_rng();
                    let my_color = if rng.gen_bool(0.5) { PieceColor::White } else { PieceColor::Black };
                    let client_color = my_color.opposite();
                    
                    // Enviar saludo (Handshake)
                    client.send(NetworkMessage::Handshake { color: client_color });
                    
                    self.network_client = Some(client);
                    self.reset_game(my_color, GameType::Multiplayer);
                    self.host_listener = None;
                }
            },
            AppMode::JoinInput => {},
            AppMode::Playing => {
                // Manejar mensajes de red
                let mut messages = Vec::new();
                if let Some(client) = &self.network_client {
                    while let Some(msg) = client.try_recv() {
                        messages.push(msg);
                    }
                }

                for msg in messages {
                    match msg {
                        NetworkMessage::Handshake { color } => {
                            println!("Mensaje recibido: Eres {:?}", color);
                            self.reset_game(color, GameType::Multiplayer);
                        },
                        NetworkMessage::Move { from, to } => {
                            println!("Movimiento recibido: {:?} -> {:?}", from, to);
                            self.board.move_piece(from, to);
                            // Reproducir sonido
                            let _ = self.resources.move_sound.play(ctx);
                            self.turn = self.turn.opposite();
                            // Verificar fin del juego
                            if self.board.is_checkmate(self.turn) {
                                self.game_over = true;
                                self.winner = Some(self.turn.opposite());
                            }
                        }
                    }
                }

                if self.game_over {
                    return Ok(());
                }

                if self.game_type == GameType::LocalAI && self.turn != self.player_color {
                     // Lógica de IA
                     let best_move = ai::get_best_move(&self.board, self.turn);
                     if let Some(((from_x, from_y), (to_x, to_y))) = best_move {
                        self.board.move_piece((from_x, from_y), (to_x, to_y));
                        let _ = self.resources.move_sound.play(ctx);
                        self.turn = self.turn.opposite();
                        if self.board.is_checkmate(self.turn) {
                            self.game_over = true;
                            self.winner = Some(self.turn.opposite());
                        }
                    } else {
                         if self.board.is_checkmate(self.turn) {
                             self.game_over = true;
                             self.winner = Some(self.turn.opposite());
                         } else {
                             self.game_over = true; 
                         }
                    }
                }
            }
        }
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        let mut canvas = graphics::Canvas::from_frame(ctx, Color::BLACK);

        let mode = self.mode;
        match mode {
            AppMode::Menu => {
                let title = Text::new("AJEDREZ");
                canvas.draw(&title, DrawParam::default().dest([350.0, 100.0]).scale([2.0, 2.0]));
                
                let play_ai = Text::new("1. Jugador vs IA");
                canvas.draw(&play_ai, DrawParam::default().dest([350.0, 300.0]));

                let host = Text::new("2. Hospedar Juego");
                canvas.draw(&host, DrawParam::default().dest([350.0, 350.0]));

                let join = Text::new("3. Unirse al Juego");
                canvas.draw(&join, DrawParam::default().dest([350.0, 400.0]));
            },
            AppMode::HostWait => {
                let text = Text::new(format!("Esperando jugador...\nCódigo (IP): {}", self.host_ip));
                canvas.draw(&text, DrawParam::default().dest([250.0, 350.0]).scale([1.5, 1.5]));
            },
            AppMode::JoinInput => {
                 let text = Text::new(format!("Ingrese Código (IP:Puerto):\n{}", self.join_ip_input));
                canvas.draw(&text, DrawParam::default().dest([250.0, 350.0]).scale([1.5, 1.5]));
                let hint = Text::new("Escriba IP y presione Enter");
                canvas.draw(&hint, DrawParam::default().dest([250.0, 450.0]));
            },
            AppMode::Playing => {
                // ... Existing Draw Logic ...
                // Dibujar Tablero
                for y in 0..BOARD_SIZE {
                    for x in 0..BOARD_SIZE {
                        let (draw_x, draw_y) = self.get_view_coords(x, y);

                        let rect = Rect::new(
                            draw_x as f32 * CELL_SIZE,
                            draw_y as f32 * CELL_SIZE,
                            CELL_SIZE,
                            CELL_SIZE,
                        );
                        
                        let color = if (x + y) % 2 == 0 {
                            WHITE_COLOR
                        } else {
                            BLACK_COLOR
                        };

                        let mesh = Mesh::new_rectangle(ctx, DrawMode::fill(), rect, color)?;
                        canvas.draw(&mesh, DrawParam::default());
                    }
                }

                // Resaltar Casilla Seleccionada
                if let Some((sx, sy)) = self.selected_square {
                     let (draw_x, draw_y) = self.get_view_coords(sx, sy);
                     let rect = Rect::new(
                        draw_x as f32 * CELL_SIZE,
                        draw_y as f32 * CELL_SIZE,
                        CELL_SIZE,
                        CELL_SIZE,
                    );
                    let mesh = Mesh::new_rectangle(ctx, DrawMode::fill(), rect, HIGHLIGHT_COLOR)?;
                    canvas.draw(&mesh, DrawParam::default());
                }

                // Resaltar Movimientos Válidos
                for &(mx, my) in &self.valid_moves_for_selected {
                    let (draw_x, draw_y) = self.get_view_coords(mx, my);
                     let rect = Rect::new(
                        draw_x as f32 * CELL_SIZE,
                        draw_y as f32 * CELL_SIZE,
                        CELL_SIZE,
                        CELL_SIZE,
                    );
                    let mesh = Mesh::new_rectangle(ctx, DrawMode::fill(), rect, VALID_MOVE_COLOR)?;
                    canvas.draw(&mesh, DrawParam::default());
                }

                // Dibujar Piezas
                for y in 0..BOARD_SIZE {
                    for x in 0..BOARD_SIZE {
                        if let Some(piece) = self.board.grid[y][x] {
                            let (draw_x, draw_y) = self.get_view_coords(x, y);

                            if let Some(image) = self.resources.pieces.get(&(piece.piece_type, piece.color)) {
                                let target_height = CELL_SIZE * 0.85; 
                                let scale = target_height / image.height() as f32;
                                
                                let img_width = image.width() as f32 * scale;
                                let img_height = image.height() as f32 * scale;
                                
                                let dest_x = (draw_x as f32 * CELL_SIZE) + (CELL_SIZE - img_width) / 2.0;
                                let dest_y = ((draw_y + 1) as f32 * CELL_SIZE) - img_height - (CELL_SIZE * 0.05); 

                                let draw_params = DrawParam::default()
                                    .dest([dest_x, dest_y])
                                    .scale([scale, scale]);
                                
                                canvas.draw(image, draw_params);
                            }
                        }
                    }
                }
            }
        }

        canvas.finish(ctx)?;
        Ok(())
    }

    fn text_input_event(&mut self, _ctx: &mut Context, character: char) -> GameResult {
        if self.mode == AppMode::JoinInput {
             if character.is_control() {
                 return Ok(()); // Ignorar caracteres de control como Backspace aqui, manejar en key_down
             }
             self.join_ip_input.push(character);
        }
        Ok(())
    }

    fn key_down_event(&mut self, _ctx: &mut Context, input: ggez::input::keyboard::KeyInput, _repeated: bool) -> GameResult {
        if self.mode == AppMode::JoinInput {
             match input.keycode {
                 Some(KeyCode::Back) => {
                     self.join_ip_input.pop();
                 },
                 Some(KeyCode::Return) => {
                     // Conectar
                     self.connect_to_host();
                 },
                 _ => {}
            }
        }
        Ok(())
    }

    fn mouse_button_down_event(
        &mut self,
        ctx: &mut Context,
        button: MouseButton,
        x: f32,
        y: f32,
    ) -> GameResult {
        let mode = self.mode;
        match mode {
            AppMode::Menu => {
                // Determinar clics
                // Hitboxes simples basados en posición de texto
                // Play AI: 350, 300
                // Host: 350, 350
                // Join: 350, 400
                if button == MouseButton::Left {
                    if x > 350.0 && x < 600.0 {
                        if y > 300.0 && y < 330.0 {
                            // AI
                            let mut rng = rand::thread_rng();
                            let my_color = if rng.gen_bool(0.5) { PieceColor::White } else { PieceColor::Black };
                            self.reset_game(my_color, GameType::LocalAI);
                        } else if y > 350.0 && y < 380.0 {
                            // Host
                            self.start_host();
                        } else if y > 400.0 && y < 430.0 {
                            // Join
                            self.mode = AppMode::JoinInput;
                            self.join_ip_input.clear();
                        }
                    }
                }
            },
            AppMode::Playing => {
                if self.game_over {
                     self.mode = AppMode::Menu;
                     return Ok(());
                }

                if self.turn != self.player_color {
                    return Ok(());
                }

                if button == MouseButton::Left {
                    let screen_grid_x = (x / CELL_SIZE) as usize;
                    let screen_grid_y = (y / CELL_SIZE) as usize;

                    if screen_grid_x >= BOARD_SIZE || screen_grid_y >= BOARD_SIZE {
                        return Ok(());
                    }

                    // Convertir coords de pantalla a coords lógicas
                    let (grid_x, grid_y) = self.get_view_coords(screen_grid_x, screen_grid_y);

                    // Si hay una pieza seleccionada y se clickea un movimiento válido
                    if let Some(selected) = self.selected_square {
                        if self.valid_moves_for_selected.contains(&(grid_x, grid_y)) {
                            // Ejecutar movimiento
                            self.board.move_piece(selected, (grid_x, grid_y));
                            let _ = self.resources.move_sound.play(ctx);
                            
                            // Enviar movimiento si es Multijugador
                            if self.game_type == GameType::Multiplayer {
                                if let Some(client) = &mut self.network_client {
                                    client.send(NetworkMessage::Move { from: selected, to: (grid_x, grid_y) });
                                }
                            }

                            self.selected_square = None;
                            self.valid_moves_for_selected.clear();
                            self.turn = self.turn.opposite();

                            // Verificar Jaque Mate
                            if self.board.is_checkmate(self.turn) {
                                self.game_over = true;
                                self.winner = Some(self.turn.opposite());
                            }
                            return Ok(());
                        }
                    }

                    // Seleccionar pieza
                    if let Some(piece) = self.board.grid[grid_y][grid_x] {
                        if piece.color == self.player_color {
                            self.selected_square = Some((grid_x, grid_y));
                            self.valid_moves_for_selected = self.board.get_valid_moves((grid_x, grid_y));
                        } else {
                             self.selected_square = None;
                             self.valid_moves_for_selected.clear();
                        }
                    } else {
                         self.selected_square = None;
                         self.valid_moves_for_selected.clear();
                    }
                }
            },
            _ => {}
        }
        Ok(())
    }
}
