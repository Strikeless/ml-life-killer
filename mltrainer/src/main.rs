#![feature(let_chains)]

use std::{cmp::Ordering, collections::VecDeque, env, fmt::{format, Arguments}, path::PathBuf, time::Instant};

use adapter::{game::{GameTrainerAdapterConfig, GameTrainerAdapterFactory}, TrainerAdapter, TrainerAdapterFactory};
use colored::{ColoredString, Colorize};
use libml::{game::NetworkPlayerConfig, network::Network};
use savedata::SaveData;
use serde::{Deserialize, Serialize};
use trainer::{Trainer, TrainerConfig};

mod adapter;
mod savedata;
mod trainer;

#[derive(Debug, Clone, Copy, Deserialize, Serialize)]
struct Config {
    trainer_config: TrainerConfig, // Configuration for the training process.
    adapter_config: GameTrainerAdapterConfig, // Configuration for the games played during training.
    player_config: NetworkPlayerConfig, // Configuration for the network.
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
                generation_contenders: 4,
                generation_mutations: 5,
                generation_iterations: 5,
                generation_unstable: false,
            },
            adapter_config: GameTrainerAdapterConfig {
                width: 8,
                height: 8,
                alive_cells: 2,
                max_steps: 4,
                network_consecutive_turns: 1,
                game_consecutive_turns: 0,
            },
            player_config: NetworkPlayerConfig {
                kernel_diameter: 1,
            },
        };

        let network = Network::new(
            config.player_config.kernel_diameter.pow(2), // Input layer height
            3,                                           // Hidden layer count
            3,                                           // Hidden layer height
            2,                                           // Output layer height
        );

        (config, network)
    };

    let adapter_factory = GameTrainerAdapterFactory {
        config: config.adapter_config,
        player_config: config.player_config,
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
    let mut score = ValueThingy::new(1000);
    let mut saved_avg_score = None;

    let mut last_notif_instant = Instant::now();
    let mut last_notif_generation = 0;
    let mut last_notif_score = score.clone();

    for generation in 0.. {
        let (trained_network, new_score) = trainer.train_generation(network);
        network = trained_network;

        score.update(new_score);

        let avg_score = score.average();
        if saved_avg_score.is_none() {
            saved_avg_score = Some(avg_score);
        }
        let saved_avg_score = saved_avg_score.as_mut().unwrap();

        let improved = avg_score >= *saved_avg_score + 10.0;

        let get_millis_since_notif = || {
            Instant::now()
                .duration_since(last_notif_instant)
                .as_millis()
        };

        if improved || get_millis_since_notif() >= 1000 {
            let current_instant = Instant::now();

            let improved_prefix = (improved)
                .then(|| "IMPROVED".to_owned())
                .unwrap_or(" ".repeat("IMPROVED".len()))
                .green();

            let notif_passed_time = current_instant.duration_since(last_notif_instant);
            let generations_per_second = (generation - last_notif_generation) as f32 / notif_passed_time.as_secs_f32();

            println!(
                "{improved_prefix} gen {generation:7}: {} | {} < {} < {} | {:4.2} gen/s",
                option_change_colored(score.value(), last_notif_score.value(), |score| format!("{:4}", score)),
                option_change_colored(score.min(), last_notif_score.min(), |min| format!("{:4}", min)),
                change_colored(score.average(), last_notif_score.average(), |avg| format!("{:4.2}", avg)),
                option_change_colored(score.max(), last_notif_score.max(), |max| format!("{:4}", max)),
                generations_per_second,
            );

            last_notif_instant = Instant::now();
            last_notif_generation = generation;
            last_notif_score = score.clone();
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

            *saved_avg_score = avg_score;
        }
    }
}

fn option_change_colored<T>(new: Option<T>, old: Option<T>, fmt_fn: fn(T) -> String) -> ColoredString where T: PartialOrd + Default {
    change_colored(new.unwrap_or_default(), old.unwrap_or_default(), fmt_fn)
}

fn change_colored<T>(new: T, old: T, fmt_fn: fn(T) -> String) -> ColoredString where T: PartialOrd {
    match new.partial_cmp(&old) {
        Some(Ordering::Equal) | None => fmt_fn(new).white(),
        Some(Ordering::Greater) => fmt_fn(new).bright_green(),
        Some(Ordering::Less) => fmt_fn(new).bright_red(),
    }
}

#[derive(Debug, Clone)]
struct ValueThingy {
    values: VecDeque<isize>,
    len: usize,
}

impl ValueThingy {
    pub fn new(len: usize) -> Self {
        Self {
            values: VecDeque::with_capacity(len),
            len,
        }
    }

    pub fn update(&mut self, value: isize) {
        if self.values.len() == self.len {
            self.values.pop_front();
        }

        self.values.push_back(value);
    }

    pub fn value(&self) -> Option<isize> {
        self.values.back().copied()
    }

    pub fn average(&self) -> f32 {
        let value_count = self.values.len();
        let sum: isize = self.values.iter().sum();
        sum as f32 / value_count as f32
    }

    pub fn max(&self) -> Option<isize> {
        self.values.iter().max().copied()
    }

    pub fn min(&self) -> Option<isize> {
        self.values.iter().min().copied()
    }
}
