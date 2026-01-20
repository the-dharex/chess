#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use ggez::{conf, event, ContextBuilder};
use std::path::PathBuf;

mod constants;
mod resources;
mod pieces;
mod board;
mod ai;
mod game;
mod network;

use constants::SCREEN_SIZE;
use game::GameState;

fn main() {
    let resources_dir = if let Ok(manifest_dir) = std::env::var("CARGO_MANIFEST_DIR") {
        let mut path = PathBuf::from(manifest_dir);
        path.push("assets");
        path
    } else {
        PathBuf::from("./assets")
    };

    let (mut ctx, event_loop) = ContextBuilder::new("chess", "thedharex")
        .window_setup(conf::WindowSetup::default().title("Ajedrez - Rust"))
        .window_mode(conf::WindowMode::default()
            .dimensions(SCREEN_SIZE.0, SCREEN_SIZE.1)
            .resizable(false)
            .maximized(false))
        .add_resource_path(resources_dir)
        .build()
        .expect("¡No se pudo crear el contexto de ggez!");

    ctx.gfx.window().set_resizable(false);
    ctx.gfx.window().set_maximized(false);
    let _ = ctx.gfx.window().set_inner_size(ggez::winit::dpi::PhysicalSize::new(SCREEN_SIZE.0, SCREEN_SIZE.1));

    let _args: Vec<String> = std::env::args().collect();
    // Tal vez manejar argumentos después

    let state = GameState::new(&mut ctx).expect("No se pudo crear el estado del juego");

    event::run(ctx, event_loop, state);
}
