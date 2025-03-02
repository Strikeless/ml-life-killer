#![feature(trait_alias, let_chains)]

use std::{sync::{Arc, RwLock}, thread, time::Duration};

use libgame::{board::GameBoard, rule::Rule, Game};

mod renderer;

pub struct State {
    game: Game,
}

fn main() {
    let game = Game::new(GameBoard::new(20, 20), Rule::default());

    let state_arc = Arc::new(RwLock::new(State { game }));

    let updater_state_arc = state_arc.clone();
    thread::spawn(move || {
        loop {
            let mut state = updater_state_arc.write().unwrap();
            state.game.tick();
            drop(state);

            thread::sleep(Duration::from_millis(500));
        }
    });

    renderer::run(state_arc.clone());
}
