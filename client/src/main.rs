#![feature(trait_alias, let_chains)]

use std::{
    sync::{
        mpsc::{self, Sender},
        Arc, RwLock,
    },
    thread,
    time::Duration,
};

use libgame::{board::GameBoard, rule::Rule, Game};

mod cli;
mod renderer;

pub struct State {
    game: Game,

    tick_rate: usize,
    ticker_stop_sender: Option<Sender<()>>,
}

fn main() {
    let game = Game::new(GameBoard::new(20, 20), Rule::default());

    let state_arc = Arc::new(RwLock::new(State {
        game,
        tick_rate: 0,
        ticker_stop_sender: None,
    }));

    let cli_state_arc = state_arc.clone();
    thread::spawn(move || cli::run_cli(cli_state_arc));

    renderer::run(state_arc);
}

pub fn stop_ticker(state_arc: Arc<RwLock<State>>) {
    let mut state = state_arc.write().unwrap();

    if let Some(ticker_stop_sender) = state.ticker_stop_sender.take() {
        ticker_stop_sender.send(()).unwrap();
    }
}

pub fn spawn_ticker(state_arc: Arc<RwLock<State>>) {
    let (stop_sender, stopper_receiver) = mpsc::channel();

    let state = state_arc.read().unwrap();
    if state.ticker_stop_sender.is_some() {
        // Don't start another ticker if one is already running.
        return;
    }
    drop(state);

    let ticker_state_arc = state_arc.clone();

    thread::spawn(move || {
        while stopper_receiver.try_recv().is_err() {
            let mut state = ticker_state_arc.write().unwrap();
            state.game.tick();

            let sleep_duration = Duration::from_micros(1_000_000 / state.tick_rate as u64);

            drop(state);
            thread::sleep(sleep_duration);
        }
    });

    let mut state = state_arc.write().unwrap();
    state.ticker_stop_sender = Some(stop_sender);
}
