#![feature(let_chains)]

use std::{cmp::Ordering, collections::VecDeque, env, fs, path::{Path, PathBuf}, process::exit, time::Instant};

use adapter::{
    game::{GameTrainerAdapterConfig, GameTrainerAdapterFactory},
    TrainerAdapter, TrainerAdapterFactory,
};
use colored::{ColoredString, Colorize};
use libml::{
    game::{networksave::NetworkSave, NetworkPlayerConfig},
    network::{functions::{Activator, Combinator}, Network, NetworkConfig},
};
use serde::{Deserialize, Serialize};
use trainer::{Trainer, TrainerConfig};

mod adapter;
mod trainer;

#[derive(Debug, Clone, Copy, Deserialize, Serialize)]
struct Config {
    trainer_config: TrainerConfig, // Configuration for the training process.
    adapter_config: GameTrainerAdapterConfig, // Configuration for the games played during training.
}

fn main() {
    let mut args = env::args().skip(1);

    let run_id = args
        .next()
        .and_then(|run_id| (&run_id != "-").then_some(run_id))
        .unwrap_or_else(|| chrono::Local::now().format("%Y%m%d").to_string());

    let config: Config = if let Some(config_path) = args.next() {
        let config_serialized = fs::read(config_path).expect("Couldn't read config");
        serde_json::from_slice(&config_serialized).expect("Couldn't deserialize config")
    } else {
        let default_config = Config {
            trainer_config: TrainerConfig {
                generation_contenders: 16,
                generation_mutations: 4,
                generation_mutations_jitter: 3,
                generation_iterations: 20,
            },
            adapter_config: GameTrainerAdapterConfig {
                width: 16,
                height: 16,
                alive_cells: 128,
                block_size: 1,
                max_rounds: 128,
                disable_nature: false,
                evil: true,
            },
        };

        let default_config_path = Path::new("trainer_default_config.json");
        let write_result = fs::write(
            default_config_path,
            serde_json::to_string_pretty(&default_config).expect("Serializing default config"),
        );

        match write_result {
            Ok(_) => println!("Default config written to '{}'.", default_config_path.display()),
            Err(e) => eprintln!("Error writing default config to '{}': {:?}", default_config_path.display(), e),
        }

        exit(0);
    };

    let network_save = if let Some(network_save_path) = args.next() && network_save_path != "-" {
        NetworkSave::load(network_save_path).expect("Couldn't load network save")
    } else {
        // TODO: Ask network parameters from the user interactively
        let kernel_diameter: usize = 5;

        let network = Network::new(
            NetworkConfig {
                activator: Activator::ReLU,
                combinator: Combinator::Mul,
            },
            kernel_diameter.pow(2), // Input layer height
            3,                      // Hidden layer count
            16,                     // Hidden layer height
            2,                      // Output layer height
        );

        let player_config = NetworkPlayerConfig {
            kernel_diameter,
            use_kernel_cache: false,
        };

        NetworkSave {
            network,
            player_config,
        }
    };

    let adapter_factory = GameTrainerAdapterFactory {
        config: config.adapter_config,
        player_config: network_save.player_config,
    };

    let trainer = Trainer::new(config.trainer_config, adapter_factory);

    run_training(
        run_id,
        trainer,
        network_save.player_config,
        network_save.network,
    );
}

fn run_training<A, AF>(
    run_id: String,
    trainer: Trainer<A, AF>,

    network_player_config: NetworkPlayerConfig,
    mut network: Network,
) where
    A: TrainerAdapter,
    AF: TrainerAdapterFactory<A>,
{
    let mut score = ValueThingy::new(50);
    let mut last_save_avg_score = None;

    let mut last_notif_instant = Instant::now();
    let mut last_notif_generation = 0;
    let mut last_notif_score = score.clone();

    let mut last_save_instant = Instant::now();

    for generation in 0.. {
        let (trained_network, new_score) = trainer.train_generation(network);
        network = trained_network;

        score.update(new_score);

        let avg_score = score.average();
        if last_save_avg_score.is_none() && score.averages_ready() {
            last_save_avg_score = Some(avg_score);
        }

        let improved =
            last_save_avg_score.is_some_and(|saved_avg_score| avg_score >= saved_avg_score + 10.0);

        let current_instant = Instant::now();

        let get_millis_since_notif = || {
            current_instant
                .duration_since(last_notif_instant)
                .as_millis()
        };

        if improved || get_millis_since_notif() >= 1000 {
            let improved_prefix = improved
                .then(|| "IMPROVED".to_owned())
                .unwrap_or(" ".repeat("IMPROVED".len()))
                .green();

            let notif_passed_time = current_instant.duration_since(last_notif_instant);
            let generations_per_second =
                (generation - last_notif_generation) as f32 / notif_passed_time.as_secs_f32();

            println!(
                "{improved_prefix} gen {generation:7}: {} | {} < {} < {} | {:4.2} gen/s",
                option_change_colored(score.value(), last_notif_score.value(), |score| format!(
                    "{:4}",
                    score
                )),
                option_change_colored(score.min(), last_notif_score.min(), |min| format!(
                    "{:4}",
                    min
                )),
                change_colored(score.average(), last_notif_score.average(), |avg| format!(
                    "{:4.2}",
                    avg
                )),
                option_change_colored(score.max(), last_notif_score.max(), |max| format!(
                    "{:4}",
                    max
                )),
                generations_per_second,
            );

            last_notif_instant = current_instant;
            last_notif_generation = generation;
            last_notif_score = score.clone();
        }

        if improved || current_instant.duration_since(last_save_instant).as_secs() > 60 {
            let path = PathBuf::from("networks").join(format!("{}_gen{}.json", run_id, generation));

            NetworkSave {
                network: network.clone(),
                player_config: network_player_config,
            }
            .save(path)
            .expect("Couldn't save network");

            last_save_avg_score = Some(avg_score);
            last_save_instant = current_instant;
        }
    }
}

fn option_change_colored<T>(
    new: Option<T>,
    old: Option<T>,
    fmt_fn: fn(T) -> String,
) -> ColoredString
where
    T: PartialOrd + Default + Copy,
{
    let new_or_default = new.unwrap_or_default();
    change_colored(new_or_default, old.unwrap_or(new_or_default), fmt_fn)
}

fn change_colored<T>(new: T, old: T, fmt_fn: fn(T) -> String) -> ColoredString
where
    T: PartialOrd,
{
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

    pub fn averages_ready(&self) -> bool {
        self.values.len() >= self.len
    }
}
