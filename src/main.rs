use bevy_ecs::prelude::*;
use geng::{prelude::*, Camera2d};

mod collision;
mod game;
mod player;
mod types;

use types::*;

fn main() {
    logger::init();
    geng::setup_panic_handler();

    let geng = Geng::new_with(geng::ContextOptions {
        title: "Geng Bevy ECS Pong".to_string(),
        ..default()
    });

    let state = game::Game::new(&geng);
    geng.run(state)
}
