use std::{
    io,
    process::exit,
    sync::{Arc, RwLock},
    time::Duration,
};

use anyhow::{anyhow, bail, Context};
use libgame::board::{GameBoard, TileState};

use crate::{
    ticker::{nature::NatureTicker, TickerHost},
    State,
};

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
            for _ in 0..times {
                state.game.tick();
            }
        }

        "start" => {
            let name = args.next().ok_or_else(|| anyhow!("No name provided"))?;

            let interval_millis = args
                .next()
                .and_then(|s| u64::from_str_radix(s, 10).ok())
                .unwrap_or(500);

            let ticker = match args.next() {
                Some("nature") | None => Box::new(NatureTicker),
                Some("network") => todo!(),
                Some(name) => bail!("Unknown ticker type '{}'", name),
            };

            let ticker_host = TickerHost::start(
                state_arc.clone(),
                Duration::from_millis(interval_millis),
                ticker,
            );

            state_arc
                .write()
                .unwrap()
                .tickers
                .insert(name.to_owned(), ticker_host);
        }

        "stop" => {
            let name = args.next().ok_or_else(|| anyhow!("No name provided"))?;

            let removed_ticker_host = state_arc.write().unwrap().tickers.remove(name);

            if let Some(removed_ticker_host) = removed_ticker_host {
                removed_ticker_host.stop();
            } else {
                bail!("No active ticker with name '{}'", name);
            }
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
