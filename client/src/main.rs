#![feature(trait_alias, let_chains)]

use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
    thread,
};

use libgame::{board::GameBoard, rule::Rule, Game};
use ticker::TickerHost;

mod cli;
mod renderer;
mod ticker;

pub struct State {
    game: Game,
    tickers: HashMap<String, TickerHost>,
}

fn main() {
    let game = Game::new(GameBoard::new(20, 20), Rule::default());

    let state_arc = Arc::new(RwLock::new(State {
        game,
        tickers: HashMap::new(),
    }));

    let cli_state_arc = state_arc.clone();
    thread::spawn(move || cli::run_cli(cli_state_arc));

    renderer::run(state_arc);
}
