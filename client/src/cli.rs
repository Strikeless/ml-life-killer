use std::{
    io,
    process::exit,
    sync::{Arc, RwLock},
};

use anyhow::{bail, Context};
use libgame::board::{GameBoard, TileState};

use crate::State;

pub fn run_cli(state_arc: Arc<RwLock<State>>) {
    for line_res in io::stdin().lines() {
        let line = line_res.unwrap();
        let args = line.split_whitespace();

        if let Err(e) = handle_cmd(state_arc.clone(), args) {
            eprintln!("! {e:?}");
        }
    }
}

fn handle_cmd<'a, I>(state_arc: Arc<RwLock<State>>, mut args: I) -> anyhow::Result<()>
where
    I: Iterator<Item = &'a str>,
{
    match args.next().context("No command")? {
        "step" => {
            let times = args.next().unwrap_or("1").parse::<usize>()?;

            let mut state = state_arc.write().unwrap();
            for i in 0..times {
                state.game.tick();
            }
        }

        "run" => {
            let rate = args.next().unwrap_or("2").parse::<usize>()?;

            state_arc.write().unwrap().tick_rate = rate;
            super::spawn_ticker(state_arc);
        }

        "stop" => {
            super::stop_ticker(state_arc);
        }

        "clear" => {
            let mut state = state_arc.write().unwrap();
            for tile in &mut state.game.board.tiles {
                *tile = TileState::Dead;
            }
        }

        "resize" => {
            let width = args.next().context("missing width")?.parse::<usize>()?;

            let height = args.next().context("missing height")?.parse::<usize>()?;

            let mut state = state_arc.write().unwrap();
            let board = &mut state.game.board;

            board.tiles.resize(width * height, TileState::Dead);
            board.width = width;
            board.height = height;
        }

        "random" => {
            let alive_count = args
                .next()
                .context("missing alive count")?
                .parse::<usize>()?;

            let mut state = state_arc.write().unwrap();
            let board = &mut state.game.board;

            *board = GameBoard::new_random(board.width, board.height, alive_count);
        }

        "exit" => {
            exit(0);
        }

        _ => bail!("Unknown command"),
    }

    println!("OK");
    Ok(())
}
