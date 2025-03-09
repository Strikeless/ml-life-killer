#![feature(trait_alias, impl_trait_in_bindings)]

use std::{env, path::PathBuf, time::Instant};

use game::{GameTrainerAdapterConfig, GameTrainerAdapterFactory};
use network::Network;
use savedata::SaveData;
use serde::{Deserialize, Serialize};
use trainer::{
    Trainer, TrainerConfig,
    adapter::{TrainerAdapter, TrainerAdapterFactory},
};

mod game;
mod network;
mod savedata;
mod trainer;

#[derive(Debug, Clone, Copy, Deserialize, Serialize)]
struct Config {
    trainer_config: TrainerConfig,
    game_config: GameTrainerAdapterConfig,
}

fn main() {
    let mut args = env::args().skip(1);

    let run_id = args
        .next()
        .and_then(|run_id| (&run_id != "-").then_some(run_id))
        .unwrap_or_else(|| chrono::Local::now().format("%Y%m%d").to_string());

    let (config, network) = if let Some(config_path) = args.next() {
        let SaveData { config, network } = savedata::load(config_path);
        (config, network)
    } else {
        let config = Config {
            trainer_config: TrainerConfig {
                generation_contenders: 8,
                generation_mutations: 5,
                generation_iterations: 4,
                generation_unstable: false,
            },
            game_config: GameTrainerAdapterConfig {
                width: 4,
                height: 4,
                alive_cells: 10,
                max_steps: 10,
                network_consecutive_turns: 1,
                game_consecutive_turns: 0,
                kernel_diameter: 5,
            },
        };

        let network = Network::new(
            config.game_config.kernel_diameter.pow(2), // Input layer height
            3,                                         // Hidden layer count
            3,                                         // Hidden layer height
            2,                                         // Output layer height
        );

        (config, network)
    };

    let adapter_factory = GameTrainerAdapterFactory {
        config: config.game_config,
    };

    let trainer = Trainer::new(config.trainer_config, adapter_factory);

    run_training(run_id, config, trainer, network);
}

fn run_training<A, AF>(
    run_id: String,
    config: Config,
    trainer: Trainer<A, AF>,
    mut network: Network,
) where
    A: TrainerAdapter,
    AF: TrainerAdapterFactory<A>,
{
    let mut prev_score = None;
    let mut best_score = None;
    let mut last_notif_instant = Instant::now();

    let mut last_saved_rolling_average_score = None;
    let mut rolling_average_score = None;

    for generation in 0.. {
        let (trained_network, new_score) = trainer.train_generation(network);
        network = trained_network;

        rolling_average_score = Some(
            rolling_average_score
                .map(|avg| (avg * 999.0 + new_score as f32) / 1000.0)
                .unwrap_or(new_score as f32)
        );

        let improved = last_saved_rolling_average_score
            .is_none_or(|last_avg| rolling_average_score.unwrap() >= last_avg + 10.0);

        let get_millis_since_notif = || {
            Instant::now()
                .duration_since(last_notif_instant)
                .as_millis()
        };

        if improved || get_millis_since_notif() >= 5000 {
            let improved_prefix = (improved)
                .then(|| "IMPROVED".to_owned())
                .unwrap_or(" ".repeat("IMPROVED".len()));

            println!(
                "{} score {:05} -> {:05} (best {:05}, avg {:05.2}), gen {:8}, {} ms",
                improved_prefix,
                prev_score.unwrap_or(0),
                new_score,
                best_score.unwrap_or(0),
                rolling_average_score.unwrap(),
                generation,
                get_millis_since_notif(),
            );

            last_notif_instant = Instant::now();
        }

        if improved {
            let path = PathBuf::from("networks").join(format!("{}_gen{}.json", run_id, generation));

            savedata::save(
                path,
                SaveData {
                    config: config.clone(),
                    network: network.clone(),
                },
            );

            last_saved_rolling_average_score = rolling_average_score;
        }

        best_score = Some(
            best_score
                .map(|prev_best| prev_best.max(new_score))
                .unwrap_or(new_score),
        );
        prev_score = Some(new_score);
    }
}
